{
  inputs.nixpkgs.url = "nixpkgs/release-23.05";
  inputs.flake-utils.url = "github:numtide/flake-utils";

  inputs.nix-rust-utils.url = "git+https://git.vdx.hu/voidcontext/nix-rust-utils.git?ref=refs/tags/v0.8.2";
  inputs.nix-rust-utils.inputs.nixpkgs.follows = "nixpkgs";

  outputs = {
    nixpkgs,
    flake-utils,
    nix-rust-utils,
    ...
  }:
    let 
    mkLamina = pkgs: 
    let 
      nru = nix-rust-utils.mkLib {inherit pkgs;};

      commonArgs = {
        src = ./.;
        buildInputs = pkgs.lib.optionals pkgs.stdenv.isDarwin [
          pkgs.libiconv
        ];
      };
    in
    rec {
      crate = nru.mkCrate (commonArgs
        // {
          doCheck = false;

          # Shell completions
          COMPLETIONS_TARGET="target/";
          nativeBuildInputs = [ pkgs.installShellFiles ];
          postInstall = ''
            installShellCompletion --bash target/lamina.bash
            installShellCompletion --fish target/lamina.fish
            installShellCompletion --zsh  target/_lamina
          '';
        });
      checks = nru.mkChecks (commonArgs
        // {
          inherit crate;
          nextest = true;
        });
    } ;
    outputs = 
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {inherit system;};    
      nru = nix-rust-utils.mkLib {inherit pkgs;};
      
      lamina = mkLamina pkgs;
    in {
      checks = builtins.removeAttrs lamina.checks ["cargo-nextest"];

      packages.default = lamina.crate;

      devShells.default = nru.mkDevShell {
        inputsFrom = [lamina.crate];
        inherit (lamina) checks;
      };
    });
  in outputs // {
    overlays.default = final: prev: {
      lamina = outputs.packages.${final.system}.default;
    };
    
    overlays.withHostPkgs = final: prev: {
      lamina = (mkLamina final).crate;
    };
  };
}
