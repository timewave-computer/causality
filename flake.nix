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
    # TEL extension subflake
    tel-extension = {
      url = "path:./nix/tel-syntax-extension";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };
    # Add inputs for the agent flakes (needed if we reference their outputs, though crate2nix might handle it)
    # agent-user-flake = { url = "git+file://./crates/agent-user?"; flake = true; };
    # agent-committee-flake = { url = "git+file://./crates/agent-committee?"; flake = true; };
  };

  outputs = { self, nixpkgs, flake-utils, crate2nix, rust-overlay, tel-extension, /* agent-user-flake, agent-committee-flake */ }:
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
          rocksdb
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
          # Adjust features if needed to ensure binaries are built
          # Maybe define specific binary crates if rootFeatures isn't enough?
          # For now, assume default build includes binaries from workspace members.
          # rootFeatures = ["minimal-build"]; 
          defaultCrateOverrides = pkgs.defaultCrateOverrides // {
            openssl-sys = attrs: {
              buildInputs = (attrs.buildInputs or []) ++ [ pkgs.openssl ];
              nativeBuildInputs = (attrs.nativeBuildInputs or []) ++ [ pkgs.pkg-config ];
              inherit (commonEnv) MACOSX_DEPLOYMENT_TARGET;
            };
            proc-macro-crate = attrs: {
              patches = [
                ./nix/patches/proc-macro-crate.patch
              ];
              inherit (commonEnv) MACOSX_DEPLOYMENT_TARGET;
            };
            ".*" = attrs: {
              inherit (commonEnv) MACOSX_DEPLOYMENT_TARGET;
            };
          };
        };

        # Define helper to get binary package from crate2nix result
        getBinPkg = name: crate2nixPackage.workspaceMembers."${name}".build;

      in rec {
        # Default package (crate2nix build)
        packages = {
          causality-controller = 
            if builtins.hasAttr "causality-controller" crate2nixPackage.workspaceMembers
            then getBinPkg "causality-controller"
            else pkgs.hello; # Fallback to a placeholder

          agent-user = 
            if builtins.hasAttr "agent-user" crate2nixPackage.workspaceMembers
            then getBinPkg "agent-user"
            else pkgs.hello; # Fallback to a placeholder

          agent-committee = 
            if builtins.hasAttr "agent-committee" crate2nixPackage.workspaceMembers
            then getBinPkg "agent-committee"
            else pkgs.hello; # Fallback to a placeholder

          env-script = nixEnvScript;
          
          # Keep default pointing to controller for now, or remove if confusing
          default = packages.env-script; 
        };

        # Apps for different build tasks 
        apps = {
          controller = {
            type = "app";
            program = "${packages.causality-controller}/bin/causality-controller";
          };
          agent-user = {
            type = "app";
            program = "${packages.agent-user}/bin/agent-user";
          };
          agent-committee = {
            type = "app";
            program = "${packages.agent-committee}/bin/agent-committee";
          };
          generate-cargo-nix = {
            type = "app";
            program = toString (pkgs.writeShellScript "generate-cargo-nix" ''
              export MACOSX_DEPLOYMENT_TARGET="${darwinDeploymentTarget}"
              cd ${self}
              ${crate2nixPkg}/bin/crate2nix generate
              echo "Cargo.nix generated successfully"
            '');
          };
          build = {
            type = "app";
            program = toString (pkgs.writeShellScript "build-causality" ''
              export MACOSX_DEPLOYMENT_TARGET="${darwinDeploymentTarget}"
              cd ${self}
              ${rustVersion}/bin/cargo build
              echo "Build completed successfully"
            '');
          };
          
          # Import the TEL extension app
          package-tel-extension = tel-extension.apps.${system}.tel-extension;
          
          # Set the default app to env
          default = apps.env; 
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
            # Don't include the built agent binaries as they might not exist yet
            # Add Node.js tools for VSCode extension development
            pkgs.nodejs
            (pkgs.nodePackages.typescript-language-server or pkgs.nodePackages.nodejs)
          ];
          
          # Explicitly set environment variables
          inherit (commonEnv) MACOSX_DEPLOYMENT_TARGET;
          
          # Add npm config to allow global installs without sudo
          NPM_CONFIG_PREFIX = "$HOME/.npm-global";
          
          shellHook = ''
            echo "Rust development environment for Causality loaded"
            echo "MACOSX_DEPLOYMENT_TARGET set to $MACOSX_DEPLOYMENT_TARGET"
            echo ""
            echo "Build commands:"
            echo "- nix run .#generate-cargo-nix  # Generate Cargo.nix file"
            echo "- nix build .#controller         # Build controller (default)"
            echo "- nix build .#agent-user         # Build user agent"
            echo "- nix build .#agent-committee    # Build committee agent"
            echo ""
            echo "Run commands:"
            echo "- nix run .#controller -- --help  # Run controller app"
            echo "- nix run .#agent-user -- --help   # Run user agent app"
            echo ""
            echo "Development tools:"
            echo "- nix run .#package-tel-extension  # Package TEL syntax highlighting for Cursor"
            echo ""
            echo "Run other commands in the Nix environment with:"
            echo "- ./result/bin/causality-env cargo build   # Run cargo in Nix env"
            echo "- ./result/bin/causality-env               # Enter Nix shell"
            
            # Set up npm global path
            mkdir -p $HOME/.npm-global/bin
            export PATH=$HOME/.npm-global/bin:$PATH
            
            # Check for vsce and install if needed
            if ! command -v vsce >/dev/null 2>&1; then
              echo "Installing vsce for VSCode extension development..."
              npm install -g @vscode/vsce
            fi
          '';
        });
      }
    );
}
