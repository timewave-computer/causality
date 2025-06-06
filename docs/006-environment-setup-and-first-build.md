# 006: Environment Setup and First Build

Welcome to Causality! Setting up a consistent and reproducible development environment is the first crucial step to exploring and contributing to the framework. This guide will walk you through the process using Nix, which manages all dependencies for both Rust and OCaml components, ensuring you have everything you need.

## Step 1: Clone the Repository

First, clone the Causality project repository to your local machine:

```bash
git clone <repository-url> # Replace <repository-url> with the actual URL of your Git repository
cd causality # Or your project's root directory name
```

## Step 2: Enter the Nix Development Environment

Causality uses a Nix flake (`flake.nix` at the root of the project) to define a development shell that includes all necessary compilers, tools, and dependencies (like Rust, Cargo, OCaml, Dune, `crate2nix` outputs, etc.).

To activate this environment, navigate to the project's root directory and run:

```bash
nix develop
```

This command might take some time on the first run as Nix downloads and builds all the specified dependencies. Subsequent runs will be much faster. Once the command completes, your shell prompt will likely change, indicating you are now inside the Nix development environment, equipped with all the tools needed for Causality development.

## Step 3: Perform an Initial Build (Rust Components)

With the development environment active, you can now build the Rust components of the Causality framework using Cargo, Rust's package manager and build system.

```bash
cargo build
```

This command will compile all Rust crates within the workspace. The first build may take some time, but subsequent builds will be incremental and faster.

## Step 4: Run Initial Tests (Rust Components)

To verify that the Rust components are set up correctly and functioning as expected, run the test suite. You can run tests for the entire workspace or for specific packages critical to the architecture:

```bash
# Run all tests in the workspace (recommended for a first verification)
cargo test

# Alternatively, run tests for key individual packages:
cargo test -p causality-types          # Foundational types for all layers
cargo test -p causality-vm             # Layer 0: Typed Register Machine
cargo test -p causality-lisp-ast       # Layer 1: Lisp AST and primitives
cargo test -p causality-lisp-compiler  # Layer 1: Lisp to Layer 0 compiler
cargo test -p causality-effects-engine # Layer 2: Effects, Intents, TEG
```

Successful completion of these tests indicates that your Rust environment for Causality is correctly configured.

## Step 5: Build and Test OCaml Components (If Applicable)

If your project includes OCaml components (e.g., `ml_causality`), the Nix development environment also provides the necessary OCaml compiler and Dune build system.

Navigate to the OCaml part of your project (e.g., `ml_causality` directory) and use Dune to build and test:

```bash
cd ml_causality # Or the relevant directory for OCaml components
dune build
dune runtest # Or dune test, depending on your project's test invocation
cd .. # Return to the project root
```

## Conclusion

Congratulations! You have successfully set up your development environment for Causality, performed initial builds, and verified the setup with tests. You are now ready to dive deeper into the framework, explore its capabilities, and start building your own linear resource applications.