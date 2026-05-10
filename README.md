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

### Run

```
> minecraft-server-manager run --help
Run the wrapped server

Usage: minecraft-server-manager run --socket <SOCKET> <COMMAND>...

Arguments:
  <COMMAND>...  Command to run, after `--` (e.g. `-- java -Xmx4G -jar server.jar`)

Options:
  -s, --socket <SOCKET>  Unix socket to create for console access
  -h, --help             Print help
```

### Attach

```
> minecraft-server-manager attach --help
Attach an interactive console (Ctrl-D to detach)

Usage: minecraft-server-manager attach --socket <SOCKET>

Options:
  -s, --socket <SOCKET>  Unix socket of the running server
  -h, --help             Print help
```

### Send

```
> minecraft-server-manager send --help
Send a single command to the server and exit

Usage: minecraft-server-manager send --socket <SOCKET> <COMMAND>...

Arguments:
  <COMMAND>...  Command words, joined with spaces (e.g. `save-all flush`)

Options:
  -s, --socket <SOCKET>  Unix socket of the running server
  -h, --help             Print help
```

## License

This project is licensed under either of [`Apache License 2.0`](LICENSE-APACHE) or [`MIT License`](LICENSE-MIT) at your option.
