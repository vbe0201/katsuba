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

      in rec {
        defaultPackage = naersk'.buildPackage {
          name = "kobold";
          src = ./.;
          buildInputs = with pkgs; [cmake python3];
          # naersk names the derivation with an -unknown suffix without a version
          # see https://github.com/nix-community/naersk/issues/224
          version = "0.2.0";
        };

        devShell = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [rustc cargo];
        };
      }
    );
}
