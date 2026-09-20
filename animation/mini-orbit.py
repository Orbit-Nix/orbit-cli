import math
import random
import sys
import time

WIDTH = 42
HEIGHT = 18
ASPECT = 2.05

ASCII_RAMP = "$@B%8&WM#*oahkbdpqwmZO0QLCJUYXzcvunxrjft/\\|()1{}[]?-_+~<>i!lI;:,\"^`'. "
RAMP_REV = ASCII_RAMP[::-1]

class ProceduralWorld:
    def __init__(self):
        # 1. Lighting Vector
        lx = random.uniform(-0.85, -0.35)
        ly = random.uniform(-0.55, -0.20)
        lz = random.uniform(0.60, 0.85)
        l_len = math.sqrt(lx*lx + ly*ly + lz*lz)
        self.light = (lx / l_len, ly / l_len, lz / l_len)

        # 2. Planet Parameters
        self.r_planet = random.uniform(5.0, 5.6)
        self.spin_speed = random.uniform(0.12, 0.22) * random.choice([-1, 1])
        self.seed_lat = random.uniform(3.2, 5.8)
        self.seed_lon = random.uniform(3.5, 6.8)
        self.crater_freq = random.uniform(10.0, 15.0)
        
        # Sub-cell gentle planet bobbing
        self.bob_x = random.uniform(0.12, 0.20)
        self.bob_y = random.uniform(0.10, 0.16)

        # Orbit Parameters
        self.orbit_a = random.uniform(15.0, 17.0)
        self.orbit_b = random.uniform(5.0, 5.8)
        self.tilt_x = random.uniform(0.28, 0.44)
        self.tilt_z = random.uniform(-0.16, 0.16)
        
        # Good looking 60fps movement
        self.orbit_speed = random.uniform(0.024, 0.034) * random.choice([-1, 1])

        # Satellite Components
        self.solar_core = random.choice(['#', 'X', 'H', '%'])
        self.bus_core = random.choice(['0', 'O', '8', '@'])

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

def render_frame(world, t):
    frame = [[' ' for _ in range(WIDTH)] for _ in range(HEIGHT)]
    z_buf = [[-999.0 for _ in range(WIDTH)] for _ in range(HEIGHT)]

    # floating-point planet center
    cx = WIDTH / 2.0 + math.sin(t * world.bob_x) * 0.4
    cy = HEIGHT / 2.0 + math.cos(t * world.bob_y) * 0.2
    planet_spin = t * world.spin_speed

    # Orbit Trail
    ORBIT_STEPS = 60
    for i in range(ORBIT_STEPS):
        if i % 2 != 0:
            continue
        theta = (i / ORBIT_STEPS) * math.pi * 2
        rx = world.orbit_a * math.cos(theta)
        ry = world.orbit_b * math.sin(theta)
        ox, oy, oz = world.transform_orbit(rx, ry, 0.0)

        px = round(cx + ox)
        py = round(cy + oy / ASPECT)

        if 0 <= px < WIDTH and 0 <= py < HEIGHT:
            if oz > z_buf[py][px]:
                frame[py][px] = '.'
                z_buf[py][px] = oz

    # Planet Sphere
    for y in range(HEIGHT):
        py = (y - cy) * ASPECT
        for x in range(WIDTH):
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
    v_len = math.sqrt(vx*vx + vy*vy + vz*vz)
    tx, ty, tz = vx/v_len, vy/v_len, vz/v_len

    # Normal & wing vectors
    norm_x, norm_y, norm_z = world.transform_orbit(0.0, 0.0, 1.0)
    wx = ty * norm_z - tz * norm_y
    wy = tz * norm_x - tx * norm_z
    wz = tx * norm_y - ty * norm_x

    # Nadir vector
    p_len = math.sqrt(sx*sx + sy*sy + sz*sz)
    nx, ny, nz = -sx/p_len, -sy/p_len, -sz/p_len

    sc = world.solar_core
    bc = world.bus_core

    # A whole bunch of god knows what
    sat_nodes = [
        ('[',  [-0.6,  0.0,  0.0,  0.0]),
        (bc,   [ 0.0,  0.0,  0.0,  0.0]),
        (']',  [ 0.6,  0.0,  0.0,  0.0]),
        ('=',  [ 0.0,  0.0,  1.1,  0.0]),
        (sc,   [ 0.0,  0.0,  1.9,  0.0]),
        ('=',  [ 0.0,  0.0, -1.1,  0.0]),
        (sc,   [ 0.0,  0.0, -1.9,  0.0]),
        ('v',  [-1.0,  0.0,  0.0,  0.0])
    ]

    # Calculate exact origin of satellite
    base_px = cx + sx
    base_py = cy + sy / ASPECT

    for glyph, (dt, dnorm, dw, dnadir) in sat_nodes:
        # Calculate displacement from base
        local_x = (dt * tx) + (dnorm * norm_x) + (dw * wx) + (dnadir * nx)
        local_y = ((dt * ty) + (dnorm * norm_y) + (dw * wy) + (dnadir * ny)) / ASPECT
        pz_3d = sz + (dt * tz) + (dnorm * norm_z) + (dw * wz) + (dnadir * nz)

        # Unified nearest-neighbor rounding anchored to the craft center
        screen_x = round(base_px + local_x)
        screen_y = round(base_py + local_y)

        if 0 <= screen_x < WIDTH and 0 <= screen_y < HEIGHT:
            if pz_3d > z_buf[screen_y][screen_x]:
                frame[screen_y][screen_x] = glyph
                z_buf[screen_y][screen_x] = pz_3d

    # Single-write buffer flush
    sys.stdout.write("\x1b[H" + "\n".join("".join(row) for row in frame) + "\n")
    sys.stdout.flush()

def main():
    sys.stdout.write("\x1b[?25l\x1b[2J")
    world = ProceduralWorld()
    t = 0.0
    try:
        while True:
            render_frame(world, t)
            t += world.orbit_speed
            time.sleep(0.016)  # 60 FPS goes brrrr
    except KeyboardInterrupt:
        sys.stdout.write("\x1b[?25h\n")

if __name__ == "__main__":
    main()
# Goodbye World! :D
