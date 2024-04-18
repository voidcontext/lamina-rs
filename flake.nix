{
  inputs.nixpkgs.url = "nixpkgs/release-23.11";
  inputs.flake-utils.url = "github:numtide/flake-utils";

  inputs.crane = {
    url = "github:ipetkov/crane/v0.15.1";
    inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    crane,
    ...
  }: let
    mkLamina = import ./nix/lamina.nix;
    outputs = flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {inherit system;};
        craneLib = crane.lib.${system};
        callPackage = pkgs.lib.callPackageWith (pkgs // {inherit crane;});
        lamina = callPackage mkLamina {};
      in {
        checks = builtins.removeAttrs lamina.checks ["cargo-nextest"];

        packages.default = lamina;

        devShells.default = craneLib.devShell {
          checks = self.checks.${system};
          packages = [
            pkgs.rust-analyzer
          ];
        };
      }
    );
  in
    outputs
    // {
      overlays.default = final: prev: {
        lamina = outputs.packages.${final.system}.default;
      };

      overlays.withHostPkgs = final: prev: let
        callPackage = final.lib.callPackageWith (final // {inherit crane;});
      in {
        lamina = callPackage mkLamina {};
      };
    };
}
