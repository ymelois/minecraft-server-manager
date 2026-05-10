# minecraft-server-manager

Straightforward tool to manage your minecraft servers.

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
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

## License

This project is licensed under either of [`Apache License 2.0`](LICENSE-APACHE) or [`MIT License`](LICENSE-MIT) at your option.
