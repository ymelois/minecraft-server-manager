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
      self,
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

      mkRustToolchain =
        pkgs:
        pkgs.fenix.fromToolchainFile {
          file = ./rust-toolchain.toml;
          sha256 = "sha256-gh/xTkxKHL4eiRXzWv8KP7vfjSk61Iq48x47BEDFgfk=";
        };

      manifest = (lib.importTOML ./Cargo.toml).package;
    in
    {
      devShells = forAllSystems supportedSystems (
        pkgs:
        let
          rustToolchain = pkgs.fenix.combine [
            pkgs.rust-analyzer
            pkgs.fenix.latest.rustfmt
            (mkRustToolchain pkgs)
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

      packages = forAllSystems supportedSystems (
        pkgs:
        let
          rustToolchain = mkRustToolchain pkgs;

          rustPlatform = pkgs.makeRustPlatform {
            cargo = rustToolchain;
            rustc = rustToolchain;
          };
        in
        {
          default = rustPlatform.buildRustPackage {
            pname = manifest.name;
            version = manifest.version;
            src = lib.cleanSource ./.;
            cargoLock.lockFile = ./Cargo.lock;

            meta = {
              description = manifest.description;
              homepage = manifest.repository;
              license = with lib.licenses; [
                mit
                asl20
              ];
              mainProgram = manifest.name;
              platform = lib.platforms.linux;
            };
          };
        }
      );

      overlays.default = final: prev: {
        minecraft-server-manager = self.packages.${prev.stdenv.hostPlatform.system}.default;
      };

      nixosModules.default = import ./module.nix { inherit self; };
    };
}
