{pkgs ? import <nixpkgs> {}}: let
  unstable = import (fetchTarball "channel:nixos-unstable") {};
in
  pkgs.mkShell {
    buildInputs = [
      unstable.rust-analyzer
      unstable.cargo-flamegraph
    ];
  }
