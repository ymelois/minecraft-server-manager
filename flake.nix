{
  description = "minecraft-server-manager development environment";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixpkgs-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.rust-analyzer-src.follows = "";
    };
  };

  outputs =
    {
      nixpkgs,
      fenix,
      ...
    }:
    let
      inherit (nixpkgs) lib;

      supportedSystems = [
        "x86_64-linux"
      ];

      forAllSystems =
        systems: f:
        lib.genAttrs systems (
          system:
          f (
            import nixpkgs {
              inherit system;
              overlays = [
                fenix.overlays.default
              ];
            }
          )
        );
    in
    {
      devShells = forAllSystems supportedSystems (
        pkgs:
        let
          toolchainFile = pkgs.fenix.fromToolchainFile {
            file = ./rust-toolchain.toml;
            sha256 = "sha256-gh/xTkxKHL4eiRXzWv8KP7vfjSk61Iq48x47BEDFgfk=";
          };

          rustToolchain = pkgs.fenix.combine [
            pkgs.rust-analyzer
            pkgs.fenix.latest.rustfmt
            toolchainFile
          ];
        in
        {
          default =
            with pkgs;
            mkShell {
              nativeBuildInputs = [
                rustToolchain
                jdk21_headless
              ];
            };
        }
      );
    };
}
