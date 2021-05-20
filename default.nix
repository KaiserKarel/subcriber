let
  oxalica_overlay = import (builtins.fetchTarball
    "https://github.com/oxalica/rust-overlay/archive/master.tar.gz");
  nixpkgs = import <nixpkgs> { overlays = [ oxalica_overlay ]; };
in with nixpkgs;
pkgs.mkShell {
    nativeBuildInputs = [
        openssl.dev
        gcc
        rustup
        rust-bin.nightly.latest.rust
        rustPackages.clippy
        jetbrains.clion
        nodejs
        pkg-config
        clang
        llvmPackages.libclang
    ];
    # why do we need to set the library path manually?
    shellHook = ''
      export LIBCLANG_PATH="${pkgs.llvmPackages.libclang}/lib";
    '';
}
