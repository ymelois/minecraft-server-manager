# minecraft-server-manager

Lightweight Minecraft server manager with Unix-socket console and S3 backups.

## Requirements

In your NixOS config add `inputs.minecraft-server-manager.nixosModules.default` and the following configuration:

```nix
environment.systemPackages = with pkgs; [
  minecraft-server-manager
];

users.users."minecraft" = {
  isSystemUser = true;
  group = "minecraft";
  home = "/srv/minecraft";
  createHome = true;
};

users.groups."minecraft" = { };

services.minecraft = {
  enable = true;
};
```

## Usage

```
> minecraft-server-manager --help
Usage: minecraft-server-manager <COMMAND>

Commands:
  run     Run the wrapped server
  attach  Attach an interactive console (Ctrl-D to detach)
  send    Send a single command to the server and exit
  backup  Backup related subcommands
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

Add `/etc/minecraft/backup.env` with mode `640` and group `minecraft`:

```
AWS_ACCESS_KEY_ID=""
AWS_SECRET_ACCESS_KEY=""
S3_BUCKET=""
S3_ENDPOINT=""
S3_REGION=""
RUSTIC_PASSWORD=""
```

Run the Minecraft server with backups using systemd:

```
systemctl start minecraft@<name>
systemctl start minecraft-backup@<name>.timer
```

## License

This project is licensed under either of [`Apache License 2.0`](LICENSE-APACHE) or [`MIT License`](LICENSE-MIT) at your option.
