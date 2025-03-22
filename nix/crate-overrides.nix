{ pkgs }:

# This file contains overrides for specific Rust crates to handle native dependencies
# or other compilation requirements.

let
  # Create a base set of overrides using the pkgs.defaultCrateOverrides
  baseOverrides = pkgs.defaultCrateOverrides or {};
in
baseOverrides // {
  # Add overrides for crates with native dependencies here
  # Example:
  # openssl-sys = attrs: {
  #   buildInputs = with pkgs; [ openssl.dev pkg-config ];
  # };
  
  # Override for openssl-sys with correct dependencies
  openssl-sys = attrs: {
    buildInputs = (attrs.buildInputs or []) ++ [ 
      pkgs.openssl.dev 
      pkgs.pkg-config
    ];
  };
  
  # Override for ethereum crates that might need special handling
  ethers = attrs: {
    buildInputs = (attrs.buildInputs or []) ++ [
      pkgs.pkg-config
    ];
  };
} 