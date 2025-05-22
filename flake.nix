{
  description = "A very basic flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };

  outputs = { self, nixpkgs }:
  let
    system = "x86_64-linux";
    pkgs = nixpkgs.legacyPackages.${system};
  in {
    devShells.${system}.default = (pkgs.buildFHSEnv {
      name = "fhs";
      targetPkgs = pkgs: with pkgs; [
        gcc
        glibc
        gmp
        gnum4
        pre-commit
        zlib
        zsh
      ];
      runScript = "zsh";
    }).env;
  };
}
