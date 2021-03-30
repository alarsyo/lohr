{
  inputs = {
    naersk = {
      url = "github:nmattia/naersk/master";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    mozillapkgs = {
      url = "github:mozilla/nixpkgs-mozilla";
      flake = false;
    };
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
  };

  outputs = { self, naersk, mozillapkgs, nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };

        mozilla = pkgs.callPackage (mozillapkgs + "/package-set.nix") { };
        rustNightly = (mozilla.rustChannelOf {
          date = "2021-03-29";
          channel = "nightly";
          sha256 = "sha256-Y94CnslybZgiZlNVV6Cg0TUPV2OeDXakPev1kqdt9Kk=";
        }).rust;

        naersk-lib = pkgs.callPackage naersk {
          cargo = rustNightly;
          rustc = rustNightly;
        };
      in
      {
        defaultPackage = naersk-lib.buildPackage {
          src = ./.;
          pname = "lohr";

          meta = with pkgs.lib; {
            description = "A Git mirroring tool";
            homepage = "https://github.com/alarsyo/lohr";
            license = with licenses; [ mit asl20 ];
            platforms = platforms.unix;
          };
        };

        defaultApp = flake-utils.lib.mkApp {
          drv = self.defaultPackage."${system}";
        };

        devShell = pkgs.mkShell {
          buildInputs = with pkgs; [
            nixpkgs-fmt
            pre-commit
            rustPackages.clippy
            rustNightly
            rustfmt
          ];

          RUST_SRC_PATH = pkgs.rustPlatform.rustLibSrc;
        };
      });
}
