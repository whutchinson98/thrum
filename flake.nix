{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    fenix.url = "github:nix-community/fenix";
    fenix.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      fenix,
    }:
    {
      overlays.default = final: prev: {
        thrum = final.rustPlatform.buildRustPackage {
          pname = "thrum";
          version = "0.0.4";
          src = self;
          cargoLock.lockFile = ./Cargo.lock;
          nativeBuildInputs = [ final.pkg-config ];
          buildInputs =
            [ final.openssl ]
            ++ final.lib.optionals final.stdenv.hostPlatform.isDarwin [
              final.darwin.apple_sdk.frameworks.Security
              final.darwin.apple_sdk.frameworks.SystemConfiguration
            ];
        };
      };
    }
    // flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          system = system;
          overlays = [ self.overlays.default ];
        };
        devPackages = with pkgs; [
          cargo-info
          cargo-udeps
          just
          pkg-config
          (
            with fenix.packages.${system};
            combine [
              complete.rustc
              complete.rust-src
              complete.cargo
              complete.clippy
              complete.rustfmt
              complete.rust-analyzer
            ]
          )
        ];

        libraries = with pkgs; [
          openssl
        ];
      in
      {
        packages.default = pkgs.thrum;

        devShell = pkgs.mkShell {
          buildInputs = devPackages ++ libraries;

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath libraries;
        };
      }
    );
}
