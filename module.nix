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

        ExecStart = "${lib.getExe cfg.package} run --socket /run/minecraft/%i.sock -- ./run.sh";

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

    systemd.services."minecraft-backup@" = {
      description = "Minecraft backup (%i)";

      serviceConfig = {
        Type = "oneshot";
        User = "minecraft";
        Group = "minecraft";

        # Mode 640, group minecraft
        EnvironmentFile = "/etc/minecraft/backup.env";

        ExecStartPre = "${lib.getExe cfg.package} send --socket /run/minecraft/%i.sock save-off";

        ExecStart = "${lib.getExe cfg.package} backup --root %i create /srv/minecraft/%i";

        ExecStopPost = "-${lib.getExe cfg.package} send --socket /run/minecraft/%i.sock save-on";

        Nice = 10;
        IOSchedulingClass = "idle";

        # Hardening
        NoNewPrivileges = true;
        ProtectSystem = "strict";
        ProtectHome = true;
        PrivateTmp = true;
        PrivateDevices = true;
        ReadOnlyPaths = [ "/srv/minecraft/%i" ];
      };
    };

    systemd.timers."minecraft-backup@" = {
      description = "Hourly Minecraft backup (%i)";

      timerConfig = {
        OnCalendar = "hourly";
        Persistent = true;
        RandomizedDelaySec = "5m";
        Unit = "minecraft-backup@%i.service";
      };
    };
  };
}
