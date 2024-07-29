# https://book.divnix.com/ch06-01-simple-c-program.html
{ pkgs ? import <nixpkgs> {} }:

with pkgs;
stdenv.mkDerivation {
  pname = "nostril";
  version = "0.1";

  src = ./.;

    #makeFlags = [ "PREFIX=$(out)" ];

    buildInputs = [ autoconf cargo cmake gcc gdb git python3 rustup secp256k1 vim ];
    buildPhase = ''
      rm -rf CMakeFiles CMakeCache.txt || true
      make simple nostril gnostr
    '';

    installPhase = ''
      mkdir -p $out/bin
      cp simple  $out/bin/simple
      cp nostril  $out/bin/nostril
      cp gnostr  $out/bin/gnostr
    '';

}
