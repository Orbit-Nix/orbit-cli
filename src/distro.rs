// OrbitOS — Host operating system detection and compatibility guard

use std::fs;
use anyhow::{bail, Result};

// Ensure that orbit is running on a supported NixOS / OrbitOS system
pub fn ensure_nixos() -> Result<()> {
    // Check /etc/os-release for ID=nixos
    if let Ok(content) = fs::read_to_string("/etc/os-release") {
        for line in content.lines() {
            let line = line.trim();
            if line == "ID=nixos" || line == "ID=\"nixos\"" || line.starts_with("ID=nixos") {
                return Ok(());
            }
        }
    }

    // Secondary check: /etc/NIXOS marker
    if std::path::Path::new("/etc/NIXOS").exists() {
        return Ok(());
    }

    // Tertiary check: nixos-version executable
    if std::process::Command::new("nixos-version").output().is_ok() {
        return Ok(());
    }

    bail!("Orbit is designed exclusively for NixOS / OrbitOS. Current system does not appear to be NixOS.")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ensure_nixos_on_nixos() {
        let _ = ensure_nixos();
    }
}
