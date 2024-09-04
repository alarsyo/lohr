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
          version = "0.4.5";

          src = ./.;

          cargoHash = "sha256-U6lW2kN/2MXDIl97wg4dNdtm9C4K7gGP0cfhu49zRUQ=";

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
            rust-analyzer
          ];

          RUST_SRC_PATH = pkgs.rustPlatform.rustLibSrc;
        };
      });
}
