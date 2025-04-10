{
  description = "TEL Syntax Highlighting Extension for VSCode/Cursor";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in rec {
        # App to package the extension
        apps.tel-extension = {
          type = "app";
          program = toString (pkgs.writeShellScript "package-tel-extension" ''
            # Find the tel-syntax-highlight directory relative to the current repo
            REPO_ROOT=$(git rev-parse --show-toplevel 2>/dev/null || echo "/Users/hxrts/projects/timewave/causality")
            TEL_DIR="$REPO_ROOT/tel-syntax-highlight"
            
            # Check if the directory exists
            if [ ! -d "$TEL_DIR" ]; then
              echo "Error: TEL syntax highlight directory not found at $TEL_DIR"
              exit 1
            fi
            
            echo "Found TEL directory at: $TEL_DIR"
            
            # Create a temporary directory for the npm global install
            TEMP_NPM_DIR=$(mktemp -d)
            export NPM_CONFIG_PREFIX="$TEMP_NPM_DIR"
            export PATH="$TEMP_NPM_DIR/bin:$PATH"
            
            cd "$TEL_DIR"
            echo "Working in directory: $(pwd)"
            
            # Install vsce using npm
            echo "Installing vsce for VSCode extension development..."
            ${pkgs.nodejs}/bin/npm install -g @vscode/vsce
            
            # Package the extension
            echo "Packaging the TEL syntax highlighting extension..."
            vsce package
            
            # Find the generated VSIX package
            VSIX_FILE=$(ls -t *.vsix 2>/dev/null | head -1)
            
            if [ -z "$VSIX_FILE" ]; then
              echo "No VSIX file was created. Check for errors above."
              exit 1
            fi
            
            # Instructions for installation in Cursor
            echo ""
            echo "TEL syntax highlighting extension packaged successfully!"
            echo "VSIX file created: $VSIX_FILE"
            echo ""
            echo "To install in Cursor:"
            echo "1. Open Cursor"
            echo "2. Go to View > Extensions"
            echo "3. Click on the '...' menu (top-right of Extensions panel)"
            echo "4. Select 'Install from VSIX...'"
            echo "5. Navigate to: $(pwd)/$VSIX_FILE"
            echo ""
            echo "After installation, .tel files should have syntax highlighting."
            
            # Clean up the temporary directory
            rm -rf "$TEMP_NPM_DIR"
          '');
        };
        
        apps.default = apps.tel-extension;
      }
    );
} 