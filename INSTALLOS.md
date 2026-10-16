# Build and Install OrbitOS

> [!IMPORTANT]
> A TUI interactive builder is planned that will let you build your own OrbitOS
> with whatever apps, features, hosts etc. you want, but until that's implemented,
> Building OrbitOS requires manual forking and editing of this repo.
> But the code is modular and documented with easy to read and understand comments, so this shouldn't be too hard.
> Below, is the guide to get you started with a *working* but kinda minimal OrbitOS.

This will only guide you to have a booting machine. Everything past that,
picking your apps, adding users, switching shells, theming apps & more is in
**[MODULES.md](https://github.com/Orbit-Nix/OrbitOS/blob/main/docs/MODULES.md)**,
which is the reference for every file in the tree. Read this one first, read
that one when you want to change something.

## 1. Fork the repo (Optional)

Forking the repo is recommended if you want easier management and usage of
multi-host and secrets features. But is completely optional, you can always skip
this step. If you do fork it though, please make sure to change out any Github
URL in the guide with the one belonging to your fork.

- [Click here](https://github.com/Orbit-Nix/OrbitOS/fork) or on "Fork" in the top
  right corner of the repo page on Github
- (Optional) Rename and rewrite the description as you like.
- Click "Create fork"

## 2. Install needed tools

```bash
nix-shell -p git micro '(import (builtins.fetchTarball "https://github.com/Orbit-Nix/orbit-cli/archive/main.tar.gz") {}).orbit'
```

That drops you into a shell with `git`, `micro` and `orbit` available, which is
everything the rest of this guide needs. Once OrbitOS is installed, `orbit` is
already in your `PATH` and you never need this again.

## 3. Clone the template/repo

OrbitOS expects to live at `/etc/nixos`. `modules/core/default.nix` points both
`nh` and the daily auto-upgrade there, and `/etc/nixos` is where NixOS itself
looks, so putting it there is what makes the rest painless.

```bash
sudo git clone https://github.com/Orbit-Nix/OrbitOS /etc/nixos
sudo chown -R "$USER" /etc/nixos      # skip this one if you're root
cd /etc/nixos
```

> [!TIP]
> Anywhere else works too, `/etc/nixos` is just the path that needs no
> explaining. If you keep the config somewhere else, change `programs.nh.flake`
> and `system.autoUpgrade.flake` in `modules/core/default.nix` to the same new
> path, and pass `--flake <path>` to `orbit`. If you skip that, rebuilds work
> but your unattended upgrades quietly do nothing useful.

On a machine that isn't going to be wiped, or if you'd rather not have your
config in `/etc`, clone into your home instead and deal with those two lines
later:

```bash
git clone https://github.com/Orbit-Nix/OrbitOS ~/.config/OrbitOS
cd ~/.config/OrbitOS
```

## 4. Configure your user

This is the only account you have to touch before the first build.

In `flake.nix`, set `username = "yourusername";` (or keep `"user"`), and make
sure it matches a directory under `users/`. If you want a different name,
rename the directory too:

```bash
git mv users/user users/yourusername
```

Then open `users/<username>/default.nix` and set two things:

```bash
mkpasswd -m sha-512
ssh-keygen -t ed25519
cat ~/.ssh/id_ed25519.pub
```

```nix
users.users.yourusername = {
  initialPassword = "nixos";       # readable, so the very first boot works
  # hashedPassword = "<what mkpasswd printed>";   # swap it in and delete the line above
  openssh.authorizedKeys.keys = [
    "ssh-ed25519 AAAA... you@laptop"           # so you can log in without typing it
  ];
};
```

> [!TIP]
> Use `micro <file.nix>` to easily edit the file contents. And don't change
> `system.stateVersion` in your host file. On a fresh install `24.05` is
> correct, and bumping it later is how you migrate a machine, not how you
> install one.

## 5. Delete the hosts you don't have

`flake.nix` ships four example hosts. Keep the ones you own, delete the rest,
from both places:

- `nixosConfigurations` in `flake.nix`
- the `host:` list in `.github/workflows/build.yml`

Building a host you don't own is the most common reason a fresh clone fails to
evaluate cleanly, and a deleted host also stops CI from building a machine that
doesn't exist.

## 6. Generate hardware configuration for your target host

Every host imports its own `hardware-configuration.nix`, and the ones in the
repo are placeholders: they use zeroed disk UUIDs and fail to mount `/` on
purpose, so a stale config can never quietly mount the wrong disk. Generate the
real one **on the machine you are installing**, from the live USB or from the
running system:

```bash
# Choose an archetype: desktop, laptop, or server
sudo nixos-generate-config --dir /etc/nixos/hosts/laptop
```

`portable` is the exception, it uses the universal hardware module and needs
nothing generated.

## 7. Enable hardware profiles (CPU & GPU)

In `hosts/<name>/default.nix`, uncomment the modules that match what the machine
actually has. They're all listed there, commented out:

```nix
../../modules/hardware/amd-cpu.nix        # or intel-cpu.nix
../../modules/hardware/nvidia-desktop.nix  # or nvidia-hybrid.nix for a laptop
```

Not sure, or building for hardware you don't have in front of you? Use
`universal-cpu.nix` and `universal-gpu.nix`. They're slower to reason about and
much faster to get a first boot out of.

## 8. The desktop, if the host has one

Every GUI host already has Hyprland, GDM, PipeWire, the dotfile engine and a
desktop shell set up, so there is nothing to do here to get a *working* desktop.
Two things are worth a look though:

- `mySystem.desktop.shell` in the host file picks the shell you land in first.
  `end4-pC` is the default and the most complete, `midnight` and `dms` are the
  alternatives, `none` is bare Hyprland. You can switch at any time later with
  `orbit shell`.
- `mySystem.apps` and `mySystem.gaming` are the lists of what gets installed.
  Leave them for now and trim them once you're in, MODULES.md has the keys.

## 9. The server, if the host is one

`hosts/server` also pulls in the Docker stacks, which need three things changed
before they're worth anything:

- the `domain` in `modules/roles/server.nix` **and** the same `domain` in
  `modules/services/stacks/infra.nix`. `home.arpa` is a placeholder that never
  collides with anything real, so it works on a LAN as-is.
- the `POSTGRES_PASSWORD=CHANGE_ME` in `modules/services/stacks/cloud.nix`
- the placeholder secrets in `autheliaConfig` and the password hash in
  `usersDatabase` in `modules/services/stacks/infra.nix`

> [!NOTE]
> The server role is the least finished part of OrbitOS, mostly because of
> Authelia. It'll evaluate and boot, but read
> [`modules/services/stacks/infra.nix`](https://github.com/Orbit-Nix/OrbitOS/blob/main/modules/services/stacks/infra.nix)
> before you point it at anything you'd mind losing. Desktop hosts are the
> well-trodden path.

Partitioning is opt-in. `hosts/server/disko.nix` is commented out of the host
because disko repartitions the disk on every build, so only wire it up if you're
setting a machine up from bare metal and you're happy to lose it:

```bash
sudo nix run github:nix-community/disko -- --mode disko hosts/server
```

## 10. Check it before you install

Both of these are safe to run anywhere, on any machine, and they'll tell you
about a broken host long before a bootloader is involved:

```bash
nix flake check   # evaluates every host you kept
nix fmt           # formats the tree, nixfmt not nixpkgs-fmt
```

> [!TIP]
> If you added a file of your own, `git add` it first. A flake in a git repo
> only sees tracked files, so a new module you never staged is silently not
> there, and the error you'll get is about an import that doesn't exist.

## 11. Install & apply

**From a Live USB**, after steps 3 to 7, plus 8 or 9 if you took them:

```bash
cd /etc/nixos
sudo nixos-install --flake .#laptop
```

The `#laptop` is not optional, it picks the host out of `nixosConfigurations`.
Use the name of the host you generated hardware config for.

**From an existing NixOS system**, once the machine is already running
something else:

```bash
orbit install /etc/nixos
```

That updates your flake inputs and rebuilds, using *this machine's hostname* as
the host name. On a machine you're reinstalling, that hostname is usually not
the one in your flake yet, so either name the machine to match first:

```bash
sudo hostnamectl set-hostname laptop
```

or skip `orbit install` and say which host you mean:

```bash
orbit rebuild -u -f /etc/nixos laptop
```

With no arguments at all, `orbit install` clones the template into `/orbitos`,
which is not where this config expects to live, so pass the path.

**Or just rebuild in place**, if you are already on OrbitOS and only changed
some files:

```bash
sudo nixos-rebuild switch --flake .#laptop
```

## 12. Reboot

For everything to get applied correctly, run `sudo reboot` once. Then change the
`nixos` password on the machine itself, since `initialPassword` sits in your
config in plain text and is readable by anyone who clones the repo:

```bash
passwd
```

## Everyday, once you're in

| Command | What it does |
| :--- | :--- |
| `orbit rebuild` | Build and switch this machine |
| `orbit rebuild -u` | Update flake inputs, then rebuild |
| `orbit rebuild -d` | Show what would change, change nothing |
| `orbit rebuild -t` | Activate now, no bootloader entry |
| `orbit rebuild laptop` | Build a different host from this machine |
| `orbit clean` | Garbage-collect old generations |
| `orbit shell` | Interactive menu to switch desktop shells |
| `orbit run <query>` | Search nixpkgs, drop into a shell with your pick |
| `orbit sync` | Pull your config repo, with a choice about local changes |
| `orbit secrets store` | Encrypt `~/.ssh` so every host shares one identity |

`orbit` picks your config up from the current directory, `$ORBIT_FLAKE`, then
`/orbitos`, then `/etc/nixos`.

## Where to go next

Everything else lives in
[MODULES.md](https://github.com/Orbit-Nix/OrbitOS/blob/main/docs/MODULES.md):

- what each module does, and when you're expected to change it
- the apps, gaming, shells and dotfile dictionaries, and how to add to them
- the Docker stacks, the `/srv` layout, and how to add your own
- the values in the tree that are yours to set, and what they have to agree with

## Aaaannd you're *in Orbit!* 🛰️
