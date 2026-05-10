{ self, ... }:
{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.services.minecraft;
in
{
  options.services.minecraft = {
    enable = lib.mkEnableOption "minecraft";

    package = lib.mkOption {
      type = lib.types.package;
      default = self.packages.${pkgs.stdenv.hostPlatform.system}.default;
      description = "minecraft-server-manager package.";
    };
  };

  config = lib.mkIf cfg.enable {
    systemd.services."minecraft@" = {
      description = "Minecraft server (%i)";
      after = [ "network-online.target" ];
      wants = [ "network-online.target" ];

      serviceConfig = {
        Type = "simple";
        User = "minecraft";
        Group = "minecraft";
        WorkingDirectory = "/srv/minecraft/%i";

        RuntimeDirectory = "minecraft";
        RuntimeDirectoryMode = "0750";

        Environment = "PATH=${
          lib.makeBinPath [
            pkgs.bash
            pkgs.coreutils
          ]
        }";

        ExecStart = ''
          ${cfg.package}/bin/minecraft-server-manager run \
            --socket /run/minecraft/%i.sock \
            -- ./run.sh
        '';

        KillSignal = "SIGTERM";
        TimeoutStopSec = 60;
        Restart = "on-failure";
        RestartSec = "10s";

        # Hardening
        NoNewPrivileges = true;
        ProtectSystem = "strict";
        ProtectHome = true;
        PrivateTmp = true;
        PrivateDevices = true;
        ReadWritePaths = [ "/srv/minecraft/%i" ];
      };
    };
  };
}
