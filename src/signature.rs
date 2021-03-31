use std::{
    io,
    ops::{Deref, DerefMut},
};

use rocket::{
    data::{ByteUnit, FromData, Outcome},
    http::ContentType,
    State,
};
use rocket::{http::Status, local_cache};
use rocket::{Data, Request};

use anyhow::{anyhow, Context};
use serde::Deserialize;

use crate::Secret;

const X_GITEA_SIGNATURE: &str = "X-Gitea-Signature";

fn validate_signature(secret: &str, signature: &str, data: &str) -> bool {
    use hmac::{Hmac, Mac, NewMac};
    use sha2::Sha256;

    type HmacSha256 = Hmac<Sha256>;

    let mut mac = HmacSha256::new_varkey(secret.as_bytes()).expect("this should never fail");

    mac.update(data.as_bytes());

    match hex::decode(signature) {
        Ok(bytes) => mac.verify(&bytes).is_ok(),
        Err(_) => false,
    }
}

pub struct SignedJson<T>(pub T);

impl<T> Deref for SignedJson<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> DerefMut for SignedJson<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

const LIMIT: ByteUnit = ByteUnit::Mebibyte(1);

impl<'r, T: Deserialize<'r>> SignedJson<T> {
    fn from_str(s: &'r str) -> anyhow::Result<Self> {
        serde_json::from_str(s)
            .map(SignedJson)
            .context("could not parse json")
    }
}

// This is a one to one implementation of request_contrib::Json's FromData, but with HMAC
// validation.
//
// Tracking issue for chaining Data guards to avoid this:
// https://github.com/SergioBenitez/Rocket/issues/775
#[rocket::async_trait]
impl<'r, T> FromData<'r> for SignedJson<T>
where
    T: Deserialize<'r>,
{
    type Error = anyhow::Error;

    async fn from_data(request: &'r Request<'_>, data: Data) -> Outcome<Self, Self::Error> {
        let json_ct = ContentType::new("application", "json");
        if request.content_type() != Some(&json_ct) {
            return Outcome::Failure((Status::BadRequest, anyhow!("wrong content type")));
        }

        let signatures = request.headers().get(X_GITEA_SIGNATURE).collect::<Vec<_>>();
        if signatures.len() != 1 {
            return Outcome::Failure((
                Status::BadRequest,
                anyhow!("request header needs exactly one signature"),
            ));
        }

        let size_limit = request.limits().get("json").unwrap_or(LIMIT);
        let content = match data.open(size_limit).into_string().await {
            Ok(s) if s.is_complete() => s.into_inner(),
            Ok(_) => {
                let eof = io::ErrorKind::UnexpectedEof;
                return Outcome::Failure((
                    Status::PayloadTooLarge,
                    io::Error::new(eof, "data limit exceeded").into(),
                ));
            }
            Err(e) => return Outcome::Failure((Status::BadRequest, e.into())),
        };

        let signature = signatures[0];
        let secret = request.guard::<State<Secret>>().await.unwrap();

        if !validate_signature(&secret.0, &signature, &content) {
            return Outcome::Failure((Status::BadRequest, anyhow!("couldn't verify signature")));
        }

        let content = match Self::from_str(local_cache!(request, content)) {
            Ok(content) => Outcome::Success(content),
            Err(e) => Outcome::Failure((Status::BadRequest, e)),
        };

        content
    }
}
