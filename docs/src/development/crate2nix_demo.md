<!-- Demo for crate2nix -->
<!-- Original file: docs/src/crate2nix_demo.md -->

# Using crate2nix with Causality

This document provides a step-by-step guide for using crate2nix to build the Causality project with Nix, once the codebase is stable and ready for this approach.

## Why crate2nix?

crate2nix offers several advantages over standard Nix Rust builds:

1. **Granular rebuilds**: Only rebuilds crates that have changed, saving CI time
2. **Precise dependency control**: Each Rust crate becomes a separate Nix derivation
3. **Customization per crate**: Override build settings for specific dependencies
4. **Better caching**: Leverage Nix's binary cache at the crate level

## Setup Instructions

### 1. Generate the Cargo.nix file

The first step is to generate a `Cargo.nix` file from your Cargo.toml:

```bash
# Using the flake app
nix run .#generate-cargo-nix

# Or directly if in the nix shell
crate2nix generate
```

This creates a `Cargo.nix` file in the project root that maps your Rust dependencies to Nix derivations.

### 2. Build the project

Once the `Cargo.nix` file exists, you can build the project:

```bash
# Using the flake
nix build .#causality-crate2nix

# Or using the template file
nix-build -A causality nix/build-when-ready.nix
```

## Understanding the Build Process

With crate2nix, the build process works as follows:

1. The `Cargo.nix` file parses your dependency tree from Cargo.toml and Cargo.lock
2. Each crate becomes a separate Nix derivation with its own build process
3. Nix's content-addressed store ensures identical crates are built only once
4. Custom overrides can be applied to specific crates as needed

### Example: Tracing a crate build

When building with crate2nix, you'll see output like:

```
building crate async-trait-0.1.68
building crate libc-0.2.147
building crate futures-core-0.3.28
...
building crate causality-0.1.0
```

Each crate is built separately, and if a crate hasn't changed since the last build, it will be fetched from the Nix store instead of being rebuilt.

## Customizing Builds

### Crate Overrides

You can customize how specific crates are built by creating overrides:

```nix
customCrateOverrides = pkgs.defaultCrateOverrides // {
  # Override for openssl-sys to add native dependencies
  openssl-sys = attrs: {
    buildInputs = (attrs.buildInputs or []) ++ [ 
      pkgs.openssl.dev 
      pkgs.pkg-config
    ];
  };
  
  # Override for the root crate
  causality = attrs: {
    buildInputs = (attrs.buildInputs or []) ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
      pkgs.darwin.apple_sdk.frameworks.Security
    ];
  };
};
```

### Feature Selection

You can control which features are enabled for the root crate:

```nix
rootFeatures = [ "default" "extra-feature" ];
```

## Troubleshooting

### Common Issues

1. **MACOSX_DEPLOYMENT_TARGET errors**: Set this environment variable if building on macOS:
   ```nix
   darwinEnv = pkgs.lib.optionalAttrs pkgs.stdenv.isDarwin {
     MACOSX_DEPLOYMENT_TARGET = "10.13";
   };
   ```

2. **Missing native dependencies**: Add them to the appropriate crate override:
   ```nix
   some-crate = attrs: {
     buildInputs = (attrs.buildInputs or []) ++ [ pkgs.some-dependency ];
   };
   ```

3. **Version conflicts**: These are usually resolved by ensuring your Cargo.lock is up to date:
   ```bash
   cargo update
   ```

## Real-World CI Optimization

In CI, crate2nix can significantly speed up builds because:

1. Only changed crates are rebuilt
2. Results can be cached in a Nix binary cache
3. Caching is per-crate, not a monolithic build

This means if you change one file in your project, only the affected crates need to be rebuilt, not the entire project.

## Future Work

Once the Causality codebase is stable and all compilation errors are resolved, we plan to:

1. Fully integrate crate2nix into the CI pipeline
2. Set up binary caching for faster CI and developer builds
3. Create targeted crate overrides for any problematic dependencies
4. Optimize build parameters for production releases 