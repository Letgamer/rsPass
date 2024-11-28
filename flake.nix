{
  description = "Rust application with Docker output";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        # Your package name here
        packageName = "backend_rspass";


        # Build the Rust package
        rustPkg = pkgs.rustPlatform.buildRustPackage {
          pname = packageName;
          version = "0.1.0";
          src = ./.;

          # Include Cargo.lock if it exists
          cargoLock = if builtins.pathExists ./Cargo.lock then {
            lockFile = ./Cargo.lock;
          } else null;

          # Native build inputs required for compilation
          nativeBuildInputs = with pkgs; [
            pkg-config
          ];

          # Disable automatic tests
          doCheck = false;

          # Custom test command
          checkPhase = ''
            cargo test -- --test-threads=1
          '';
        };

        # Create a minimal Docker image for the Rust application
        dockerImage = pkgs.dockerTools.buildLayeredImage {
          name = packageName;

          # Tag the image with both the Git tag and "latest"
          tag = "latest";

          contents = [ rustPkg ];

          config = {
            Cmd = [ "${rustPkg}/bin/${packageName}" ];
            WorkingDir = "/app"; # Set the working directory
            #User = "1000:1000";  # Use a non-root user
          };
        };

      in
      {
        packages = {
          default = rustPkg;   # Default package to build
          docker = dockerImage; # Docker image for deployment
        };

        # Development shell environment
        devShells.default = pkgs.mkShell {
          buildInputs = [
            pkgs.rust-bin.stable.latest.default
            pkgs.cargo-watch
            pkgs.cargo-depgraph
            pkgs.cargo-cyclonedx
            pkgs.graphviz
            pkgs.sbomnix
          ];

          shellHook = ''
            echo "Welcome to the Rust development shell!"
            echo "Only run the aliases if you have built the package!"
            export PATH=$PATH:${pkgs.graphviz}/bin
            alias cargo-graph='cargo depgraph --all-deps --dedup-transitive-deps | dot -Tpng > dependencies/cargo-graph.png'
            alias cargo-sbom='cargo cyclonedx --target all --spec-version 1.3 -f json --override-filename cargo.cdx && mv cargo.cdx.json dependencies/'
            alias nix-graph='nixgraph ./result/ --depth=100 && cp -r graph.png dependencies/nix-graph.png'
            alias nix-sbom='sbomnix ./result --depth 100 --csv dependencies/nix-sbom.csv --cdx dependencies/nix-sbom.cdx.json --spdx dependencies/nix-sbom.spdx.json && rm http_cache.sqlite'
            alias nix-vuln='vulnxscan --sbom dependencies/nix-sbom.cdx.json --out dependencies/nix-vulns.csv && rm http_cache.sqlite'
            alias cargo-vuln='vulnxscan --sbom dependencies/cargo.cdx.json --out dependencies/cargo-vulns.csv && rm http_cache.sqlite'
          '';
        };
      }
    );
}
