{ pkgs ? import <nixpkgs> { } }:
pkgs.mkShell {
  inputsFrom = with pkgs; [
    openssl
  ];

  buildInputs = with pkgs; [
    openssl
  ];

  packages = with pkgs; [
    openssl
    pkg-config
  ]; 
}

