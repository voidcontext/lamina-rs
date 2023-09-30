{
  inputs.nixpkgs.url = "nixpkgs/release-23.05";
  inputs.nixpkgs-unstable.url = "nixpkgs/nixpkgs-unstable";
  inputs.flake-utils.url = "github:numtide/flake-utils";

  inputs.nix-rust-utils.url = "git+ssh://gitea@git.vdx.hu/voidcontext/nix-rust-utils-staging.git?ref=source-filtering";
  inputs.nix-rust-utils.inputs.nixpkgs.follows = "nixpkgs";
  inputs.nix-rust-utils.inputs.nixpkgs-unstable.follows = "nixpkgs-unstable";

  outputs = {
    nixpkgs,
    flake-utils,
    nix-rust-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {inherit system;};
      nru = nix-rust-utils.mkLib {inherit pkgs;};
      commonArgs = {
        src = ./.;
        buildInputs =
          [
            pkgs.git
            pkgs.nix
          ]
          ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            pkgs.libiconv
          ];
        sourceFilter = path: type: true;
          # ((builtins.match "tests/fixtures" path != null) || (nru.craneLib.filterCargoSources path type));
      };

      crate = nru.mkCrate (commonArgs // {
        doCheck = false;
      });
      checks = nru.mkChecks (commonArgs // {
        inherit crate;
        nextest = true;
      });
    in {
      inherit checks;

      packages.default = crate;
      packages.cargo-nextest = checks.cargo-nextest.overrideAttrs (_: { RUST_BACKTRACE="full";});

      devShells.default = nru.mkDevShell {inputsFrom = [crate]; inherit checks;};
    });
}
