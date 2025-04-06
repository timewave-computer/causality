# crates/agent-user/flake.nix
{
  description = "User Agent for Causality Simulation";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    # Reference the parent flake for workspace dependencies
    causality = { url = "git+file://../../?"; flake = true; }; 
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, causality }:
    flake-utils.lib.eachDefaultSystem (system: let
      overlays = [ (import rust-overlay) ];
      pkgs = import nixpkgs { inherit system overlays; };
      rustPlatform = pkgs.makeRustPlatform {
        cargo = pkgs.rust-bin.stable."1.79.0".default.override { 
            extensions = [ "rust-src" ]; 
            targets = ["aarch64-apple-darwin"];
        };
        # Inherit build inputs from the workspace flake devShell
        nativeBuildInputs = causality.devShells.${system}.default.nativeBuildInputs;
      };
    in {
      packages.default = rustPlatform.buildRustPackage {
        pname = "causality-agent-user";
        version = "0.1.0";

        src = ./.; # Source is the current directory

        cargoLock.lockFile = ../../Cargo.lock; # Use workspace lock file

        # Use build inputs from the workspace devShell
        buildInputs = causality.devShells.${system}.default.buildInputs;
        nativeBuildInputs = causality.devShells.${system}.default.nativeBuildInputs;

        # If specific build steps are needed, add them here
        # buildPhase = ''
        #   cargo build --release --target ${pkgs.stdenv.hostPlatform.config}
        # ''
      };

      # Define the app to run the agent
      apps.default = {
        type = "app";
        program = "${self.packages.${system}.default}/bin/agent-user";
      };
    });
} 