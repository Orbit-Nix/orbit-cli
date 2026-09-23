# Orbit CLI (`orbit`)

Fast all-in-one NixOS CLI tool and [OrbitOS](https://github.com/Orbit-Nix/OrbitOS) installer.
Unified system management, imperative package runner, rebuilder & updater, and more!

---

> [!WARNING]
> Orbit-CLI is in very early stages, please use with caution.   
> Any contributions are very welcome!

> [!NOTE]
> Here to install or build your own [OrbitOS](https://github.com/Orbit-Nix/OrbitOS)?    
> Please read [The OS building and installation guide](https://github.com/Orbit-Nix/orbit-cli/blob/main/INSTALLOS.md).

## Quick Start

While Orbit-CLI is meant to be used on OrbitOS, it is also available (with limited features) for any NixOS config, also including the OrbitOS builder.
You can install and use Orbit-CLI with one of 3 methods;

#### Method 1: Drop into an interactive shell with `orbit` (Imperative)
```bash
nix-shell -p '(import (builtins.fetchTarball "https://github.com/Orbit-Nix/orbit-cli/archive/main.tar.gz") {}).orbit'
```

#### Method 2: Run via `nix run` (Imperative)
```bash
nix run github:Orbit-Nix/orbit-cli -- rebuild -u
```

#### Method 3: Add to your system configuration (Declarative)
In your `flake.nix`:
```nix
{
  inputs = {
    # 1. Add orbit flake input:
    orbit.url = "github:Orbit-Nix/orbit-cli";
  };

  outputs = { self, nixpkgs, orbit, ... }: {
    # ... your nixosSystem configurations ...
    
    # 2. Include in system or user packages:
    environment.systemPackages = [
      orbit.packages.${pkgs.system}.default
    ];
  };
}
```

#### And you're *in Orbit!*

---

## Features & Commands

> [!NOTE]
> All commands but the builder (not implemented yet) will fail when ran on anything other than NixOS (including forks) as a safeguard so it doesn't mess with your system.

### 1. Interactive Imperative App Runner
- **`orbit run <query>`** — Inpired by Yay, searches for any app in nixpkgs and let's you pick which to launch, like a search engine.
  - Supports multi-selection
  ```bash
  orbit run <query> (e.g. sl)
  ```
- **`orbit run-gui <query>`** — Same as above, but immediately launches the app after downloading. Meant for GUI apps.
  ```bash
  orbit run-gui zen-browser
  ```

### 2. System Rebuilder
- **`orbit rebuild`**: Build and apply NixOS system configuration. (Uses the current host name)
  - `-u, --update` : Update flake inputs and rebuild.
  - `-t, --test` : Test build without adding to bootloader menu.
  - `-v, --vm` : Build configs into a VM saved in `~/vmachines/` and launches it.
  - `-d, --dry` : Dry run previewing package diffs and changes.
  - `-a, --ask` : Ask for confirmation before applying.
  - `--show-trace` : Output evaluation traces.
- **`orbit update`** — Update flake inputs without rebuilding.
- **`orbit build`** — Build generation without activating. (Only adds to bootloader)
- **`orbit dry`** — Dry-run preview of package diffs and changes.
- **`orbit clean`** — Clean older generations freeing up space.

### 3. Config Git Sync
- **`orbit sync`** — Pull incoming changes from the config repository. If uncommitted local changes exist, prompts to:
  1. Abort sync (keep local changes).
  2. Overwrite local (reset to remote).
  3. Overwrite remote (commit and force push).

### 4. Shell Switcher (OrbitOS only!)
- **`orbit shell [shell name]`** — Change the active Hyprland dotfiles to any of the following (More to come):
  1. [end4-pC](https://github.com/pctrade/end4-pC)
  2. [Midnight](https://github.com/dim-ghub/midnight-shell)
  3. [DankMaterialShell](https://danklinux.com)

### 5. Secrets Management
- **`orbit secrets store [--ssh] [--encryption <pass|ssh>]`** — Encrypt and archive ~/.ssh into secrets/ssh.tar.age.
- **`orbit secrets restore [--ssh] [--encryption <pass|ssh>] [-a archive]`** — Decrypt and restore `.ssh`.

### 6. Installation
- **`orbit install [--config <path-or-repo>]`** — Prepare configuration (from local directory, clone, or custom path) and trigger `orbit rebuild -u`.

> [!TIP]
> Got ideas? feel free to fork and build with us! or even simply opening up a suggestion issue, we'll take care of it :D
