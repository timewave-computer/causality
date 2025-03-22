# shell.nix - For compatibility with non-flake nix users
(import (fetchTarball "https://github.com/edolstra/flake-compat/archive/master.tar.gz") {
  src = ./.;
}).defaultNix.devShells.${builtins.currentSystem}.default 