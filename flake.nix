{
  description = "rustmission";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };
  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        supportedSystems = [ "x86_64-linux" ];
        forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
        pkgsFor = nixpkgs.legacyPackages;
      in {
        packages.default = pkgs.callPackage ./. { };

        devShells.default = import ./shell.nix { inherit pkgs; };
      }
    );
}

