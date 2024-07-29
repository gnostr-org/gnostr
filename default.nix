# https://book.divnix.com/ch06-01-simple-c-program.html
{ pkgs ? import <nixpkgs> {} }:

with pkgs;
stdenv.mkDerivation {
  pname = "nostril";
  version = "0.1";

  src = ./.;

    buildInputs = [ autoconf cargo cmake gcc gdb git python3 rustup secp256k1 vim ];
    buildPhase = ''
      make simple
    '';

    installPhase = ''
      mkdir -p $out/bin
      cp simple  $out/bin/simple
    '';


  #makeFlags = [ "PREFIX=$(out)" ];

}
