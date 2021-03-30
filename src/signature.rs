use std::{
    io::Read,
    ops::{Deref, DerefMut},
};

use rocket::{
    data::{FromData, Outcome},
    http::ContentType,
    State,
};
use rocket::{
    data::{Transform, Transformed},
    http::Status,
};
use rocket::{Data, Request};

use anyhow::anyhow;
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

const LIMIT: u64 = 1 << 20;

// This is a one to one implementation of request_contrib::Json's FromData, but with HMAC
// validation.
//
// Tracking issue for chaining Data guards to avoid this:
// https://github.com/SergioBenitez/Rocket/issues/775
impl<'a, T> FromData<'a> for SignedJson<T>
where
    T: Deserialize<'a>,
{
    type Error = anyhow::Error;
    type Owned = String;
    type Borrowed = str;

    fn transform(
        request: &Request,
        data: Data,
    ) -> rocket::data::Transform<Outcome<Self::Owned, Self::Error>> {
        let size_limit = request.limits().get("json").unwrap_or(LIMIT);
        let mut s = String::with_capacity(512);
        match data.open().take(size_limit).read_to_string(&mut s) {
            Ok(_) => Transform::Borrowed(Outcome::Success(s)),
            Err(e) => Transform::Borrowed(Outcome::Failure((
                Status::BadRequest,
                anyhow!("couldn't read json: {}", e),
            ))),
        }
    }

    fn from_data(request: &Request, o: Transformed<'a, Self>) -> Outcome<Self, Self::Error> {
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

        let signature = signatures[0];

        let content = o.borrowed()?;

        let secret = request.guard::<State<Secret>>().unwrap();

        if !validate_signature(&secret.0, &signature, content) {
            return Outcome::Failure((Status::BadRequest, anyhow!("couldn't verify signature")));
        }

        let content = match serde_json::from_str(content) {
            Ok(content) => content,
            Err(e) => {
                return Outcome::Failure((
                    Status::BadRequest,
                    anyhow!("couldn't parse json: {}", e),
                ))
            }
        };

        Outcome::Success(SignedJson(content))
    }
}
