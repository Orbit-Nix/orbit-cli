// OrbitOS — Terminal Easter Egg Animations

pub const ORBIT_PY: &str = include_str!("../animation/orbit.py");
pub const MINI_ORBIT_PY: &str = include_str!("../animation/mini-orbit.py");
pub const HEART_PY: &str = include_str!("../animation/heart.py");
pub const MINI_HEART_PY: &str = include_str!("../animation/mini-heart.py");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationType {
    Orbit,
    MiniOrbit,
    Heart,
    MiniHeart,
}

pub fn play_animation(anim: AnimationType) -> anyhow::Result<()> {
    let script = match anim {
        AnimationType::Orbit => ORBIT_PY,
        AnimationType::MiniOrbit => MINI_ORBIT_PY,
        AnimationType::Heart => HEART_PY,
        AnimationType::MiniHeart => MINI_HEART_PY,
    };

    let status = std::process::Command::new("python3")
        .arg("-c")
        .arg(script)
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status();

    match status {
        Ok(_) => Ok(()),
        Err(e) => {
            crate::util::print_err(&format!("Failed to start animation runtime (python3): {}", e));
            anyhow::bail!("Animation failed to execute");
        }
    }
}
