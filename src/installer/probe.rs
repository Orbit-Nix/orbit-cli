// OrbitOS — System, Hardware, and Environment Probe

use std::fs;
use std::path::Path;
use crate::installer::types::{CpuType, GpuType, HostType};

#[derive(Debug, Clone)]
pub struct DetectedMachine {
    pub is_live_image: bool,
    pub hostname: String,
    pub cpu: CpuType,
    pub gpu: GpuType,
    pub host_type: HostType,
    pub has_battery: bool,
    pub has_ssh_keys: bool,
}

impl DetectedMachine {
    pub fn probe() -> Self {
        let is_live_image = detect_live_image();
        let hostname = detect_hostname();
        let has_battery = detect_battery();
        let cpu = detect_cpu();
        let gpu = detect_gpu(has_battery);
        let host_type = detect_archetype(has_battery, gpu);
        let has_ssh_keys = detect_ssh_keys();

        Self {
            is_live_image,
            hostname,
            cpu,
            gpu,
            host_type,
            has_battery,
            has_ssh_keys,
        }
    }
}

fn detect_live_image() -> bool {
    // 1. Check /etc/os-release for installer branding
    if let Ok(content) = fs::read_to_string("/etc/os-release") {
        let lower = content.to_lowercase();
        if lower.contains("installer") || lower.contains("live") {
            return true;
        }
    }

    // 2. Check /run/iso or /iso mounts
    if Path::new("/run/iso").exists() || Path::new("/iso").exists() {
        return true;
    }

    // 3. Check hostname / user default
    if let Ok(h) = fs::read_to_string("/etc/hostname") {
        if h.trim() == "nixos" {
            // Live ISO sets hostname to 'nixos' by default
            return true;
        }
    }

    false
}

fn detect_hostname() -> String {
    if let Ok(content) = fs::read_to_string("/etc/hostname") {
        let h = content.trim();
        if !h.is_empty() && h != "nixos" {
            return h.to_string();
        }
    }

    if let Ok(output) = std::process::Command::new("hostname").output() {
        let h = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !h.is_empty() && h != "nixos" {
            return h;
        }
    }

    "orbit".to_string()
}

fn detect_battery() -> bool {
    if let Ok(entries) = fs::read_dir("/sys/class/power_supply") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("BAT") {
                return true;
            }
        }
    }
    false
}

fn detect_cpu() -> CpuType {
    if let Ok(content) = fs::read_to_string("/proc/cpuinfo") {
        if content.contains("AuthenticAMD") {
            return CpuType::Amd;
        } else if content.contains("GenuineIntel") {
            return CpuType::Intel;
        }
    }
    CpuType::Universal
}

fn detect_gpu(has_battery: bool) -> GpuType {
    let mut has_nvidia = false;
    let mut has_amd = false;
    let mut has_intel = false;

    // Scan /sys/bus/pci/devices/*/vendor
    if let Ok(entries) = fs::read_dir("/sys/bus/pci/devices") {
        for entry in entries.flatten() {
            let vendor_path = entry.path().join("vendor");
            if let Ok(v) = fs::read_to_string(vendor_path) {
                let v = v.trim().to_lowercase();
                if v == "0x10de" {
                    has_nvidia = true;
                } else if v == "0x1002" {
                    has_amd = true;
                } else if v == "0x8086" {
                    has_intel = true;
                }
            }
        }
    }

    if has_nvidia {
        if has_battery {
            GpuType::NvidiaHybrid
        } else {
            GpuType::NvidiaDesktop
        }
    } else if has_amd {
        GpuType::Amd
    } else if has_intel {
        GpuType::IntelIntegrated
    } else {
        GpuType::Universal
    }
}

fn detect_archetype(has_battery: bool, gpu: GpuType) -> HostType {
    if has_battery || gpu == GpuType::NvidiaHybrid {
        return HostType::Laptop;
    }

    // Check DMI chassis type
    if let Ok(content) = fs::read_to_string("/sys/class/dmi/id/chassis_type") {
        let num: u32 = content.trim().parse().unwrap_or(0);
        match num {
            8 | 9 | 10 | 11 | 14 | 31 | 32 => return HostType::Laptop,
            17 | 23 | 28 => return HostType::Server,
            3 | 4 | 6 | 7 => return HostType::PC,
            _ => {}
        }
    }

    HostType::PC
}

fn detect_ssh_keys() -> bool {
    let user_info = crate::util::get_target_user();
    let ssh_dir = user_info.home_dir.join(".ssh");
    if ssh_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(ssh_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("id_") && !name.ends_with(".pub") {
                    return true;
                }
            }
        }
    }
    false
}
