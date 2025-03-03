{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    systems.url = "github:nix-systems/default";
    devenv.url = "github:cachix/devenv";
    nix2container.url = "github:nlewo/nix2container";
    nix2container.inputs.nixpkgs.follows = "nixpkgs";
    mk-shell-bin.url = "github:rrbutani/nix-mk-shell-bin";
  };

  nixConfig = {
    extra-trusted-public-keys = "devenv.cachix.org-1:w1cLUi8dv3hnoSPGAuibQv+f9TZLr6cv/Hm9XgU50cw=";
    extra-substituters = "https://devenv.cachix.org";
  };

  outputs = {
    self,
    flake-parts,
    ...
  } @ inputs:
    flake-parts.lib.mkFlake {inherit inputs;} {
      imports = [
        inputs.devenv.flakeModule
      ];

      systems = import inputs.systems;

      perSystem = {
        config,
        pkgs,
        system,
        self',
        ...
      }: let
        katsuba_toml = builtins.fromTOML (builtins.readFile ./src/katsuba/Cargo.toml);
        katsuba_py_toml = builtins.fromTOML (builtins.readFile ./src/katsuba-py/pyproject.toml);
      in {
        packages.katsuba = pkgs.rustPlatform.buildRustPackage {
          pname = katsuba_toml.package.name;
          version = katsuba_toml.package.version;
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          nativeBuildInputs = with pkgs.rustPlatform; [cargoBuildHook];
          buildAndTestSubdir = "src/katsuba";
          buildInputs = with pkgs; [python3];
        };

        packages."katsuba-py" = pkgs.python3.pkgs.buildPythonPackage {
          pname = "katsuba-py";
          version = katsuba_py_toml.project.version;
          src = ./.;
          pyproject = true;
          cargoDeps = pkgs.rustPlatform.importCargoLock {
            lockFile = ./Cargo.lock;
          };
          nativeBuildInputs = with pkgs.rustPlatform; [cargoSetupHook maturinBuildHook];
          pythonImportsCheck = ["katsuba"];
          buildAndTestSubdir = "src/katsuba-py";
        };

        packages.default = self'.packages.katsuba;

        devenv.shells.default = {
          packages = with pkgs; [git maturin];

          languages = {
            nix.enable = true;
            python = {
              enable = true;
              package = pkgs.python3;
            };
            rust.enable = true;
          };
        };
      };
    };
}
