{
  inputs.nixpkgs-indirect-ref.url = "nixpkgs/release-23.05";
  inputs.nixpkgs-indirect-rev.url = "nixpkgs/dc1517e4b9e481e15bf7c80740a6a8a1423fe3ad";
  inputs.nixpkgs-github.url = "github:Nixos/nixpkgs/release-23.05";
  inputs.nix-rust-utils-git.url = "git+https://git.vdx.hu/voidcontext/nix-rust-utils.git?ref=refs/tags/v0.3.0";

  outputs = {...}: {};
}