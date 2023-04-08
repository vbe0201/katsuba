{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs = {self, flake-utils, naersk, nixpkgs}:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = (import nixpkgs) {
          inherit system;
        };

        naersk' = pkgs.callPackage naersk {};

        kobold_cli_cargo = builtins.fromTOML (builtins.readFile ./cli/Cargo.toml);

      in rec {
        defaultPackage = naersk'.buildPackage {
          name = "kobold";
          src = ./.;
          buildInputs = with pkgs; [cmake python3];
          version = kobold_cli_cargo.package.version;
        };

        devShell = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [rustc cargo];
        };
      }
    );
}
