<!-- Nix environment for development -->
<!-- Original file: docs/src/nix_environment.md -->

# Using the Nix Environment for Causality

This document explains how to use the Nix environment for Causality development and building.

## Why Use the Nix Environment?

Using the Nix environment ensures:

1. All developers have the exact same build environment
2. All dependencies are correctly pinned and isolated
3. `MACOSX_DEPLOYMENT_TARGET` is properly set for macOS builds
4. Reproducible builds between systems and CI

## Setting Up the Environment

There are two main ways to use the Nix environment:

### Option 1: Using the causality-env Script (Recommended)

Run the setup script to create the environment wrapper:

```bash
./scripts/setup-env.sh
```

This will create a `causality-env` script in the project root. Now you can:

- **Enter the Nix shell**:
  ```bash
  ./causality-env
  ```

- **Run a command in the Nix environment**:
  ```bash
  ./causality-env cargo build
  ./causality-env cargo test
  ./causality-env cargo check --no-default-features --features minimal-build
  ```

### Option 2: Using direnv (Most Convenient)

If you have [direnv](https://direnv.net/) installed:

1. Allow the .envrc file:
   ```bash
   direnv allow
   ```

2. The environment will be automatically loaded when you cd into the project.

## Building with Nix

To build the project with Nix directly:

```bash
# Using crate2nix (default)
nix build

# Using standard Rust build
nix build .#causality-std
```

## Environment Variables

The following environment variables are automatically set in the Nix environment:

- `MACOSX_DEPLOYMENT_TARGET`: Set to `11.0` on macOS systems to ensure compatibility

## Generating Cargo.nix

The Cargo.nix file maps Rust dependencies to Nix:

```bash
nix run .#generate-cargo-nix
```

## Troubleshooting

If you encounter issues with the build:

1. Ensure you're using the Nix environment:
   ```bash
   ./causality-env cargo build
   ```

2. Check that MACOSX_DEPLOYMENT_TARGET is set:
   ```bash
   ./causality-env printenv MACOSX_DEPLOYMENT_TARGET
   ```

3. Try rebuilding the environment script:
   ```bash
   ./scripts/setup-env.sh
   ``` 