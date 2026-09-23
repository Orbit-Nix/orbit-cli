{
  description = "Orbit CLI - Fast, modular system manager, package runner, and rebuilder for NixOS / OrbitOS";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "orbit";
          version = "0.1.0";
          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          nativeBuildInputs = with pkgs; [
            pkg-config
            rustPlatform.bindgenHook
          ];

          buildInputs = with pkgs; [
            sqlite
            openssl
          ];

          # Runtime dependencies wrapped or used by orbit
          meta = with pkgs.lib; {
            description = "Fast, modular system manager, rebuilder, and secret management CLI for NixOS / OrbitOS";
            homepage = "https://github.com/Orbit-Nix/orbit-cli";
            license = licenses.mit;
            mainProgram = "orbit";
          };
        };

        apps.default = flake-utils.lib.mkApp {
          drv = self.packages.${system}.default;
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            cargo
            rustc
            rustfmt
            clippy
            pkg-config
            sqlite
            openssl
          ];
        };
      });
}
