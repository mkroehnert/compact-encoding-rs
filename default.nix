# run with nix-shell .
with import <nixpkgs> {}; {
    wasmEnv = stdenv.mkDerivation {
        name = "rust-dev";
        buildInputs = [
          clangStdenv
          openssl
          pkgconfig
          binutils
          zlib
          sqlite
          zip
          cargo-make
          cargo-edit
          #wasm-pack
        ];

#      shellHook =
#      ;
#        extraCmds = ''
#        '';
    };
}
