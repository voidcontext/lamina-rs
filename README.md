# lamina-rs

A high level CLI tool to manage and sync nix flakes

#### Features:

- [x] print last modified date/time of flake inputs
- [ ] print last date/time of the revision (commit) of flake inputs
  - [ ] using git
  - [ ] using github api
  - [ ] using gitlab api (?)
- [x] sync rev of a flake input from another flake, when branch, etc is matching
  - [x] git
  - [x] github
  - [x] gitlab - not fully tested
- [x] fully sync the input with another flake including modifying `flake.nix`


#### Usage

```
Usage: lamina [OPTIONS] <COMMAND>

Commands:
  sync           Syncs input with another flake
  batch-sync     Syncs multiple inputs with another flake, the inputs need to have matching name
  last-modified  Prints the last modified date/time of the flake inputs
  help           Print this message or the help of the given subcommand(s)

Options:
  -d, --debug
  -h, --help   Print help
```
