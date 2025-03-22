{
  description = "Causality: A distributed computing system with algebraic effects";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    crate2nix = {
      url = "github:kolloch/crate2nix";
      flake = false;
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, crate2nix, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system;
          overlays = overlays;
        };
        
        # For macOS, set a deployment target
        darwinDeploymentTarget = "11.0";
        
        # Create a set of common environment variables
        commonEnv = {
          # Always set MACOSX_DEPLOYMENT_TARGET, it won't affect non-macOS systems
          MACOSX_DEPLOYMENT_TARGET = darwinDeploymentTarget;
        };

        # Bring crate2nix into scope - this is the corrected reference
        crate2nixPkg = pkgs.callPackage crate2nix {};

        # Rust version to use (with extensions)
        rustVersion = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "clippy" "rustfmt" ];
        };

        # Common dependencies for both build methods
        commonInputs = with pkgs; [
          openssl
          pkg-config
        ] ++ lib.optionals stdenv.isDarwin [
          darwin.apple_sdk.frameworks.Security
          darwin.apple_sdk.frameworks.SystemConfiguration
        ];

        # Create a script to ensure everything runs in the Nix environment
        nixEnvScript = pkgs.writeShellScriptBin "causality-env" ''
          #!/usr/bin/env bash
          # This script ensures all commands run in the Nix environment
          export MACOSX_DEPLOYMENT_TARGET="${darwinDeploymentTarget}"
          
          if [ $# -eq 0 ]; then
            echo "Entering Nix environment for Causality"
            echo "MACOSX_DEPLOYMENT_TARGET is set to $MACOSX_DEPLOYMENT_TARGET"
            exec nix develop
          else
            echo "Running command in Nix environment: $@"
            echo "MACOSX_DEPLOYMENT_TARGET is set to $MACOSX_DEPLOYMENT_TARGET"
            nix develop --command "$@"
          fi
        '';

        # Generate Cargo.nix if it doesn't exist
        generatedCargoNix = pkgs.runCommand "generate-cargo-nix" (commonEnv // {
          buildInputs = [ crate2nixPkg ];
        }) ''
          mkdir -p $out
          cd ${self}
          ${crate2nixPkg}/bin/crate2nix generate
          cp Cargo.nix $out/
        '';

        # Import the generated Cargo.nix
        importedCargoNix = if builtins.pathExists ./Cargo.nix 
                          then import ./Cargo.nix 
                          else import "${generatedCargoNix}/Cargo.nix";

        # Create the crate2nix package
        crate2nixPackage = importedCargoNix {
          inherit pkgs;
          rootFeatures = ["minimal-build"];
          defaultCrateOverrides = pkgs.defaultCrateOverrides // {
            # Add custom crate overrides if needed
            openssl-sys = attrs: {
              buildInputs = (attrs.buildInputs or []) ++ [ pkgs.openssl ];
              nativeBuildInputs = (attrs.nativeBuildInputs or []) ++ [ pkgs.pkg-config ];
              inherit (commonEnv) MACOSX_DEPLOYMENT_TARGET;
            };
            # Add a fix for proc-macro-crate
            proc-macro-crate = attrs: {
              patches = [
                ./nix/patches/proc-macro-crate.patch
              ];
              inherit (commonEnv) MACOSX_DEPLOYMENT_TARGET;
            };
            # Ensure all crates have the correct MACOSX_DEPLOYMENT_TARGET
            ".*" = attrs: {
              inherit (commonEnv) MACOSX_DEPLOYMENT_TARGET;
            };
          };
        };

        # Standard Rust build as fallback
        causalityStdPkg = pkgs.rustPlatform.buildRustPackage ({
          pname = "causality";
          version = "0.1.0";
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
          };
          buildInputs = commonInputs;
          nativeBuildInputs = [ pkgs.pkg-config ];
          cargoBuildFlags = ["--no-default-features" "--features" "minimal-build"];
        } // commonEnv);

      in rec {
        # Default package (crate2nix build)
        packages = {
          causality = crate2nixPackage.rootCrate.build;
          causality-std = causalityStdPkg;
          default = packages.causality;
          # Add the environment script as a package
          env-script = nixEnvScript;
        };

        # Apps for different build tasks 
        apps = {
          # Generate the Cargo.nix file
          generate-cargo-nix = {
            type = "app";
            program = toString (pkgs.writeShellScript "generate-cargo-nix" ''
              export MACOSX_DEPLOYMENT_TARGET="${darwinDeploymentTarget}"
              cd ${self}
              ${crate2nixPkg}/bin/crate2nix generate
              echo "Cargo.nix generated successfully"
            '');
          };
          # Standard build
          build = {
            type = "app";
            program = toString (pkgs.writeShellScript "build-causality" ''
              export MACOSX_DEPLOYMENT_TARGET="${darwinDeploymentTarget}"
              cd ${self}
              ${rustVersion}/bin/cargo build
              echo "Build completed successfully"
            '');
          };
          default = apps.generate-cargo-nix;
          # Add an app to enter the environment
          env = {
            type = "app";
            program = "${nixEnvScript}/bin/causality-env";
          };
        };

        # Development shell with explicitly exported environment variables
        devShells.default = pkgs.mkShell ({
          buildInputs = commonInputs ++ [
            rustVersion
            crate2nixPkg
            pkgs.cargo-audit
            pkgs.cargo-edit
            nixEnvScript  # Include our environment script
          ];
          
          # Explicitly set environment variables
          inherit (commonEnv) MACOSX_DEPLOYMENT_TARGET;
          
          shellHook = ''
            echo "Rust development environment for Causality loaded"
            echo "MACOSX_DEPLOYMENT_TARGET set to $MACOSX_DEPLOYMENT_TARGET"
            echo ""
            echo "Build commands:"
            echo "- nix run .#generate-cargo-nix  # Generate Cargo.nix file"
            echo "- nix build                     # Build with crate2nix (default)"
            echo "- nix build .#causality-std     # Build with standard Rust"
            echo "- nix run .#build               # Build with cargo"
            echo ""
            echo "Run everything in the Nix environment with:"
            echo "- ./result/bin/causality-env cargo build   # Run cargo in Nix env"
            echo "- ./result/bin/causality-env               # Enter Nix shell"
          '';
        });
      }
    );
}
