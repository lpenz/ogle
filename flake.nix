{
  description = "Execute a command periodically, showing the output only when it changes";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages."${system}";
      in
      rec {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "ogle";
          version = "2.1.0";
          src = self;
          cargoLock.lockFile = ./Cargo.lock;
        };
      }
    );
}
