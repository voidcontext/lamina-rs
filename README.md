# lamina-rs

A CLI tool to interact with nix flakes

#### Features:

- [x] print last modified date/time of flake inputs
- [ ] print last date/time of the revision (commit) of flake inputs
  - [ ] using git
  - [ ] using github api
  - [ ] using gitlab api (?)
- [ ] sync rev of a flake input from another flake, when branch, etc is matching
- [ ] fully sync the input with another flake including modifying `flake.nix`


#### Usage

```
Usage: lamina <COMMAND>

Commands:
  last-modified  Prints the last modified date/time of the flake inputs
  help           Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```