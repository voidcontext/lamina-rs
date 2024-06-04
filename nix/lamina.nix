{
  system,
  lib,
  stdenv,
  libiconv,
  openssl,
  darwin,
  pkg-config,
  installShellFiles,
  crane,
  ...
}: let
  src = ../.;
  craneLib = crane.lib.${system};
  commonArgs = {
    inherit src;
    buildInputs = [openssl pkg-config] ++ lib.optionals stdenv.isDarwin [
      libiconv
      darwin.apple_sdk.frameworks.Security
    ];
  };

  cargoArtifacts = craneLib.buildDepsOnly commonArgs;

  lamina = craneLib.buildPackage (commonArgs
    // {
      doCheck = false;

      # Shell completions
      COMPLETIONS_TARGET = "target/";
      nativeBuildInputs = [installShellFiles];
      postInstall = ''
        installShellCompletion --bash target/lamina.bash
        installShellCompletion --fish target/lamina.fish
        installShellCompletion --zsh  target/_lamina
      '';

      passthru.checks = {
        inherit lamina;

        lamina-clippy = craneLib.cargoClippy (commonArgs
          // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- -Dwarnings -W clippy::pedantic -A clippy::missing-errors-doc -A clippy::missing-panics-doc";
          });

        lamina-doc = craneLib.cargoDoc (commonArgs
          // {
            inherit cargoArtifacts;
          });

        # Check formatting
        lamina-fmt = craneLib.cargoFmt {
          inherit src;
        };

        # # Audit dependencies
        # lamina-audit = craneLib.cargoAudit {
        #   inherit src advisory-db;
        # };

        # # Audit licenses
        # lamina-deny = craneLib.cargoDeny {
        #   inherit src;
        # };

        # Run tests with cargo-nextest
        # Consider setting `doCheck = false` on `lamina` if you do not want
        # the tests to run twice
        lamina-nextest = craneLib.cargoNextest (commonArgs
          // {
            inherit cargoArtifacts;
            partitions = 1;
            partitionType = "count";
            # skip integration tests
            cargoNextestExtraArgs = "-E 'not kind(test)'";
          });
      };
    });
in
  lamina
