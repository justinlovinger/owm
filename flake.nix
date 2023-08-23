{
  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    naersk,
    nixpkgs,
    utils,
  }:
    utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {inherit system;};
      naersk-lib = pkgs.callPackage naersk {};
    in {
      defaultPackage = naersk-lib.buildPackage {
        src = ./.;
        doCheck = true;
      };
      devShell = with pkgs;
        mkShell {
          buildInputs = [
            cargo
            rust-analyzer
            rustc
            rustfmt
            rustPackages.clippy
          ];
          RUST_SRC_PATH = rustPlatform.rustLibSrc;
          PROPTEST_DISABLE_FAILURE_PERSISTENCE = 1;
        };
    });
}
