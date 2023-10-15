{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    systems.url = "github:nix-systems/default";
    devenv.url = "github:cachix/devenv";
  };

  nixConfig = {
    extra-trusted-public-keys = "devenv.cachix.org-1:w1cLUi8dv3hnoSPGAuibQv+f9TZLr6cv/Hm9XgU50cw=";
    extra-substituters = "https://devenv.cachix.org";
  };

  outputs = {
    self,
    nixpkgs,
    systems,
    devenv,
  } @ inputs: let
    forEachSystem = nixpkgs.lib.genAttrs (import systems);
  in {
    packages = forEachSystem (
      system: let
        pkgs = nixpkgs.legacyPackages.${system};

        kobold_toml = builtins.fromTOML (builtins.readFile ./src/kobold/Cargo.toml);
        kobold_py_toml = builtins.fromTOML (builtins.readFile ./src/kobold-py/pyproject.toml);
      in {
        default = pkgs.rustPlatform.buildRustPackage {
          pname = "kobold";
          version = kobold_toml.package.version;
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          nativeBuildInputs = with pkgs.rustPlatform; [cargoBuildHook];
          buildAndTestSubdir = "src/kobold";
          buildInputs = with pkgs; [cmake python3];
        };

        "kobold-py" = pkgs.python3.pkgs.buildPythonPackage {
          pname = "kobold-py";
          version = kobold_py_toml.project.version;
          src = ./.;
          format = "pyproject";
          cargoDeps = pkgs.rustPlatform.importCargoLock {
            lockFile = ./Cargo.lock;
          };
          nativeBuildInputs = with pkgs.rustPlatform; [cargoSetupHook maturinBuildHook];
          buildAndTestSubdir = "src/kobold-py";
        };
      }
    );

    devShells = forEachSystem (
      system: let
        pkgs = nixpkgs.legacyPackages.${system};
      in {
        default = devenv.lib.mkShell {
          inherit inputs pkgs;
          modules = [
            {
              packages = with pkgs; [cmake maturin];

              languages = {
                nix.enable = true;
                python = {
                  enable = true;
                  package = pkgs.python311;
                  poetry.enable = true;
                };
                rust.enable = true;
              };
            }
          ];
        };
      }
    );
  };
}
