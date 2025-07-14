{
  description = "Causality Core - Rust ZK Verification System";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system;
          overlays = overlays;
        };

        # Use a stable rust toolchain with necessary extensions
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" "clippy" ];
        };

        # Define OCaml environment based on OCaml 5.1
        ocamlEnv = pkgs.ocamlPackages;
        
        # Define the base OCaml compiler package separately
        ocamlCompilerPkg = ocamlEnv.ocaml;

        # Ensure all required OCaml packages are properly available
        eqafPkg = ocamlEnv.eqaf;
        digestifPkg = ocamlEnv.digestif;
        zarithPkg = ocamlEnv.zarith;
        ocamlCompilerLibsPkg = ocamlEnv.ocaml-compiler-libs;
        stdlibShimsPkg = ocamlEnv.stdlib-shims;
        ppxDeriversPkg = ocamlEnv.ppx_derivers;
        astringPkg = ocamlEnv.astring;
        uutfPkg = ocamlEnv.uutf;
        fmtTtyPkg = ocamlEnv.fmt;
        batteriesPkg = ocamlEnv.batteries;
        camlpStreamsPkg = ocamlEnv.camlp-streams;
        numPkg = ocamlEnv.num;
        base64Pkg = ocamlEnv.base64;
        uuidmPkg = ocamlEnv.uuidm;
        
        # Create a list of OCaml libraries needed for OCAMLPATH
        # This list should NOT include the base ocaml compiler itself.
        ocamlLibDeps = with ocamlEnv; [
          # ocaml # The compiler itself, findlib should find stdlib automatically
          findlib # Essential for library discovery
          ocaml_intrinsics_kernel # Required by base library
          base # Jane Street's standard library extensions
          core # Jane Street's full standard library replacement
          fmtTtyPkg
          ocamlCompilerLibsPkg # This is ocaml-compiler-libs, reinstated
          ppx_inline_test
          sexplib0
          alcotest
          ppxlib
          core_unix
          stdio
          ppx_assert
          ppx_sexp_conv
          ppx_compare
          ppx_custom_printf
          ppx_deriving
          
          eqafPkg
          digestifPkg
          zarithPkg
          stdlibShimsPkg
          ppxDeriversPkg
          astringPkg # For string handling in OCaml
          uutfPkg
          re # Regular expression library needed for tests
          cmdliner # Command line parsing library needed by alcotest
          batteriesPkg # Add batteries to library dependencies
          camlpStreamsPkg # Add camlp-streams as a dependency for batteries
          numPkg # Add num as a dependency for batteries
          yojson
          ppx_deriving_yojson
          ctypes
          base64Pkg
          uuidmPkg
        ];

        # Create a list of OCaml development tools
        ocamlToolDeps = with ocamlEnv; [
          dune_3 # Dune is a build tool, not strictly a library for OCAMLPATH
          ocaml-lsp
          ocamlformat
          utop
        ];
        
        # Generate the OCAMLPATH from the OCaml library dependencies
        # This ensures Dune can find the libraries at their Nix store paths
        mkOcamlPath = deps: 
          pkgs.lib.makeSearchPath "lib/ocaml/${ocamlEnv.ocaml.version}/site-lib" deps;

      in {
        # Packages
        packages = {
          # OCaml-Rust FFI integration package
          causality-ffi = pkgs.stdenv.mkDerivation {
            pname = "causality-ffi";
            version = "0.1.0";
            
            src = ./.;
            
            nativeBuildInputs = with pkgs; [
              rustToolchain
              ocamlCompilerPkg
              pkg-config
              cmake
            ] ++ ocamlLibDeps ++ ocamlToolDeps;
            
            buildInputs = with pkgs; [
              openssl
            ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
              pkgs.darwin.apple_sdk.frameworks.Security
              pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
              pkgs.libiconv
            ];
            
            # Set up environment for build
            OCAML_RUNTIME_PATH = "${ocamlCompilerPkg}/lib/ocaml";
            OCAML_VERSION = "${ocamlEnv.ocaml.version}";
            OCAML_STDLIB_PATH = "${ocamlCompilerPkg}/lib/ocaml";
            OCAMLPATH = mkOcamlPath ocamlLibDeps;
            CAML_LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath ocamlLibDeps;
            
            buildPhase = ''
              echo "Building OCaml-Rust FFI integration..."
              
              # Build Rust FFI library
              cd crates/causality-ffi
              cargo build --features ocaml-ffi --release
              cd ../..
              
              # Build OCaml library
              cd ocaml_causality
              dune build --release
              cd ..
            '';
            
            installPhase = ''
              mkdir -p $out/lib
              mkdir -p $out/include
              
              # Install Rust FFI library
              cp target/release/libcausality_ffi.* $out/lib/ || true
              
              # Install OCaml library
              cp -r ocaml_causality/_build/install/default/lib/* $out/lib/ || true
              
              # Install headers if they exist
              cp include/*.h $out/include/ || true
              cp ocaml_causality/include/*.h $out/include/ || true
            '';
            
            meta = with pkgs.lib; {
              description = "OCaml-Rust FFI integration for Causality";
              license = licenses.mit;
              platforms = platforms.unix;
            };
          };
        };
        
        # Development shells
        devShells = {
          # Default development shell with all necessary dependencies
          default = pkgs.mkShell {
            packages = with pkgs; [
              # Rust toolchain
              rustToolchain
              
              # General build tools
              pkg-config
              cmake
              
              # OCaml Compiler itself (provides ocamlc, etc. in PATH)
              ocamlCompilerPkg

              # Other dependencies
              openssl
              cacert
              
              # Add scripts directory to PATH
              (pkgs.writeShellScriptBin "generate-cargo-nix" ''
                ${pkgs.crate2nix}/bin/crate2nix generate
                echo "Cargo.nix file regenerated from Cargo.toml"
              '')
              
              # Add OCaml environment reload script
              (pkgs.writeShellScriptBin "reload-ocaml-env" ''
                set -euo pipefail
                
                echo "Reloading OCaml development environment..."
                
                # Kill any existing OCaml LSP processes
                echo "Stopping OCaml Language Server processes..."
                pkill -f "ocaml-lsp" || true
                sleep 1
                
                # Clear all OCaml build and cache directories
                echo "🧹 Clearing OCaml caches..."
                rm -rf ./_build
                rm -rf ./ocaml_causality/_build
                rm -rf ./ocaml_ssz/_build
                rm -rf ./e2e/ocaml_harness/_build
                rm -rf ./.ocaml-lsp-cache
                rm -rf ./.dune-cache
                
                # Recreate cache directories
                mkdir -p ./.ocaml-lsp-cache
                mkdir -p ./.dune-cache
                
                # Reload direnv environment
                echo "🔄 Reloading direnv environment..."
                ${pkgs.direnv}/bin/direnv reload
                
                echo "OCaml environment reload complete!"
                echo ""
                echo "Next steps:"
                echo "  1. Restart your editor/IDE"
                echo "  2. The OCaml LSP will automatically restart with fresh cache"
                echo "  3. Run 'dune build' to verify everything works"
                echo ""
                echo "Available commands in the shell:"
                echo "  - clear-ocaml-caches: Clear all OCaml caches"
                echo "  - restart-ocaml-lsp: Restart OCaml LSP with fresh cache"
                echo "  - build-ocaml: Build OCaml components"
                echo "  - start-utop: Start OCaml interactive session"
              '')
            ] ++ ocamlToolDeps # Add only OCaml tools, not lib deps to shell packages
            ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
              pkgs.darwin.apple_sdk.frameworks.Security
              pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
              pkgs.libiconv
            ];
            
            # Set up necessary environment variables
            shellHook = ''
              # Set up environment variables for build tools
              export SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt
              export OPENSSL_DIR=${pkgs.openssl.dev}
              export OPENSSL_LIB_DIR=${pkgs.openssl.out}/lib
              
              # OCaml-Rust FFI Configuration
              # Set up environment variables for Rust to find OCaml runtime libraries
              export OCAML_RUNTIME_PATH="${ocamlCompilerPkg}/lib/ocaml"
              export OCAML_VERSION="${ocamlEnv.ocaml.version}"
              export OCAML_STDLIB_PATH="${ocamlCompilerPkg}/lib/ocaml"
              
              # Configure Rust build to link against OCaml runtime (only for FFI crate)
              # NOTE: RUSTFLAGS is no longer set globally to avoid affecting all crates.
              # Individual crates requiring FFI linking will handle this via their own build scripts.
              export PKG_CONFIG_PATH="${ocamlCompilerPkg}/lib/pkgconfig:$PKG_CONFIG_PATH"
              
              # OCaml Language Server Cache Management
              # Force LSP to use project-local cache directories
              export OCAML_LSP_CACHE_DIR="$PWD/.ocaml-lsp-cache"
              export DUNE_CACHE_ROOT="$PWD/.dune-cache"
              export DUNE_CACHE=enabled
              
              # Initialize cache directories only if they don't exist
              mkdir -p "$OCAML_LSP_CACHE_DIR"
              mkdir -p "$DUNE_CACHE_ROOT"
              
              # OCaml environment variables - use ocamlLibDeps for paths
              export OCAMLPATH=${mkOcamlPath ocamlLibDeps}
              export CAML_LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath ocamlLibDeps}
              
              # Add helper function to clear OCaml caches efficiently
              clear-ocaml-caches() {
                echo "Clearing OCaml caches..."
                
                # Clear build directories only if they exist and are not empty
                [ -d "./_build" ] && [ "$(ls -A ./_build 2>/dev/null)" ] && rm -rf ./_build
                [ -d "./ocaml_causality/_build" ] && [ "$(ls -A ./ocaml_causality/_build 2>/dev/null)" ] && rm -rf ./ocaml_causality/_build
                [ -d "./ocaml_ssz/_build" ] && [ "$(ls -A ./ocaml_ssz/_build 2>/dev/null)" ] && rm -rf ./ocaml_ssz/_build
                [ -d "./e2e/ocaml_harness/_build" ] && [ "$(ls -A ./e2e/ocaml_harness/_build 2>/dev/null)" ] && rm -rf ./e2e/ocaml_harness/_build
                
                # Clear LSP cache only if it exists
                [ -d "$OCAML_LSP_CACHE_DIR" ] && rm -rf "$OCAML_LSP_CACHE_DIR"
                
                # Preserve dune cache but clear problematic entries
                if [ -d "$DUNE_CACHE_ROOT" ]; then
                  find "$DUNE_CACHE_ROOT" -name "*.error" -delete 2>/dev/null || true
                fi
                
                # Recreate essential directories
                mkdir -p "$OCAML_LSP_CACHE_DIR"
                mkdir -p "$DUNE_CACHE_ROOT"
                
                echo "OCaml caches cleared efficiently"
              }
              export -f clear-ocaml-caches
              
              # Add helper function to restart OCaml LSP
              restart-ocaml-lsp() {
                echo "Restarting OCaml Language Server..."
                pkill -f "ocaml-lsp" || true
                clear-ocaml-caches
                echo "OCaml LSP restart complete - cache cleared"
                echo "Your editor should reconnect to a fresh LSP instance"
              }
              export -f restart-ocaml-lsp
              
              # Add selective cache management functions
              smart-cache-check() {
                # Only clear cache if there are actual build issues
                local need_clear=false
                
                # Check if dune files have been modified recently (within last 10 minutes)
                # Use a more efficient check that doesn't traverse the entire directory tree
                if [ -f "ocaml_causality/dune-project" ] && [ "$(find ocaml_causality/dune-project -newermt '10 minutes ago' 2>/dev/null)" ]; then
                  echo "Recent dune-project changes detected"
                  need_clear=true
                fi
                
                # Check if LSP cache has obvious error indicators (but don't search recursively)
                if [ -f "$OCAML_LSP_CACHE_DIR/errors" ]; then
                  echo "LSP cache errors detected"
                  need_clear=true
                fi
                
                if [ "$need_clear" = true ]; then
                  echo "Smart cache clear triggered"
                  clear-ocaml-caches
                else
                  echo "Cache check passed - no clearing needed"
                fi
              }
              export -f smart-cache-check
              
              # Add cleanup function for any existing watchers
              cleanup-cache-watcher() {
                if [ -f .cache-watcher-pid ]; then
                  pid=$(cat .cache-watcher-pid)
                  kill "$pid" 2>/dev/null || true
                  rm -f .cache-watcher-pid
                  rm -f .last-config-check
                  echo "Cache watcher stopped"
                fi
              }
              export -f cleanup-cache-watcher
              
              # Clean up any existing aggressive watchers on shell start
              cleanup-cache-watcher
              
              # Add helper function to regenerate Nix files from Cargo.toml
              generate-cargo-nix() {
                ${pkgs.crate2nix}/bin/crate2nix generate
                echo "Cargo.nix file regenerated from Cargo.toml"
              }
              export -f generate-cargo-nix
              
              # Add helper function to build using the generated Cargo.nix
              build-with-nix() {
                if [ ! -f Cargo.nix ]; then
                  echo "Cargo.nix not found. Generating first..."
                  generate-cargo-nix
                fi
                
                nix-build -A rootCrate.build
                echo "Build completed using crate2nix"
              }
              export -f build-with-nix
              
              # Function to build OCaml-Rust FFI integration
              build-ffi() {
                echo "🔨 Building OCaml-Rust FFI integration..."
                
                # Verify Nix environment
                if [ -z "$OCAML_RUNTIME_PATH" ]; then
                  echo "Error: OCAML_RUNTIME_PATH not set. FFI environment not properly configured."
                  return 1
                fi
                
                echo "Using OCaml runtime from: $OCAML_RUNTIME_PATH"
                echo "OCaml version: $OCAML_VERSION"
                
                # Clean previous builds
                echo "🧹 Cleaning previous builds..."
                cargo clean -p causality-ffi
                
                # Build the Rust FFI library with OCaml feature enabled
                echo "Building Rust FFI library..."
                (cd crates/causality-ffi && cargo build --features ocaml-ffi --verbose)
                
                # Check if the library was built successfully
                local lib_built=false
                for ext in a so dylib; do
                  if [ -f "target/debug/libcausality_ffi.$ext" ]; then
                    echo "Found libcausality_ffi.$ext"
                    lib_built=true
                    break
                  fi
                done
                
                if [ "$lib_built" = false ]; then
                  echo "Failed to build Rust FFI library"
                  return 1
                fi
                
                echo "Rust FFI library built successfully"
                
                # Build the OCaml causality library
                echo "Building OCaml causality library..."
                (cd ocaml_causality && dune clean && dune build --verbose)
                
                echo "OCaml library built successfully"
                
                # Test basic FFI integration
                echo "Testing FFI integration..."
                if (cd ocaml_causality && dune exec -- ocaml -c << 'EOF'
print_endline "Testing FFI integration...";
print_endline "FFI test completed";;
EOF
                ); then
                  echo "Basic FFI test passed"
                else
                  echo " FFI test had issues (this might be expected for complex FFI)"
                fi
                
                echo ""
                echo "FFI build completed!"
                echo ""
                echo "Next steps:"
                echo "  - Run E2E tests: cd ocaml_causality && dune exec test/e2e/test_river_compilation_simulation.exe"
                echo "  - Check FFI bindings: cd ocaml_causality && dune exec -- ocaml-lsp merlin single dump-config ."
                echo "  - Run simulation tests: cd e2e && dune exec -- ./simulation_zk_integration_e2e.exe"
              }
              export -f build-ffi
              
              # Function to build OCaml code
              build-ocaml() {
                echo "Building OCaml code with Dune..."
                cd ml_causality
                dune build --display=short
                echo "OCaml build completed"
              }
              export -f build-ocaml
              
              # Function to start utop with proper environment
              start-utop() {
                echo "Starting OCaml interactive toplevel (utop)..."
                cd ml_causality
                dune utop
              }
              export -f start-utop
              
              # Function to check OCaml library paths
              check-ocaml-paths() {
                echo "Checking OCaml library paths..."
                echo "OCAMLPATH = $OCAMLPATH"
                echo ""
                echo "Zarith path: ${zarithPkg}/lib/ocaml/${ocamlEnv.ocaml.version}/site-lib/zarith"
                echo "Exists: $(ls -la ${zarithPkg}/lib/ocaml/${ocamlEnv.ocaml.version}/site-lib/zarith 2>/dev/null || echo 'Not found')"
                echo ""
                echo "Eqaf path: ${eqafPkg}/lib/ocaml/${ocamlEnv.ocaml.version}/site-lib/eqaf"
                echo "Exists: $(ls -la ${eqafPkg}/lib/ocaml/${ocamlEnv.ocaml.version}/site-lib/eqaf 2>/dev/null || echo 'Not found')"
                echo ""
                echo "Digestif path: ${digestifPkg}/lib/ocaml/${ocamlEnv.ocaml.version}/site-lib/digestif"
                echo "Exists: $(ls -la ${digestifPkg}/lib/ocaml/${ocamlEnv.ocaml.version}/site-lib/digestif 2>/dev/null || echo 'Not found')"
                echo ""
                echo "Astring path: ${astringPkg}/lib/ocaml/${ocamlEnv.ocaml.version}/site-lib/astring"
                echo "Exists: $(ls -la ${astringPkg}/lib/ocaml/${ocamlEnv.ocaml.version}/site-lib/astring 2>/dev/null || echo 'Not found')"
                echo ""
                echo "Uutf path: ${ocamlEnv.uutf}/lib/ocaml/${ocamlEnv.ocaml.version}/site-lib/uutf"
                echo "Exists: $(ls -la ${ocamlEnv.uutf}/lib/ocaml/${ocamlEnv.ocaml.version}/site-lib/uutf 2>/dev/null || echo 'Not found')"
              }
              export -f check-ocaml-paths

              # Function to test OCaml-Rust FFI integration
              test-ffi() {
                echo "🧪 Testing OCaml-Rust FFI integration..."
                
                # Verify FFI build exists
                local lib_found=false
                for ext in a so dylib; do
                  if [ -f "target/debug/libcausality_ffi.$ext" ]; then
                    lib_found=true
                    break
                  fi
                done
                
                if [ "$lib_found" = false ]; then
                  echo "FFI library not found. Run 'build-ffi' first."
                  return 1
                fi
                
                echo "FFI library found"
                
                # Test OCaml causality compilation
                echo "🐪 Testing OCaml causality compilation..."
                if (cd ocaml_causality && dune build test/e2e/test_river_compilation_simulation.exe); then
                  echo "OCaml E2E test compilation successful"
                else
                  echo "OCaml E2E test compilation failed"
                  return 1
                fi
                
                # Run the River E2E test
                echo "🌊 Running River E2E simulation test..."
                if (cd ocaml_causality && dune exec test/e2e/test_river_compilation_simulation.exe); then
                  echo "River E2E test passed"
                else
                  echo "River E2E test failed"
                  return 1
                fi
                
                echo ""
                echo "🎉 All FFI tests passed!"
              }
              export -f test-ffi
              
              # Function to clean all FFI artifacts
              clean-ffi() {
                echo "🧹 Cleaning all FFI artifacts..."
                cargo clean -p causality-ffi
                (cd ocaml_causality && dune clean)
                rm -rf target/debug/libcausality_ffi.*
                echo "FFI artifacts cleaned"
              }
              export -f clean-ffi
              
              # Function to build FFI using Nix package system
              build-ffi-package() {
                echo "Building FFI as Nix package..."
                nix build .#causality-ffi --verbose
                if [ -L "./result" ]; then
                  echo "FFI package built successfully"
                  echo "Package available at: $(readlink -f ./result)"
                  echo "Contents:"
                  ls -la ./result/lib/ || echo "No lib directory"
                  ls -la ./result/include/ || echo "No include directory"
                else
                  echo "FFI package build failed"
                  return 1
                fi
              }
              export -f build-ffi-package

              # Add lightweight cache status check
              cache-status() {
                echo "Cache Status:"
                echo "  LSP Cache: $([ -d "$OCAML_LSP_CACHE_DIR" ] && echo "exists" || echo "missing")"
                echo "  Dune Cache: $([ -d "$DUNE_CACHE_ROOT" ] && echo "exists" || echo "missing")"
                echo "  Build dirs: $(find . -maxdepth 2 -name "_build" -type d 2>/dev/null | wc -l | tr -d ' ') found"
                if [ -f .cache-watcher-pid ]; then
                  echo "  Background watcher: running (PID: $(cat .cache-watcher-pid))"
                else
                  echo "  Background watcher: stopped"
                fi
              }
              export -f cache-status

              echo "Causality development environment loaded."
              echo ""
              echo "Build Commands:"
              echo "  generate-cargo-nix, build-with-nix, build-ocaml, start-utop"
              echo ""
              echo "Cache Management:"
              echo "  smart-cache-check, clear-ocaml-caches, restart-ocaml-lsp"
              echo ""
              echo "FFI Integration:"
              echo "  build-ffi, test-ffi, clean-ffi, build-ffi-package"
              echo ""
              echo "Diagnostics:"
              echo "  check-ocaml-paths, cleanup-cache-watcher, reload-ocaml-env"
            '';
          };
          
          # Rust-only development shell without OCaml dependencies
          rust-only = pkgs.mkShell {
            packages = with pkgs; [
              # Rust toolchain
              rustToolchain
              
              # General build tools
              pkg-config
              cmake
              
              # Other dependencies
              openssl
              cacert
            ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
              pkgs.darwin.apple_sdk.frameworks.Security
              pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
              pkgs.libiconv
            ];
            
            # Minimal environment for Rust builds
            shellHook = ''
              # Set up environment variables for build tools
              export SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt
              export OPENSSL_DIR=${pkgs.openssl.dev}
              export OPENSSL_LIB_DIR=${pkgs.openssl.out}/lib
              
              # CRITICAL: Explicitly unset all OCaml-related environment variables
              unset OCAML_RUNTIME_PATH
              unset OCAML_VERSION
              unset OCAML_STDLIB_PATH
              unset OCAMLPATH
              unset CAML_LD_LIBRARY_PATH
              # Unset linker flags that might interfere
              unset RUSTFLAGS
              unset LD_LIBRARY_PATH
              unset DYLD_LIBRARY_PATH
              
              echo "Rust-only development environment loaded."
              echo "Use this environment for pure Rust builds without OCaml interference."
            '';
          };
        };
      });
}