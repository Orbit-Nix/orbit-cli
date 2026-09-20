import math
import os
import random
import select
import signal
import sys
import time

try:
    import termios
    import tty
    HAS_TERMIOS = True
except ImportError:
    HAS_TERMIOS = False

ASPECT = 2.05
ASCII_RAMP = "$@B%8&WM#*oahkbdpqwmZO0QLCJUYXzcvunxrjft/\\|()1{}[]?-_+~<>i!lI;:,\"^`'. "
RAMP_REV = ASCII_RAMP[::-1]

def get_terminal_dimensions():
    try:
        cols, lines = os.get_terminal_size()
        return max(30, cols), max(12, lines)
    except Exception:
        return 42, 18

class ProceduralWorld:
    def __init__(self):
        # 1. Lighting Vector
        lx = random.uniform(-0.85, -0.35)
        ly = random.uniform(-0.55, -0.20)
        lz = random.uniform(0.60, 0.85)
        l_len = math.sqrt(lx*lx + ly*ly + lz*lz)
        self.light = (lx / l_len, ly / l_len, lz / l_len)

        # 2. Planet Parameters (Base reference at 42x18)
        self.base_r_planet = random.uniform(5.0, 5.6)
        self.spin_speed = random.uniform(0.12, 0.22) * random.choice([-1, 1])
        self.seed_lat = random.uniform(3.2, 5.8)
        self.seed_lon = random.uniform(3.5, 6.8)
        self.crater_freq = random.uniform(10.0, 15.0)
        
        # Sub-cell gentle planet bobbing
        self.bob_x = random.uniform(0.12, 0.20)
        self.bob_y = random.uniform(0.10, 0.16)

        # Orbit Parameters (Base reference at 42x18)
        self.base_orbit_a = random.uniform(15.0, 17.0)
        self.base_orbit_b = random.uniform(14.0, 16.0)
        self.base_proj_y = random.uniform(5.0, 5.8)
        self.tilt_z = random.uniform(-0.16, 0.16)
        
        # Movement speed
        self.orbit_speed = random.uniform(0.024, 0.034) * random.choice([-1, 1])

        # Satellite Components
        self.solar_core = random.choice(['#', 'X', 'H', '%'])
        self.bus_core = random.choice(['0', 'O', '8', '@'])

        self.width = 42
        self.height = 18
        self.scale = 1.0
        self.update_dimensions(42, 18)

    def update_dimensions(self, width, height):
        self.width = width
        self.height = height
        eff_size = min(width, height * ASPECT)
        self.scale = max(0.4, eff_size / 37.0)

        self.r_planet = self.base_r_planet * self.scale
        self.orbit_a = self.base_orbit_a * self.scale
        self.orbit_b = self.base_orbit_b * self.scale
        proj_y = self.base_proj_y * self.scale
        self.tilt_x = math.acos(min(0.95, max(0.1, proj_y / self.orbit_b)))

    def transform_orbit(self, x, y, z):
        y1 = y * math.cos(self.tilt_x) - z * math.sin(self.tilt_x)
        z1 = y * math.sin(self.tilt_x) + z * math.cos(self.tilt_x)
        x2 = x * math.cos(self.tilt_z) - y1 * math.sin(self.tilt_z)
        y2 = x * math.sin(self.tilt_z) + y1 * math.cos(self.tilt_z)
        return x2, y2, z1

    def get_planet_char(self, px, py, pz, spin):
        nx = px / self.r_planet
        ny = py / self.r_planet
        nz = pz / self.r_planet

        lat = math.asin(max(-1.0, min(1.0, ny)))
        lon = math.atan2(px, pz) + spin

        c1 = math.sin(lon * self.seed_lon + math.cos(lat * self.seed_lat)) * 0.28
        c2 = math.cos(lon * 5.2 - lat * 4.4) * 0.14
        c3 = math.sin(lat * self.crater_freq + math.cos(lon * 8.2)) * 0.08
        c4 = math.sin(lat * 24.0) * 0.04
        terrain = c1 + c2 + c3 + c4

        lx, ly, lz = self.light
        dot = nx * lx + ny * ly + nz * lz

        if dot < -0.16:
            return ' '

        vx, vy, vz = 0.0, 0.0, 1.0
        hx, hy, hz = lx + vx, ly + vy, lz + vz
        hlen = math.sqrt(hx*hx + hy*hy + hz*hz)
        spec = (max(0.0, (nx*hx + ny*hy + nz*hz) / hlen) ** 9) * 0.40
        rim = ((1.0 - nz) ** 2.5) * 0.22

        intensity = max(0.0, dot * 0.72 + spec + terrain + rim)
        idx = int(intensity * (len(RAMP_REV) - 1))
        return RAMP_REV[max(0, min(len(RAMP_REV) - 1, idx))]

def rotate_3d(x, y, z, rx, ry, rz):
    cx, sx = math.cos(rx), math.sin(rx)
    y1 = y * cx - z * sx
    z1 = y * sx + z * cx
    cy, sy = math.cos(ry), math.sin(ry)
    x2 = x * cy + z1 * sy
    z2 = -x * sy + z1 * cy
    cz, sz = math.cos(rz), math.sin(rz)
    x3 = x2 * cz - y1 * sz
    y3 = x2 * sz + y1 * cz
    return x3, y3, z2

def check_stdin_key():
    try:
        r, _, _ = select.select([sys.stdin], [], [], 0)
        if r:
            ch = os.read(sys.stdin.fileno(), 32)
            if ch:
                return ch.decode('utf-8', errors='ignore')
    except Exception:
        pass
    return None

def get_satellite_nodes(world):
    sc = world.solar_core
    bc = world.bus_core

    body_nodes = [
        ('[',  [-0.6,  0.0,  0.0,  0.0]),
        (bc,   [ 0.0,  0.0,  0.0,  0.0]),
        (']',  [ 0.6,  0.0,  0.0,  0.0]),
        ('v',  [-1.0,  0.0,  0.0,  0.0])
    ]
    left_wing_nodes = [
        ('=',  [ 0.0,  0.0,  1.1,  0.0]),
        (sc,   [ 0.0,  0.0,  1.9,  0.0]),
    ]
    right_wing_nodes = [
        ('=',  [ 0.0,  0.0, -1.1,  0.0]),
        (sc,   [ 0.0,  0.0, -1.9,  0.0]),
    ]
    return body_nodes, left_wing_nodes, right_wing_nodes

def render_frame(world, t):
    w, h = world.width, world.height
    frame = [[' ' for _ in range(w)] for _ in range(h)]
    z_buf = [[-999.0 for _ in range(w)] for _ in range(h)]

    # floating-point planet center
    cx = w / 2.0 + math.sin(t * world.bob_x) * 0.4 * world.scale
    cy = h / 2.0 + math.cos(t * world.bob_y) * 0.2 * world.scale
    planet_spin = t * world.spin_speed

    # Orbit Trail
    orbit_steps = max(30, int(60 * world.scale))
    for i in range(orbit_steps):
        if i % 2 != 0:
            continue
        theta = (i / float(orbit_steps)) * math.pi * 2
        rx = world.orbit_a * math.cos(theta)
        ry = world.orbit_b * math.sin(theta)
        ox, oy, oz = world.transform_orbit(rx, ry, 0.0)

        px = round(cx + ox)
        py = round(cy + oy / ASPECT)

        if 0 <= px < w and 0 <= py < h:
            if oz > z_buf[py][px]:
                frame[py][px] = '.'
                z_buf[py][px] = oz

    # Planet Sphere
    for y in range(h):
        py = (y - cy) * ASPECT
        for x in range(w):
            px = x - cx
            dist_sq = px * px + py * py
            r_sq = world.r_planet * world.r_planet

            if dist_sq <= r_sq:
                pz = math.sqrt(r_sq - dist_sq)
                edge = math.sqrt(dist_sq) / world.r_planet

                if edge > 0.90:
                    ang = math.atan2(py, px)
                    if -math.pi/8 <= ang < math.pi/8 or ang >= 7*math.pi/8 or ang < -7*math.pi/8:
                        ch = '|'
                    elif math.pi/8 <= ang < 3*math.pi/8 or -7*math.pi/8 <= ang < -5*math.pi/8:
                        ch = '/'
                    elif 3*math.pi/8 <= ang < 5*math.pi/8 or -5*math.pi/8 <= ang < -3*math.pi/8:
                        ch = '-'
                    else:
                        ch = '\\'
                else:
                    ch = world.get_planet_char(px, py, pz, planet_spin)

                frame[y][x] = ch
                z_buf[y][x] = pz

    # Satellite
    sat_rx = world.orbit_a * math.cos(t)
    sat_ry = world.orbit_b * math.sin(t)
    sx, sy, sz = world.transform_orbit(sat_rx, sat_ry, 0.0)

    # Tangent vector
    direction = 1.0 if world.orbit_speed > 0 else -1.0
    vel_rx = -world.orbit_a * math.sin(t) * direction
    vel_ry = world.orbit_b * math.cos(t) * direction
    vx, vy, vz = world.transform_orbit(vel_rx, vel_ry, 0.0)
    v_len = math.sqrt(vx*vx + vy*vy + vz*vz) + 1e-6
    tx, ty, tz = vx/v_len, vy/v_len, vz/v_len

    # Normal & wing vectors
    norm_x, norm_y, norm_z = world.transform_orbit(0.0, 0.0, 1.0)
    wx = ty * norm_z - tz * norm_y
    wy = tz * norm_x - tx * norm_z
    wz = tx * norm_y - ty * norm_x

    # Nadir vector
    p_len = math.sqrt(sx*sx + sy*sy + sz*sz) + 1e-6
    nx, ny, nz = -sx/p_len, -sy/p_len, -sz/p_len

    body_nodes, left_wing_nodes, right_wing_nodes = get_satellite_nodes(world)
    all_sat_nodes = body_nodes + left_wing_nodes + right_wing_nodes

    base_px = cx + sx
    base_py = cy + sy / ASPECT

    for glyph, (dt, dnorm, dw, dnadir) in all_sat_nodes:
        local_x = (dt * tx) + (dnorm * norm_x) + (dw * wx) + (dnadir * nx)
        local_y = ((dt * ty) + (dnorm * norm_y) + (dw * wy) + (dnadir * ny)) / ASPECT
        pz_3d = sz + (dt * tz) + (dnorm * norm_z) + (dw * wz) + (dnadir * nz)

        screen_x = round(base_px + local_x)
        screen_y = round(base_py + local_y)

        if 0 <= screen_x < w and 0 <= screen_y < h:
            if pz_3d > z_buf[screen_y][screen_x]:
                frame[screen_y][screen_x] = glyph
                z_buf[screen_y][screen_x] = pz_3d

    sys.stdout.write("\x1b[H" + "\n".join("".join(row) for row in frame))
    sys.stdout.flush()

def play_quit_sequence(world, start_t):
    COMET_FRAMES = 35
    TOTAL_FRAMES = 160

    w, h = world.width, world.height
    scale = world.scale

    sat_rx = world.orbit_a * math.cos(start_t)
    sat_ry = world.orbit_b * math.sin(start_t)
    sat_pos_x, sat_pos_y, sat_pos_z = world.transform_orbit(sat_rx, sat_ry, 0.0)

    direction = 1.0 if world.orbit_speed > 0 else -1.0
    vel_rx = -world.orbit_a * math.sin(start_t) * direction
    vel_ry = world.orbit_b * math.cos(start_t) * direction
    init_orb_vx, init_orb_vy, init_orb_vz = world.transform_orbit(vel_rx, vel_ry, 0.0)
    v_mag = math.sqrt(init_orb_vx**2 + init_orb_vy**2 + init_orb_vz**2) + 1e-6
    tx, ty, tz = init_orb_vx/v_mag, init_orb_vy/v_mag, init_orb_vz/v_mag

    norm_x, norm_y, norm_z = world.transform_orbit(0.0, 0.0, 1.0)
    wx = ty * norm_z - tz * norm_y
    wy = tz * norm_x - tx * norm_z
    wz = tx * norm_y - ty * norm_x

    p_mag = math.sqrt(sat_pos_x**2 + sat_pos_y**2 + sat_pos_z**2) + 1e-6
    nx, ny, nz = -sat_pos_x/p_mag, -sat_pos_y/p_mag, -sat_pos_z/p_mag

    body_nodes, left_wing_nodes, right_wing_nodes = get_satellite_nodes(world)

    comet_side = random.choice([-1.0, 1.0])
    comet_start_x = comet_side * random.uniform(24.0, 28.0) * scale
    comet_start_y = -random.uniform(10.0, 14.0) * scale * ASPECT
    comet_start_z = random.uniform(15.0, 22.0) * scale

    impact_px = -comet_side * random.uniform(0.15, 0.35) * world.r_planet
    impact_py = random.uniform(-0.3, 0.2) * world.r_planet
    impact_pz = math.sqrt(max(0.5, world.r_planet**2 - impact_px**2 - impact_py**2))

    comet_trail = []
    planet_fragments = []
    plasma_particles = []
    orbit_debris = []
    sat_sparks = []

    sat_curr_pos = [sat_pos_x, sat_pos_y, sat_pos_z]
    sat_vel = [0.0, 0.0, 0.0]
    sat_rot = [0.0, 0.0, 0.0]
    sat_rot_vel = [0.0, 0.0, 0.0]

    wing1_pos = None
    wing1_vel = None
    wing1_rot = [0.0, 0.0, 0.0]
    wing1_rot_vel = None

    wing2_pos = None
    wing2_vel = None
    wing2_rot = [0.0, 0.0, 0.0]
    wing2_rot_vel = None

    wing1_detached = False
    wing2_detached = False

    t = start_t
    last_dims = (w, h)

    for frame_idx in range(TOTAL_FRAMES):
        key = check_stdin_key()
        if key and any(c in ('q', 'Q', '\x1b', '\x03', '\x04') for c in key):
            break

        curr_w, curr_h = get_terminal_dimensions()
        if (curr_w, curr_h) != last_dims:
            last_dims = (curr_w, curr_h)
            world.update_dimensions(curr_w, curr_h)
            w, h = curr_w, curr_h
            scale = world.scale
            sys.stdout.write("\x1b[2J")

        frame = [[' ' for _ in range(w)] for _ in range(h)]
        z_buf = [[-999.0 for _ in range(w)] for _ in range(h)]

        cx = w / 2.0 + math.sin(t * world.bob_x) * 0.4 * scale
        cy = h / 2.0 + math.cos(t * world.bob_y) * 0.2 * scale
        planet_spin = t * world.spin_speed

        impact_world_x = cx + impact_px
        impact_world_y = cy + impact_py / ASPECT

        if frame_idx < COMET_FRAMES:
            orbit_steps = max(30, int(60 * scale))
            for i in range(orbit_steps):
                if i % 2 != 0:
                    continue
                theta = (i / float(orbit_steps)) * math.pi * 2
                rx = world.orbit_a * math.cos(theta)
                ry = world.orbit_b * math.sin(theta)
                ox, oy, oz = world.transform_orbit(rx, ry, 0.0)
                px = round(cx + ox)
                py = round(cy + oy / ASPECT)
                if 0 <= px < w and 0 <= py < h:
                    if oz > z_buf[py][px]:
                        frame[py][px] = '.'
                        z_buf[py][px] = oz

            comet_approach = frame_idx / float(COMET_FRAMES)
            warning_glow = max(0.0, (comet_approach - 0.55) / 0.45) ** 2.5

            for y in range(h):
                py = (y - cy) * ASPECT
                for x in range(w):
                    px = x - cx
                    dist_sq = px * px + py * py
                    r_sq = world.r_planet * world.r_planet

                    if dist_sq <= r_sq:
                        pz = math.sqrt(r_sq - dist_sq)
                        edge = math.sqrt(dist_sq) / world.r_planet

                        if edge > 0.90:
                            ang = math.atan2(py, px)
                            if -math.pi/8 <= ang < math.pi/8 or ang >= 7*math.pi/8 or ang < -7*math.pi/8:
                                ch = '|'
                            elif math.pi/8 <= ang < 3*math.pi/8 or -7*math.pi/8 <= ang < -5*math.pi/8:
                                ch = '/'
                            elif 3*math.pi/8 <= ang < 5*math.pi/8 or -5*math.pi/8 <= ang < -3*math.pi/8:
                                ch = '-'
                            else:
                                ch = '\\'
                        else:
                            ch = world.get_planet_char(px, py, pz, planet_spin)
                            if warning_glow > 0.05:
                                d_imp = math.sqrt((px - impact_px)**2 + (py - impact_py)**2 + (pz - impact_pz)**2)
                                if d_imp < world.r_planet * 0.75:
                                    glow_f = (1.0 - d_imp / (world.r_planet * 0.75)) * warning_glow
                                    if glow_f > 0.65:
                                        ch = random.choice(['#', '@', '%', '*'])
                                    elif glow_f > 0.35:
                                        ch = random.choice(['*', 'O', '+'])

                        frame[y][x] = ch
                        z_buf[y][x] = pz

            sat_rx = world.orbit_a * math.cos(t)
            sat_ry = world.orbit_b * math.sin(t)
            sx, sy, sz = world.transform_orbit(sat_rx, sat_ry, 0.0)
            sat_curr_pos = [sx, sy, sz]

            vel_rx = -world.orbit_a * math.sin(t) * direction
            vel_ry = world.orbit_b * math.cos(t) * direction
            vx, vy, vz = world.transform_orbit(vel_rx, vel_ry, 0.0)
            v_len = math.sqrt(vx*vx + vy*vy + vz*vz) + 1e-6
            tx, ty, tz = vx/v_len, vy/v_len, vz/v_len

            norm_x, norm_y, norm_z = world.transform_orbit(0.0, 0.0, 1.0)
            wx = ty * norm_z - tz * norm_y
            wy = tz * norm_x - tx * norm_z
            wz = tx * norm_y - ty * norm_x

            p_len = math.sqrt(sx*sx + sy*sy + sz*sz) + 1e-6
            nx, ny, nz = -sx/p_len, -sy/p_len, -sz/p_len

            base_px = cx + sx
            base_py = cy + sy / ASPECT

            all_sat_nodes = body_nodes + left_wing_nodes + right_wing_nodes
            for glyph, (dt, dnorm, dw, dnadir) in all_sat_nodes:
                local_x = (dt * tx) + (dnorm * norm_x) + (dw * wx) + (dnadir * nx)
                local_y = ((dt * ty) + (dnorm * norm_y) + (dw * wy) + (dnadir * ny)) / ASPECT
                pz_3d = sz + (dt * tz) + (dnorm * norm_z) + (dw * wz) + (dnadir * nz)

                screen_x = round(base_px + local_x)
                screen_y = round(base_py + local_y)

                if 0 <= screen_x < w and 0 <= screen_y < h:
                    if pz_3d > z_buf[screen_y][screen_x]:
                        frame[screen_y][screen_x] = glyph
                        z_buf[screen_y][screen_x] = pz_3d

            prog = (frame_idx / float(COMET_FRAMES)) ** 2.05
            cur_cx = comet_start_x + (impact_px - comet_start_x) * prog
            cur_cy = comet_start_y + (impact_py - comet_start_y) * prog
            cur_cz = comet_start_z + (impact_pz - comet_start_z) * prog

            comet_trail.append((cur_cx, cur_cy, cur_cz))
            trail_len = max(6, int(12 * scale))
            if len(comet_trail) > trail_len:
                comet_trail.pop(0)

            tail_ramp = ['@', '%', '#', '*', '+', '-', '.', "'"]
            for tidx, (tx_c, ty_c, tz_c) in enumerate(reversed(comet_trail[:-1])):
                tail_px = round(cx + tx_c + random.uniform(-0.3, 0.3) * scale)
                tail_py = round(cy + ty_c / ASPECT + random.uniform(-0.2, 0.2) * scale)
                ramp_idx = min(len(tail_ramp) - 1, int((tidx / float(len(comet_trail))) * len(tail_ramp)))
                tail_char = tail_ramp[ramp_idx]

                if 0 <= tail_px < w and 0 <= tail_py < h:
                    if tz_c > z_buf[tail_py][tail_px]:
                        frame[tail_py][tail_px] = tail_char
                        z_buf[tail_py][tail_px] = tz_c

            head_screen_x = round(cx + cur_cx)
            head_screen_y = round(cy + cur_cy / ASPECT)
            if 0 <= head_screen_x < w and 0 <= head_screen_y < h:
                frame[head_screen_y][head_screen_x] = random.choice(['@', '0', '*'])
                z_buf[head_screen_y][head_screen_x] = cur_cz + 2.0

            t += world.orbit_speed

        else:
            rel_f = frame_idx - COMET_FRAMES

            if rel_f == 0:
                num_fragments = max(60, int(140 * scale))
                phi = math.pi * (math.sqrt(5.0) - 1.0)
                for i in range(num_fragments):
                    y_norm = 1.0 - (i / float(num_fragments - 1)) * 2.0
                    rad = math.sqrt(max(0.0, 1.0 - y_norm * y_norm))
                    theta = phi * i
                    x_norm = math.cos(theta) * rad
                    z_norm = math.sin(theta) * rad

                    depth = random.uniform(0.3, 1.0)
                    px = x_norm * world.r_planet * depth
                    py = y_norm * world.r_planet * depth
                    pz = z_norm * world.r_planet * depth

                    d_to_imp = math.sqrt((px - impact_px)**2 + (py - impact_py)**2 + (pz - impact_pz)**2) + 0.1
                    imp_dir_x = (px - impact_px) / d_to_imp
                    imp_dir_y = (py - impact_py) / d_to_imp
                    imp_dir_z = (pz - impact_pz) / d_to_imp

                    p_norm_len = math.sqrt(px*px + py*py + pz*pz) + 0.1
                    rad_dir_x = px / p_norm_len
                    rad_dir_y = py / p_norm_len
                    rad_dir_z = pz / p_norm_len

                    blast_intensity = (1.4 / (1.0 + d_to_imp * 0.12)) * random.uniform(0.6, 1.8) * scale
                    vx_f = (imp_dir_x * 0.65 + rad_dir_x * 0.35 + random.uniform(-0.25, 0.25)) * blast_intensity
                    vy_f = (imp_dir_y * 0.65 + rad_dir_y * 0.35 + random.uniform(-0.25, 0.25)) * blast_intensity
                    vz_f = (imp_dir_z * 0.65 + rad_dir_z * 0.35 + random.uniform(-0.25, 0.25)) * blast_intensity

                    init_char = world.get_planet_char(px, py, pz, planet_spin)
                    if init_char == ' ':
                        init_char = random.choice(['#', '%', '8', '*'])

                    planet_fragments.append({
                        'pos': [px, py, pz],
                        'vel': [vx_f, vy_f, vz_f],
                        'char': init_char,
                        'decay': random.uniform(0.015, 0.035),
                        'life': 1.0,
                    })

                num_plasma = max(15, int(35 * scale))
                for _ in range(num_plasma):
                    speed = random.uniform(1.2, 2.5) * scale
                    ang_th = random.uniform(0, math.pi * 2)
                    ang_ph = random.uniform(-math.pi/2, math.pi/2)
                    plasma_particles.append({
                        'pos': [impact_px, impact_py, impact_pz],
                        'vel': [
                            math.cos(ang_th) * math.cos(ang_ph) * speed,
                            math.sin(ang_ph) * speed,
                            math.sin(ang_th) * math.cos(ang_ph) * speed
                        ],
                        'life': 1.0,
                        'decay': random.uniform(0.04, 0.09)
                    })

                num_orbit_debris = max(25, int(60 * scale))
                for i in range(num_orbit_debris):
                    if i % 2 != 0:
                        continue
                    theta = (i / float(num_orbit_debris)) * math.pi * 2
                    rx = world.orbit_a * math.cos(theta)
                    ry = world.orbit_b * math.sin(theta)
                    ox, oy, oz = world.transform_orbit(rx, ry, 0.0)
                    r_dist = math.sqrt(ox*ox + oy*oy + oz*oz) + 0.1
                    speed = random.uniform(0.3, 0.9) * scale
                    orbit_debris.append({
                        'pos': [ox, oy, oz],
                        'vel': [
                            (ox / r_dist) * speed,
                            (oy / r_dist) * speed,
                            (oz / r_dist) * speed,
                        ],
                        'life': 1.0,
                        'decay': random.uniform(0.05, 0.11)
                    })

                dx = sat_curr_pos[0] - impact_px
                dy = sat_curr_pos[1] - impact_py
                dz = sat_curr_pos[2] - impact_pz
                d_blast = math.sqrt(dx*dx + dy*dy + dz*dz) + 1e-4

                kick = random.uniform(2.0, 2.8) * scale
                sat_vel = [
                    init_orb_vx * 0.4 + (dx / d_blast) * kick,
                    init_orb_vy * 0.4 + (dy / d_blast) * kick,
                    init_orb_vz * 0.4 + (dz / d_blast) * kick
                ]
                sat_rot = [0.0, 0.0, 0.0]
                sat_rot_vel = [
                    random.uniform(0.20, 0.35) * random.choice([-1, 1]),
                    random.uniform(0.22, 0.38) * random.choice([-1, 1]),
                    random.uniform(0.18, 0.30) * random.choice([-1, 1])
                ]

            for od in orbit_debris:
                if od['life'] > 0:
                    od['pos'][0] += od['vel'][0]
                    od['pos'][1] += od['vel'][1]
                    od['pos'][2] += od['vel'][2]
                    od['life'] -= od['decay']
                    px = round(cx + od['pos'][0])
                    py = round(cy + od['pos'][1] / ASPECT)
                    if 0 <= px < w and 0 <= py < h:
                        if od['pos'][2] > z_buf[py][px]:
                            ch = '.' if od['life'] > 0.4 else "'"
                            z_buf[py][px] = od['pos'][2]

            if rel_f < 5:
                flash_r = int((rel_f + 1) * 2.0 * scale)
                for dy_f in range(-flash_r, flash_r + 1):
                    for dx_f in range(-flash_r * 2, flash_r * 2 + 1):
                        fx = round(impact_world_x + dx_f)
                        fy = round(impact_world_y + dy_f)
                        if 0 <= fx < w and 0 <= fy < h:
                            d = math.sqrt((dx_f / 2.0)**2 + dy_f**2)
                            if d <= flash_r:
                                frame[fy][fx] = random.choice(['#', '@', '%', '*'])
                                z_buf[fy][fx] = 90.0

            if 2 <= rel_f < 22:
                sw_radius = (rel_f - 1) * 1.5 * scale
                sw_steps = int(sw_radius * 6)
                for step in range(sw_steps):
                    sw_ang = (step / float(max(1, sw_steps))) * math.pi * 2
                    sw_x = round(impact_world_x + math.cos(sw_ang) * sw_radius * 1.4)
                    sw_y = round(impact_world_y + math.sin(sw_ang) * sw_radius / ASPECT)
                    if 0 <= sw_x < w and 0 <= sw_y < h:
                        if 85.0 > z_buf[sw_y][sw_x]:
                            frame[sw_y][sw_x] = random.choice(['0', '*', '=', '.'])
                            z_buf[sw_y][sw_x] = 85.0

            for p in plasma_particles:
                if p['life'] > 0:
                    p['pos'][0] += p['vel'][0]
                    p['pos'][1] += p['vel'][1]
                    p['pos'][2] += p['vel'][2]
                    p['vel'][0] *= 0.94
                    p['vel'][1] *= 0.94
                    p['vel'][2] *= 0.94
                    p['life'] -= p['decay']
                    px = round(cx + p['pos'][0])
                    py = round(cy + p['pos'][1] / ASPECT)
                    if 0 <= px < w and 0 <= py < h:
                        if p['pos'][2] + 20.0 > z_buf[py][px]:
                            frame[py][px] = random.choice(['@', '*', '+', '.'])
                            z_buf[py][px] = p['pos'][2] + 20.0

            for frag in planet_fragments:
                if frag['life'] > 0:
                    frag['pos'][0] += frag['vel'][0]
                    frag['pos'][1] += frag['vel'][1]
                    frag['pos'][2] += frag['vel'][2]
                    frag['vel'][0] *= 0.99
                    frag['vel'][1] *= 0.99
                    frag['vel'][2] *= 0.99
                    frag['life'] -= frag['decay']
                    px = round(cx + frag['pos'][0])
                    py = round(cy + frag['pos'][1] / ASPECT)
                    if 0 <= px < w and 0 <= py < h:
                        if frag['pos'][2] > z_buf[py][px]:
                            if frag['life'] > 0.6:
                                ch = frag['char']
                            elif frag['life'] > 0.35:
                                ch = random.choice(['*', '+', '='])
                            elif frag['life'] > 0.15:
                                ch = '.'
                            else:
                                ch = "'"
                            frame[py][px] = ch
                            z_buf[py][px] = frag['pos'][2]

            sat_curr_pos[0] += sat_vel[0]
            sat_curr_pos[1] += sat_vel[1]
            sat_curr_pos[2] += sat_vel[2]

            sat_rot[0] += sat_rot_vel[0]
            sat_rot[1] += sat_rot_vel[1]
            sat_rot[2] += sat_rot_vel[2]

            if rel_f >= 5 and not wing1_detached:
                wing1_detached = True
                wing1_pos = [sat_curr_pos[0], sat_curr_pos[1], sat_curr_pos[2]]
                wing1_vel = [sat_vel[0] + random.uniform(0.5, 1.0) * scale, sat_vel[1] + random.uniform(-0.5, 0.5) * scale, sat_vel[2]]
                wing1_rot_vel = [random.uniform(0.3, 0.5) * random.choice([-1, 1])] * 3

            if rel_f >= 8 and not wing2_detached:
                wing2_detached = True
                wing2_pos = [sat_curr_pos[0], sat_curr_pos[1], sat_curr_pos[2]]
                wing2_vel = [sat_vel[0] - random.uniform(0.5, 1.0) * scale, sat_vel[1] + random.uniform(-0.5, 0.5) * scale, sat_vel[2]]
                wing2_rot_vel = [random.uniform(0.3, 0.5) * random.choice([-1, 1])] * 3

            if random.random() < 0.75:
                sat_sparks.append({
                    'pos': [sat_curr_pos[0], sat_curr_pos[1], sat_curr_pos[2]],
                    'vel': [random.uniform(-0.2, 0.2) * scale, random.uniform(-0.2, 0.2) * scale, random.uniform(-0.2, 0.2) * scale],
                    'life': 1.0,
                    'decay': random.uniform(0.06, 0.15)
                })

            for sp in sat_sparks:
                if sp['life'] > 0:
                    sp['pos'][0] += sp['vel'][0]
                    sp['pos'][1] += sp['vel'][1]
                    sp['pos'][2] += sp['vel'][2]
                    sp['life'] -= sp['decay']
                    sp_x = round(cx + sp['pos'][0])
                    sp_y = round(cy + sp['pos'][1] / ASPECT)
                    if 0 <= sp_x < w and 0 <= sp_y < h:
                        if sp['pos'][2] > z_buf[sp_y][sp_x]:
                            frame[sp_y][sp_x] = '*' if sp['life'] > 0.5 else '.'
                            z_buf[sp_y][sp_x] = sp['pos'][2]

            rx_rot, ry_rot, rz_rot = sat_rot
            sat_base_px = cx + sat_curr_pos[0]
            sat_base_py = cy + sat_curr_pos[1] / ASPECT
            sat_base_pz = sat_curr_pos[2]

            for glyph, (dt, dnorm, dw, dnadir) in body_nodes:
                lx = (dt * tx) + (dnorm * norm_x) + (dw * wx) + (dnadir * nx)
                ly = (dt * ty) + (dnorm * norm_y) + (dw * wy) + (dnadir * ny)
                lz = (dt * tz) + (dnorm * norm_z) + (dw * wz) + (dnadir * nz)

                rlx, rly, rlz = rotate_3d(lx, ly, lz, rx_rot, ry_rot, rz_rot)
                screen_x = round(sat_base_px + rlx)
                screen_y = round(sat_base_py + rly / ASPECT)
                pz_3d = sat_base_pz + rlz

                if 0 <= screen_x < w and 0 <= screen_y < h:
                    if pz_3d > z_buf[screen_y][screen_x]:
                        frame[screen_y][screen_x] = glyph
                        z_buf[screen_y][screen_x] = pz_3d

            if not wing1_detached:
                w1_base_px, w1_base_py, w1_base_pz = sat_base_px, sat_base_py, sat_base_pz
                w1_rx, w1_ry, w1_rz = rx_rot, ry_rot, rz_rot
            else:
                wing1_pos[0] += wing1_vel[0]
                wing1_pos[1] += wing1_vel[1]
                wing1_pos[2] += wing1_vel[2]
                wing1_rot[0] += wing1_rot_vel[0]
                wing1_rot[1] += wing1_rot_vel[1]
                wing1_rot[2] += wing1_rot_vel[2]
                w1_base_px = cx + wing1_pos[0]
                w1_base_py = cy + wing1_pos[1] / ASPECT
                w1_base_pz = wing1_pos[2]
                w1_rx, w1_ry, w1_rz = wing1_rot

            for glyph, (dt, dnorm, dw, dnadir) in left_wing_nodes:
                lx = (dt * tx) + (dnorm * norm_x) + (dw * wx) + (dnadir * nx)
                ly = (dt * ty) + (dnorm * norm_y) + (dw * wy) + (dnadir * ny)
                lz = (dt * tz) + (dnorm * norm_z) + (dw * wz) + (dnadir * nz)

                rlx, rly, rlz = rotate_3d(lx, ly, lz, w1_rx, w1_ry, w1_rz)
                screen_x = round(w1_base_px + rlx)
                screen_y = round(w1_base_py + rly / ASPECT)
                pz_3d = w1_base_pz + rlz

                if 0 <= screen_x < w and 0 <= screen_y < h:
                    if pz_3d > z_buf[screen_y][screen_x]:
                        frame[screen_y][screen_x] = glyph
                        z_buf[screen_y][screen_x] = pz_3d

            if not wing2_detached:
                w2_base_px, w2_base_py, w2_base_pz = sat_base_px, sat_base_py, sat_base_pz
                w2_rx, w2_ry, w2_rz = rx_rot, ry_rot, rz_rot
            else:
                wing2_pos[0] += wing2_vel[0]
                wing2_pos[1] += wing2_vel[1]
                wing2_pos[2] += wing2_vel[2]
                wing2_rot[0] += wing2_rot_vel[0]
                wing2_rot[1] += wing2_rot_vel[1]
                wing2_rot[2] += wing2_rot_vel[2]
                w2_base_px = cx + wing2_pos[0]
                w2_base_py = cy + wing2_pos[1] / ASPECT
                w2_base_pz = wing2_pos[2]
                w2_rx, w2_ry, w2_rz = wing2_rot

            for glyph, (dt, dnorm, dw, dnadir) in right_wing_nodes:
                lx = (dt * tx) + (dnorm * norm_x) + (dw * wx) + (dnadir * nx)
                ly = (dt * ty) + (dnorm * norm_y) + (dw * wy) + (dnadir * ny)
                lz = (dt * tz) + (dnorm * norm_z) + (dw * wz) + (dnadir * nz)

                rlx, rly, rlz = rotate_3d(lx, ly, lz, w2_rx, w2_ry, w2_rz)
                screen_x = round(w2_base_px + rlx)
                screen_y = round(w2_base_py + rly / ASPECT)
                pz_3d = w2_base_pz + rlz

                if 0 <= screen_x < w and 0 <= screen_y < h:
                    if pz_3d > z_buf[screen_y][screen_x]:
                        frame[screen_y][screen_x] = glyph
                        z_buf[screen_y][screen_x] = pz_3d

        sys.stdout.write("\x1b[H" + "\n".join("".join(row) for row in frame))
        sys.stdout.flush()
        time.sleep(0.016)

def main():
    old_settings = None
    if HAS_TERMIOS and sys.stdin.isatty():
        try:
            old_settings = termios.tcgetattr(sys.stdin)
            tty.setcbreak(sys.stdin.fileno())
        except Exception:
            pass

    sys.stdout.write("\x1b[?25l\x1b[2J")
    sys.stdout.flush()

    world = ProceduralWorld()
    w, h = get_terminal_dimensions()
    world.update_dimensions(w, h)
    last_dims = (w, h)

    t = 0.0
    quit_triggered = False

    try:
        while not quit_triggered:
            curr_w, curr_h = get_terminal_dimensions()
            if (curr_w, curr_h) != last_dims:
                last_dims = (curr_w, curr_h)
                world.update_dimensions(curr_w, curr_h)
                sys.stdout.write("\x1b[2J")

            render_frame(world, t)

            key = check_stdin_key()
            if key:
                if any(c in ('q', 'Q', '\x1b', '\x03', '\x04') for c in key):
                    quit_triggered = True
                    break

            t += world.orbit_speed
            time.sleep(0.016)

        if quit_triggered:
            play_quit_sequence(world, t)

    except KeyboardInterrupt:
        try:
            play_quit_sequence(world, t)
        except KeyboardInterrupt:
            pass
    finally:
        if old_settings is not None and HAS_TERMIOS and sys.stdin.isatty():
            try:
                termios.tcsetattr(sys.stdin, termios.TCSADRAIN, old_settings)
            except Exception:
                pass
        sys.stdout.write("\x1b[?25h\x1b[0m\n")
        sys.stdout.flush()

if __name__ == "__main__":
    main()
# Goodbye World! :D
