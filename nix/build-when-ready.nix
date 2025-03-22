# Example of how to use crate2nix for the Causality project
# This file can be used as a reference when the codebase is ready to build.
{ nixpkgs ? <nixpkgs>
, system ? builtins.currentSystem
, pkgs ? import nixpkgs { inherit system; }
}:

let
  # Import crate2nix either from flake inputs or directly
  crate2nix = import (pkgs.fetchFromGitHub {
    owner = "nix-community";
    repo = "crate2nix";
    rev = "0.14.1";
    sha256 = "sha256-nSQ0L2NyqrlD2z2xaIbUEzr8ggGHKDXnuq0mQfUFGyM=";
  }) {};

  # Set macOS deployment target if on Darwin
  darwinConfig = pkgs.lib.optionalAttrs pkgs.stdenv.isDarwin {
    MACOSX_DEPLOYMENT_TARGET = "10.13";
  };

  # Create custom crate overrides
  customCrateOverrides = pkgs.defaultCrateOverrides // {
    # Add openssl override
    openssl-sys = attrs: {
      buildInputs = (attrs.buildInputs or []) ++ [ 
        pkgs.openssl.dev 
        pkgs.pkg-config
      ];
    };
    
    # Add other crate-specific overrides as needed
    ethers = attrs: {
      buildInputs = (attrs.buildInputs or []) ++ [
        pkgs.pkg-config
      ];
    };
    
    # Override for the root causality crate to set environment variables
    causality = attrs: {
      buildInputs = (attrs.buildInputs or []) ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
        pkgs.darwin.apple_sdk.frameworks.Security
        pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
      ];
      
      # Set environment variables during the build
      inherit (darwinConfig) MACOSX_DEPLOYMENT_TARGET;
    };
  };

  # Build the project from the generated Cargo.nix
  causality = 
    let 
      # Import the generated Cargo.nix
      cargo-nix = pkgs.callPackage ../Cargo.nix {
        inherit pkgs;
        defaultCrateOverrides = customCrateOverrides;
      };
    in
      # Build the root crate
      cargo-nix.rootCrate.build;

in {
  # The final package
  inherit causality;
  
  # Also expose the cargo-nix build for debugging
  cargo-nix = cargo-nix;
  
  # Shell for development
  devShell = pkgs.mkShell ({
    buildInputs = with pkgs; [
      cargo
      rustc
      rustfmt
      clippy
      pkg-config
      openssl
    ] ++ lib.optionals pkgs.stdenv.isDarwin [
      darwin.apple_sdk.frameworks.Security
      darwin.apple_sdk.frameworks.SystemConfiguration
    ];
  } // darwinConfig);
} 