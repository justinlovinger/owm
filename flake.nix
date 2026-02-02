{
  inputs = {
    systems.url = "github:nix-systems/default";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    naersk = {
      url = "github:nix-community/naersk/master";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      systems,
      nixpkgs,
      naersk,
    }:
    let
      pkgs = eachSystem (
        system:
        import nixpkgs {
          inherit system;
          overlays = [ naersk.overlays.default ];
        }
      );
      eachSystem = nixpkgs.lib.genAttrs (import systems);
    in
    {
      packages = eachSystem (system: {
        default = pkgs.${system}.naersk.buildPackage {
          src = ./.;
          doCheck = true;
          cargoTestOptions = xs: xs ++ [ "--all" ];
        };
      });
      devShells = eachSystem (system: {
        default =
          with pkgs.${system};
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
    };
}
