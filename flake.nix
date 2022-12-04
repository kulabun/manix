{
  description = "A fast CLI documentation searcher for Nix.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-22.11";
    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    home-manager = {
      url = "github:nix-community/home-manager/release-22.11";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs @ {
    self,
    nixpkgs,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachSystem flake-utils.lib.allSystems (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [
          (final: prev: {
            home-manager-options = inputs.home-manager.packages.${system}.docs-json;
            nix-options = (import "${nixpkgs}/nixos/release.nix" {inherit nixpkgs;}).options;
          })
        ];
      };
      naersk = pkgs.callPackage inputs.naersk {};
      manix = pkgs.callPackage ./. {inherit pkgs naersk;};
    in {
      packages.manix = manix;
      defaultPackage = manix;
    });
}
