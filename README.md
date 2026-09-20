# Orbit CLI (`orbit`)

Fast, unified system management, imperative package runner, and rebuilder CLI for OrbitOS / NixOS.

---

## Quick Start

You can run and use `orbit` immediately on **any** NixOS system without modifying your configuration:

### Method 1: Drop into an interactive shell with `orbit`
```bash
nix-shell -p '(import (builtins.fetchTarball "https://github.com/m-uvex/NixOS/archive/main.tar.gz") {}).orbit'
```

### Method 2: Run directly via `nix run`
```bash
nix run github:m-uvex/NixOS#orbit -- rebuild -u
# or locally:
nix run .#default -- run firefox
```

### Method 3: Add to your system configuration (Declarative)
In your `flake.nix` or NixOS configuration:
```nix
# Add orbit flake input:
inputs.orbit.url = "github:m-uvex/NixOS";

# Include in system or user packages:
environment.systemPackages = [
  inputs.orbit.packages.${system}.default
];
```

---

## Features & Commands

### 1. Interactive Imperative App Runner (`yay`-style)
- **`orbit run <query>`** — Search packages on Nixpkgs, display formatted & numbered results with versions/descriptions/installed badges, and drop into an interactive `nix-shell` with selected packages.
  - Supports multi-selection: e.g. `1 2 3`, `1-3`, `1, 4, 5`.
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

### 4. Secrets Management
- **`orbit secrets store [--ssh] [--encryption <pass|ssh>]`** — Encrypt and archive `.ssh` into `secrets/ssh.tar.age`.
- **`orbit secrets restore [--ssh] [--encryption <pass|ssh>] [-a archive]`** — Decrypt and restore `.ssh`.

### 5. Installation
- **`orbit install [--config <path-or-repo>]`** — Prepare configuration (from local directory, clone, or custom path) and trigger `orbit rebuild -u`.

### Notice:
While this is meant to install and run on OrbitOS, it can also be used on any NixOS installation. Will fail when ran on anything other than NixOS (including forks) tho as a safeguard so it doesn't mess with your system.
Got ideas? feel free to fork and build with us! or even simply opening up a suggestion issue :D
