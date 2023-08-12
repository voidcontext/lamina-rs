{
  inputs.nixpkgs.url = "nixpkgs/release-23.05";
  inputs.nixpkgs-unstable.url = "nixpkgs/nixpkgs-unstable";

  inputs.nix-rust-utils.url = "git+https://git.vdx.hu/voidcontext/nix-rust-utils.git?ref=refs/tags/v0.7.0";
  inputs.nix-rust-utils.inputs.nixpkgs.follows = "nixpkgs";
  inputs.nix-rust-utils.inputs.nixpkgs-unstable.follows = "nixpkgs-unstable";

  outputs = {nix-rust-utils, ...}:
    nix-rust-utils.lib.mkOutputs ({...}: {crate.src = ./.;});
}
