// OrbitOS — 3D Celestial Orbit Simulation (Direct Port of orbit.py)

const ASPECT: f64 = 2.05;
const ASCII_RAMP: &[u8] = b" .'`^\",:;Il!i><~+_-?][}{1)(|\\/tfjrxnuvczXYUJCLQ0OZmwqpbdkhao*#MW&8%B@$";

pub struct OrbitSimulation {
    light: (f64, f64, f64),
    base_r_planet: f64,
    spin_speed: f64,
    seed_lat: f64,
    seed_lon: f64,
    crater_freq: f64,
    bob_x: f64,
    bob_y: f64,
    base_orbit_a: f64,
    base_orbit_b: f64,
    base_proj_y: f64,
    tilt_z: f64,
    orbit_speed: f64,
    solar_core: char,
    bus_core: char,
}

impl Default for OrbitSimulation {
    fn default() -> Self {
        Self::new()
    }
}

impl OrbitSimulation {
    pub fn new() -> Self {
        // Pre-normalized light vector
        let lx: f64 = -0.55;
        let ly: f64 = -0.35;
        let lz: f64 = 0.75;
        let l_len = (lx * lx + ly * ly + lz * lz).sqrt();
        let light = (lx / l_len, ly / l_len, lz / l_len);

        Self {
            light,
            base_r_planet: 13.5,
            spin_speed: 0.14,
            seed_lat: 4.5,
            seed_lon: 5.2,
            crater_freq: 13.0,
            bob_x: 0.16,
            bob_y: 0.13,
            base_orbit_a: 34.0,
            base_orbit_b: 32.0,
            base_proj_y: 11.2,
            tilt_z: 0.05,
            orbit_speed: 0.02,
            solar_core: '#',
            bus_core: '0',
        }
    }

    fn transform_orbit(&self, x: f64, y: f64, z: f64, tilt_x: f64) -> (f64, f64, f64) {
        let y1 = y * tilt_x.cos() - z * tilt_x.sin();
        let z1 = y * tilt_x.sin() + z * tilt_x.cos();
        let x2 = x * self.tilt_z.cos() - y1 * self.tilt_z.sin();
        let y2 = x * self.tilt_z.sin() + y1 * self.tilt_z.cos();
        (x2, y2, z1)
    }

    fn get_planet_char(&self, px: f64, py: f64, pz: f64, spin: f64, r_planet: f64) -> char {
        let nx = px / r_planet;
        let ny = py / r_planet;
        let nz = pz / r_planet;

        let lat = ny.clamp(-1.0, 1.0).asin();
        let lon = px.atan2(pz) + spin;

        let c1 = (lon * self.seed_lon + (lat * self.seed_lat).cos()).sin() * 0.28;
        let c2 = (lon * 5.2 - lat * 4.4).cos() * 0.14;
        let c3 = (lat * self.crater_freq + (lon * 8.2).cos()).sin() * 0.08;
        let c4 = (lat * 24.0).sin() * 0.04;
        let terrain = c1 + c2 + c3 + c4;

        let (lx, ly, lz) = self.light;
        let dot = nx * lx + ny * ly + nz * lz;

        if dot < -0.16 {
            return ' ';
        }

        let (vx, vy, vz) = (0.0, 0.0, 1.0);
        let (hx, hy, hz) = (lx + vx, ly + vy, lz + vz);
        let hlen = (hx * hx + hy * hy + hz * hz).sqrt();
        let spec = (((nx * hx + ny * hy + nz * hz) / hlen).max(0.0).powi(9)) * 0.40;
        let rim = ((1.0 - nz).powf(2.5)) * 0.22;

        let intensity = (dot * 0.72 + spec + terrain + rim).max(0.0);
        let max_idx = ASCII_RAMP.len() - 1;
        let idx = ((intensity * max_idx as f64).round() as usize).min(max_idx);
        ASCII_RAMP[idx] as char
    }

    pub fn render(&self, width: usize, height: usize, t: f64) -> Vec<Vec<char>> {
        if width < 5 || height < 3 {
            return vec![vec![' '; width]; height];
        }

        let mut frame = vec![vec![' '; width]; height];
        let mut z_buf = vec![vec![-999.0; width]; height];

        let w = width as f64;
        let h = height as f64;

        let eff_size = w.min(h * ASPECT);
        let scale = (eff_size / 56.0).max(0.38);

        let r_planet = self.base_r_planet * scale;
        let orbit_a = self.base_orbit_a * scale;
        let orbit_b = self.base_orbit_b * scale;
        let proj_y = self.base_proj_y * scale;
        let tilt_x = (proj_y / orbit_b).clamp(0.1, 0.95).acos();

        let cx = w / 2.0 + (t * self.bob_x).sin() * 0.7 * scale;
        let cy = h / 2.0 + (t * self.bob_y).cos() * 0.35 * scale;
        let planet_spin = t * self.spin_speed;

        // 1. Orbit Trail
        let orbit_steps = (120.0 * scale).max(50.0) as usize;
        for i in 0..orbit_steps {
            if i % 2 != 0 {
                continue;
            }
            let theta = (i as f64 / orbit_steps as f64) * std::f64::consts::PI * 2.0;
            let rx = orbit_a * theta.cos();
            let ry = orbit_b * theta.sin();
            let (ox, oy, oz) = self.transform_orbit(rx, ry, 0.0, tilt_x);

            let px = (cx + ox).round() as i32;
            let py = (cy + oy / ASPECT).round() as i32;

            if px >= 0 && (px as usize) < width && py >= 0 && (py as usize) < height {
                let ux = px as usize;
                let uy = py as usize;
                if oz > z_buf[uy][ux] {
                    frame[uy][ux] = '.';
                    z_buf[uy][ux] = oz;
                }
            }
        }

        // 2. Planet Sphere
        let r_sq = r_planet * r_planet;
        for y in 0..height {
            let py = (y as f64 - cy) * ASPECT;
            for x in 0..width {
                let px = x as f64 - cx;
                let dist_sq = px * px + py * py;

                if dist_sq <= r_sq {
                    let pz = (r_sq - dist_sq).sqrt();
                    let edge = dist_sq.sqrt() / r_planet;

                    let ch = if edge > (if scale >= 0.8 { 0.955 } else { 0.91 }) {
                        let ang = py.atan2(px);
                        let pi = std::f64::consts::PI;
                        if (-pi / 8.0..pi / 8.0).contains(&ang) || ang >= 7.0 * pi / 8.0 || ang < -7.0 * pi / 8.0 {
                            '|'
                        } else if (pi / 8.0..3.0 * pi / 8.0).contains(&ang) || (-7.0 * pi / 8.0..-5.0 * pi / 8.0).contains(&ang) {
                            '/'
                        } else if (3.0 * pi / 8.0..5.0 * pi / 8.0).contains(&ang) || (-5.0 * pi / 8.0..-3.0 * pi / 8.0).contains(&ang) {
                            '-'
                        } else {
                            '\\'
                        }
                    } else {
                        self.get_planet_char(px, py, pz, planet_spin, r_planet)
                    };

                    frame[y][x] = ch;
                    z_buf[y][x] = pz;
                }
            }
        }

        // 3. Satellite
        let sat_rx = orbit_a * t.cos();
        let sat_ry = orbit_b * t.sin();
        let (sx, sy, sz) = self.transform_orbit(sat_rx, sat_ry, 0.0, tilt_x);

        let direction = if self.orbit_speed > 0.0 { 1.0 } else { -1.0 };
        let vel_rx = -orbit_a * t.sin() * direction;
        let vel_ry = orbit_b * t.cos() * direction;
        let (vx, vy, vz) = self.transform_orbit(vel_rx, vel_ry, 0.0, tilt_x);
        let v_len = (vx * vx + vy * vy + vz * vz).sqrt() + 1e-6;
        let (tx, ty, tz) = (vx / v_len, vy / v_len, vz / v_len);

        let (norm_x, norm_y, norm_z) = self.transform_orbit(0.0, 0.0, 1.0, tilt_x);
        let wx = ty * norm_z - tz * norm_y;
        let wy = tz * norm_x - tx * norm_z;
        let wz = tx * norm_y - ty * norm_x;

        let p_len = (sx * sx + sy * sy + sz * sz).sqrt() + 1e-6;
        let (nx, ny, nz) = (-sx / p_len, -sy / p_len, -sz / p_len);

        let sat_nodes = get_satellite_nodes(scale, self.bus_core, self.solar_core);

        let base_px = cx + sx;
        let base_py = cy + sy / ASPECT;

        for (glyph, (dt, dnorm, dw, dnadir)) in sat_nodes {
            let local_x = (dt * tx) + (dnorm * norm_x) + (dw * wx) + (dnadir * nx);
            let local_y = ((dt * ty) + (dnorm * norm_y) + (dw * wy) + (dnadir * ny)) / ASPECT;
            let pz_3d = sz + (dt * tz) + (dnorm * norm_z) + (dw * wz) + (dnadir * nz);

            let screen_x = (base_px + local_x).round() as i32;
            let screen_y = (base_py + local_y).round() as i32;

            if screen_x >= 0 && (screen_x as usize) < width && screen_y >= 0 && (screen_y as usize) < height {
                let ux = screen_x as usize;
                let uy = screen_y as usize;
                if pz_3d > z_buf[uy][ux] {
                    frame[uy][ux] = glyph;
                    z_buf[uy][ux] = pz_3d;
                }
            }
        }

        frame
    }
}

fn get_satellite_nodes(
    scale: f64,
    bc: char,
    sc: char,
) -> Vec<(char, (f64, f64, f64, f64))> {
    if scale < 0.65 {
        // Compact craft for smaller terminal viewports
        vec![
            ('[', (-0.6, 0.0, 0.0, 0.0)),
            (bc, (0.0, 0.0, 0.0, 0.0)),
            (']', (0.6, 0.0, 0.0, 0.0)),
            ('v', (-1.0, 0.0, 0.0, 0.0)),
            ('=', (0.0, 0.0, 1.1, 0.0)),
            (sc, (0.0, 0.0, 1.9, 0.0)),
            ('=', (0.0, 0.0, -1.1, 0.0)),
            (sc, (0.0, 0.0, -1.9, 0.0)),
        ]
    } else {
        // Full high-detail craft
        vec![
            ('[', (-0.7, 0.0, 0.0, 0.0)),
            (bc, (0.0, 0.0, 0.0, 0.0)),
            (']', (0.7, 0.0, 0.0, 0.0)),
            ('=', (0.0, 0.5, 0.0, 0.0)),
            ('=', (0.0, -0.5, 0.0, 0.0)),
            ('|', (0.0, 0.0, 0.0, 0.9)),
            ('(', (-0.7, 0.0, 0.0, 1.5)),
            ('=', (0.0, 0.0, 0.0, 1.6)),
            (')', (0.7, 0.0, 0.0, 1.5)),
            ('*', (0.0, 0.0, 0.0, 2.2)),
            (':', (0.0, 0.0, 0.0, -0.9)),
            ('^', (0.0, 0.0, 0.0, -1.6)),
            ('v', (-1.2, 0.0, 0.0, 0.0)),
            ('-', (0.0, 0.0, 1.2, 0.0)),
            ('+', (0.0, 0.5, 2.0, 0.0)),
            (sc, (0.0, 0.5, 2.8, 0.0)),
            (sc, (0.0, 0.5, 3.6, 0.0)),
            ('+', (0.0, 0.5, 4.4, 0.0)),
            ('|', (0.0, 0.0, 4.4, 0.0)),
            ('+', (0.0, -0.5, 2.0, 0.0)),
            (sc, (0.0, -0.5, 2.8, 0.0)),
            (sc, (0.0, -0.5, 3.6, 0.0)),
            ('+', (0.0, -0.5, 4.4, 0.0)),
            ('-', (0.0, 0.0, -1.2, 0.0)),
            ('+', (0.0, 0.5, -2.0, 0.0)),
            (sc, (0.0, 0.5, -2.8, 0.0)),
            (sc, (0.0, 0.5, -3.6, 0.0)),
            ('+', (0.0, 0.5, -4.4, 0.0)),
            ('|', (0.0, 0.0, -4.4, 0.0)),
            ('+', (0.0, -0.5, -2.0, 0.0)),
            (sc, (0.0, -0.5, -2.8, 0.0)),
            (sc, (0.0, -0.5, -3.6, 0.0)),
            ('+', (0.0, -0.5, -4.4, 0.0)),
        ]
    }
}
