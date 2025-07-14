#!/bin/bash
# Clean Rust build script that unsets all OCaml environment variables

# Save the current PATH
SAVED_PATH="$PATH"

# Unset all OCaml-related environment variables
unset RUSTFLAGS
unset OCAML_RUNTIME_PATH
unset OCAML_VERSION  
unset OCAML_STDLIB_PATH
unset OCAMLPATH
unset CAML_LD_LIBRARY_PATH
unset LD_LIBRARY_PATH
unset DYLD_LIBRARY_PATH
unset OCAML_LSP_CACHE_DIR
unset DUNE_CACHE_ROOT
unset DUNE_CACHE

# Restore PATH
export PATH="$SAVED_PATH"

echo "🧹 Cleaned OCaml environment variables"
echo "🦀 Running clean Rust build..."
echo "RUSTFLAGS: ${RUSTFLAGS:-<unset>}"

# Run the cargo build with clean environment
exec cargo "$@"
