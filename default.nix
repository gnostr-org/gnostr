{ pkgs ? import <nixpkgs> {} }:

with pkgs;
stdenv.mkDerivation {
  pname = "nostril";
  version = "0.2.1";

  src = ./.;

  makeFlags = [ "PREFIX=$(out)" ];

  buildInputs = [ secp256k1 ];
}
