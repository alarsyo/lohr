# lohr

`lohr` is a Git mirroring tool.

I created it to solve a simple problem I had: I host my own git server at
<https://git.alarsyo.net>, but want to mirror my public projects to GitHub /
GitLab, for backup and visibility purposes.

GitLab has a mirroring setting, but it doesn't allow for multiple mirrors, as
far as I know. I also wanted my instance to be the single source of truth.

## How it works

Gitea is setup to send webhooks to my `lohr` server on every push update. When
`lohr` receives a push, it clones the concerned repository, or updates it if
already cloned. Then it pushes the update to **all remotes listed** in the
[.lohr](.lohr) file at the repo root.

### Destructive

This is a very destructive process: anything removed from the single source of
truth is effectively removed from any mirror as well.

## Installing

`lohr` is [published on crates.io](https://crates.io/crates/lohr), so you can
install it with `cargo install`:

    $ cargo install lohr

Note: currently this method won't get you the latest version of `lohr`, as it
depends on Rocket v0.5.0, which isn't released yet. Updated versions of `lohr`
will be published on crates.io as soon as Rocket v0.5.0 releases.

## Setup

### Quickstart

Setting up `lohr` should be quite simple:

1.  Create a `Rocket.toml` file and [add your
    configuration](https://rocket.rs/v0.4/guide/configuration/).

2.  Export a secret variable:
    
        $ export LOHR_SECRET=42 # please don't use this secret

3.  Run `lohr`:
    
        $ cargo run # or `cargo run --release` for production usage

4.  Configure your favorite git server to send a webhook to `lohr`'s address on
    every push event.
    
    I used [Gitea's webhooks format](https://docs.gitea.io/en-us/webhooks/), but
    I **think** they're similar to GitHub and GitLab's webhooks, so these should
    work too! (If they don't, **please** file an issue!)
    
    Don't forget to set the webhook secret to the one you chose above.

5.  Add a `.lohr` file containing the remotes you want to mirror this repo to:
    
        git@github.com:you/your_repo
    
    and push it. That's it! `lohr` is mirroring your repo now.


### Configuration

#### Home directory

`lohr` needs a place to clone repos and store its data. By default, it's the
current directory, but you can set the `LOHR_HOME` environment variable to
customize it.

#### Shared secret

As shown in the quickstart guide, you **must** set the `LOHR_SECRET` environment
variable.

#### Extra remote configuration

You can provide `lohr` with a YAML file containing additional configuration. You
can pass its path to the `--config` flag when launching `lohr`. If no
configuration is provided via a CLI flag, `lohr` will check the `LOHR_CONFIG`
environment variable. If the environment variable isn't set either, it will
check in `LOHR_HOME` is a `lohr-config.yaml` file exists, and try to load it.

This file takes the following format:

``` yaml
default_remotes:
    - "git@github:user"
    - "git@gitlab:user"

additional_remotes:
    - "git@git.sr.ht:~user"

blacklist:
    - "private-.*"
```

- `default_remotes` is a list of remotes to use if no `.lohr` file is found in a
  repository.
- `additional_remotes` is a list of remotes to add in any case, whether the
  original set of remotes is set via `default_remotes` or via a `.lohr` file.
- `blacklist` is a list of regular expressions to match against the full
  repository names. Any that matches will not be mirrored, even if it contains a
  `.lohr` file.

Both settings take as input a list of "stems", i.e. incomplete remote addresses,
to which the repo's name will be appended (so for example, if my
`default_remotes` contains `git@github.com:alarsyo`, and a push event webhook is
received for repository `git@gitlab.com:some/long/path/repo_name`, then the
mirror destination will be `git@github.com:alarsyo/repo_name`.

## Contributing

I accept patches anywhere! Feel free to [open a GitHub Pull
Request](https://github.com/alarsyo/lohr/pulls), [a GitLab Merge
Request](https://gitlab.com/alarsyo/lohr/-/merge_requests), or [send me a patch
by email](https://lists.sr.ht/~alarsyo/lohr-dev)!

## Why lohr?

I was looking for a cool name, and thought about the Magic Mirror in Snow White.
Some **[furious wikipedia
searching](https://en.wikipedia.org/wiki/Magic_Mirror_(Snow_White))** later, I
found that the Magic Mirror was probably inspired by [the Talking Mirror in Lohr
am Main](http://spessartmuseum.de/seiten/schneewittchen_engl.html). That's it,
that's the story.

## License

`lohr` is distributed under the terms of both the MIT license and the Apache
License (Version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) for details.
