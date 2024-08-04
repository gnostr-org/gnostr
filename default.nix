# https://book.divnix.com/ch06-01-simple-c-program.html
{ pkgs ? import <nixpkgs> {} }:

with pkgs;
stdenv.mkDerivation {
  pname = "gnostr";
  version = "0.0.1";

  src = ./.;

    #makeFlags = [ "PREFIX=$(out)" ];

    buildInputs = [ autoconf cargo coreutils gcc gdb git openssl python3 rustup secp256k1 vim ];
    buildPhase = ''
      make gnostr
    '';

    installPhase = ''
      mkdir -p $out/bin
      cp gnostr  $out/bin/gnostr
    '';

}
