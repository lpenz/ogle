{
  description = "Execute a command periodically, showing the output only when it changes";
  inputs.nixpkgs.url = github:NixOS/nixpkgs/nixos-22.05;
  outputs = { self, nixpkgs }: {
    packages.x86_64-linux.default =
      with import nixpkgs { system = "x86_64-linux"; };

      rustPlatform.buildRustPackage {
        pname = "ogle";
        version = "1.4.3";
        src = self;
        cargoLock.lockFile = ./Cargo.lock;
      };
  };
}
