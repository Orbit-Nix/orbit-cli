# Orbit CLI (`orbit`)

Fast, unified system management, imperative package runner, and rebuilder CLI for OrbitOS / NixOS.

---

## 🚀 Quick Start / Imperative Installation

You can run and use `orbit` immediately on **any** NixOS system without modifying your configuration:

### Method 1: Drop into an interactive shell with `orbit`
```bash
nix-shell -p '(import (builtins.fetchTarball "https://github.com/m-uvex/NixOS/archive/main.tar.gz") {}).orbit'
```
*(or locally from the orbit repository / directory)*:
```bash
nix shell github:m-uvex/NixOS#orbit
# or if using a local clone:
nix shell .#default
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

## ⚡ Features & Commands

### 1. Interactive Imperative App Runner (`yay`-style)
- **`orbit run <query>`** — Search packages on Nixpkgs, display formatted & numbered results with versions/descriptions/installed badges, and drop into an interactive `nix-shell` with selected packages.
  - Supports multi-selection: e.g. `1 2 3`, `1-3`, `1, 4, 5`.
  ```bash
  orbit run sl
  ```
- **`orbit run-gui <query>`** — Same search and selection workflow, but immediately launches the application's executable upon building/downloading.
  ```bash
  orbit run-gui zen-browser
  ```

### 2. System Rebuilder
- **`orbit rebuild`** (or bare **`orbit`**) — Build and apply NixOS system configuration.
  - `-u, --update` : Update flake inputs and rebuild.
  - `-t, --test` : Test build without adding to bootloader menu.
  - `-v, --vm` : Build VM via `nixos-rebuild build-vm`, save runner to `~/vmachines/`, and launch.
  - `-d, --dry` : Dry run previewing package diffs and changes.
  - `-a, --ask` : Ask for confirmation before applying.
  - `--show-trace` : Output evaluation traces.
- **`orbit update`** — Update flake inputs (`nix flake update`) without rebuilding.
- **`orbit build`** — Build generation without activating.
- **`orbit dry`** — Dry-run preview of package diffs.
- **`orbit clean`** — Clean older generations with `nh clean all`.

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

### 6. OS Guard
- Strictly verifies that the running OS is NixOS / OrbitOS before executing any command.
