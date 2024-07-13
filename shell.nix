{ pkgs ? import <nixpkgs> {} }:
with pkgs;
mkShell {
  buildInputs = [ autoreconfHook cmake gdb git python3 secp256k1 vim ];
}
