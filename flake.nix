{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        defaultPackage = pkgs.rustPlatform.buildRustPackage {
          pname = "lohr";
          version = "0.3.1";

          src = ./.;

          cargoSha256 = "sha256-XnBvb13Pv7bNTLCL3WV+bxRK0/uMEKA1/Bk0Tfua3Rs=";

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
            cargo
            clippy
            nixpkgs-fmt
            pre-commit
            rustPackages.clippy
            rustc
            rustfmt
          ];

          RUST_SRC_PATH = pkgs.rustPlatform.rustLibSrc;
        };
      });
}
