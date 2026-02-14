{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = {
    flake-utils,
    rust-overlay,
    ...
  } @ inputs:
    flake-utils.lib.eachDefaultSystem (
      system: let
        overlays = [(import rust-overlay)];
        pkgs = import inputs.nixpkgs {
          inherit system overlays;
        };

        katsuba_toml = builtins.fromTOML (builtins.readFile ./src/katsuba/Cargo.toml);
        katsuba_py_toml = builtins.fromTOML (builtins.readFile ./src/katsuba-py/pyproject.toml);
      in {
        packages.default = pkgs.rustPlatform.buildRustPackage {
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

        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            clang
            maturin
            python3
            (rust-bin.stable.latest.default.override {
              extensions = ["rust-src" "rust-analyzer"];
            })
          ];
        };
      }
    );
}
