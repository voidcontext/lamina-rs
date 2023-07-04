{
  inputs = {
    nixpkgs-indirect-ref.url = "nixpkgs/release-23.05";
    nixpkgs-indirect-rev = {
      url = "nixpkgs/8b3ad2fe8e06c2189908b7164f2f0bf2126b83b1";
    };
    nixpkgs-github = {
      url = "github:Nixos/nixpkgs/release-23.05";
    };
    nix-rust-utils-git = {
      url = "git+https://github.com/voidcontext/nix-rust-utils.git?ref=refs/tags/v0.4.0";
    };
  };

  outputs = {...}: {};
}