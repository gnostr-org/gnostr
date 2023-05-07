{ pkgs ? import <nixpkgs> {} }:

with pkgs;
stdenv.mkDerivation {
  pname = "nostril";
  version = "0.1.6";

  src = ./.;

  makeFlags = [ "PREFIX=$(out)" ];

  buildInputs = [ secp256k1 ];
}
