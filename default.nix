{ pkgs ? import <nixpkgs> {} }:

pkgs.rustPlatform.buildRustPackage rec {
  name = "stare";
  src = ./.;
  cargoSha256 = "1vy35x7sdk3h09l2w08ywxldlcwgwq5hihd89w62jzppbi3p0007";
}
