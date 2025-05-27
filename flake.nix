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
        ocamlEnv = pkgs.ocaml-ng.ocamlPackages_5_1;
        
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
        
        # Create a list of OCaml libraries needed for OCAMLPATH
        # This list should NOT include the base ocaml compiler itself.
        ocamlLibDeps = with ocamlEnv; [
          # ocaml # The compiler itself, findlib should find stdlib automatically
          findlib # Essential for library discovery
          fmt
          ocamlCompilerLibsPkg # This is ocaml-compiler-libs, reinstated
          ppx_inline_test
          sexplib0
          alcotest
          ppxlib
          core
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
        # Development shell with all necessary dependencies
        devShells.default = pkgs.mkShell {
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
          ] ++ ocamlLibDeps ++ ocamlToolDeps # Add lib and tool deps to shell packages
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

            # --- Debugging OCAMLPATH Construction ---
            echo "---- START OCAMLPATH DEBUG ----"
            echo "DEBUG: Value of uutfPkg (raw): ${uutfPkg}"
            echo "DEBUG: uutfPkg.outPath: ${uutfPkg.outPath}"
            echo "DEBUG: mkOcamlPath for uutfPkg only: ${mkOcamlPath [ uutfPkg ]}"
            echo "DEBUG: Value of astringPkg (raw): ${astringPkg}"
            echo "DEBUG: astringPkg.outPath: ${astringPkg.outPath}"
            echo "DEBUG: mkOcamlPath for astringPkg only: ${mkOcamlPath [ astringPkg ]}"
            export OCAMLPATH_BEFORE_ASSIGN="${mkOcamlPath ocamlLibDeps}"
            echo "DEBUG: Full OCAMLPATH (before assignment to env var): $OCAMLPATH_BEFORE_ASSIGN"
            echo "---- END OCAMLPATH DEBUG ----"
            
            # OCaml environment variables - use ocamlLibDeps for paths
            export OCAMLPATH=${mkOcamlPath ocamlLibDeps}
            export CAML_LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath ocamlLibDeps}
            
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
              echo "Uutf path: ${uutfPkg}/lib/ocaml/${ocamlEnv.ocaml.version}/site-lib/uutf"
              echo "Exists: $(ls -la ${uutfPkg}/lib/ocaml/${ocamlEnv.ocaml.version}/site-lib/uutf 2>/dev/null || echo 'Not found')"
            }
            export -f check-ocaml-paths

            echo "Nix development shell loaded."
            echo "Run 'generate-cargo-nix' to update Rust dependencies."
            echo "Run 'build-with-nix' to build Rust components."
            echo "Run 'build-ocaml' to build OCaml components."
            echo "Run 'start-utop' for OCaml interactive session."
            echo "Run 'check-ocaml-paths' for OCaml library path diagnostics."
          '';
        };
      });
}