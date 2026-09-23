# Build and Install OrbitOS

> [!IMPORTANT]
> A TUI interactive builder is planned that will let you build your own OrbitOS
> with whatever apps, features, hosts etc. you want, but until that's implemented,
> Building OrbitOS requires manual forking and editing of this repo.
> But the code is modular and documented with easy to read and understand comments, so this shouldn't be too hard.
> Below, is the guide to get you started with a *working* but kinda minimal OrbitOS.

## 1. Fork the repo (Optional)
Forking the repo is recommended if you want easier management and usage of multi-host and secrets features. But is completely optionally, you can always skip this step. 
If you do fork it though, please make sure to change out any Github URL in the guide with the one belonging to your fork.

- [Click here](https://github.com/Orbit-Nix/OrbitOS/fork) or on "Fork" in the top right corner of the repo page on Github
- (Optional) Rename and rewrite the description as you like.
- Click "Create fork"

## 2. Install needed tools
   ```bash
   nix-shell -p git micro '(import (builtins.fetchTarball "https://github.com/Orbit-Nix/orbit-cli/archive/main.tar.gz") {}).orbit'
   ```

## 3. Clone the template/repo
   ```bash
   git clone https://github.com/Orbit-Nix/OrbitOS ~/.cache/OrbitOS-template
   cd ~/.cache/OrbitOS-template
   ```

## 4. Configure your user
    - In `flake.nix`, set `username = "yourusername";` (or keep `"user"`).
    - In `users/<username>/default.nix`, set your initial/hashed password and add your SSH public keys to `openssh.authorizedKeys.keys`.
> [!TIP]
> Use `micro <file.nix>` to easily edit the file contents

## 5. Generate hardware configuration for your target host
   ```bash
   # Choose an archetype: desktop, laptop, or server
   sudo nixos-generate-config --dir ./hosts/desktop
   ```

## 6. Enable hardware profiles (CPU & GPU)
    - In `hosts/desktop/default.nix`, uncomment the matching hardware modules (e.g. `../../modules/hardware/amd-cpu.nix`, `../../modules/hardware/nvidia-desktop.nix`, etc.).

## 7. Install & apply
    - **Using Orbit-CLI (Recommended):
      ```bash
      orbit install ~/.cache/OrbitOS-template
      ```
    - **Fresh install from Live USB**:
      ```bash
      sudo nixos-install --flake .#desktop
      ```
    - **Existing NixOS system**:
      ```bash
      sudo nixos-rebuild switch --flake .#desktop
      ```

## 8. Reboot
    - For everything to get applied correctly, run `sudo reboot` once.

## Aaaannd you're *in Orbit!* 🛰️