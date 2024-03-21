{ pkgs }:
pkgs.rustPlatform.buildRustPackage rec {
  pname = "rustmission";
  version = "0.1.0";
  cargoLock.lockFile = ./Cargo.lock;
  src = pkgs.lib.cleanSource ./.;
  buildInputs = [ pkgs.openssl ];
  nativeBuildInputs = [ pkgs.pkg-config ];
  doCheck = false;
}
