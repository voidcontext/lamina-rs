{
  inputs.nix-rust-utils.url = "git+https://git.vdx.hu/voidcontext/nix-rust-utils.git";
  outputs = {nix-rust-utils, ...}:
    nix-rust-utils.lib.mkOutputs ({...}: {crate.src = ./.;});
}
