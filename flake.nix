{
  description = "A flake for your Rust backend";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, ... }:
    flake-utils.lib.simpleFlake {
      inherit self;
      packages = {
        default = nixpkgs.lib.mkFlake {
          packages.default = pkgs: pkgs.rustPlatform.buildRustPackage {
            pname = "backend";
            src = ./.;
          };
        };
      };
      devShell = pkgs: pkgs.mkShell {
        buildInputs = [
          pkgs.rustc
          pkgs.cargo
        ];
      };
    };
}
