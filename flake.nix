{
  description = "farc flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
    crane,
  }: let
  in
    flake-utils.lib.eachDefaultSystem
    (system: let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };
        craneLib = crane.lib.${system};
        src = craneLib.cleanCargoSource ./.;
        cargoArtifacts = craneLib.buildDepsOnly {
          inherit src;
        };
      in rec {
        packages = rec {
          farc = craneLib.buildPackage {
            inherit src cargoArtifacts;

            # Add extra inputs here or any other derivation settings
            doCheck = false;
            # buildInputs = [];
            # nativeBuildInputs = [];
          };
          farc-app = craneLib.buildPackage {
            inherit src cargoArtifacts;

            cargoExtraArgs = "--example farc";

            # Add extra inputs here or any other derivation settings
            doCheck = false;
            # buildInputs = [];
            # nativeBuildInputs = [];
          };
          default = farc-app;
        };
        apps = rec {
          farc = {
            type = "app";
            program = "${packages.farc-app}/bin/farc";
          };
          default = farc;
        };
        devShells.default = with pkgs; pkgs.mkShell rec {
          nativeBuildInputs = [
            pkg-config
          ];
          buildInputs = [
            (rust-bin.selectLatestNightlyWith (toolchain: toolchain.default.override {
              extensions = [ "rust-src" "rust-analyzer" ];
              targets = [];
            }))
          ];
        };
      }
    );
}
