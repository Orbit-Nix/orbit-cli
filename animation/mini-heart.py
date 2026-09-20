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

        # 2. Heart Parameters
        self.base_scale = random.uniform(5.8, 6.4)
        self.pulse_speed = random.uniform(2.4, 3.0)
        self.pulse_intensity = random.uniform(0.07, 0.10)

        # Sub-cell gentle heart bobbing
        self.bob_x = random.uniform(0.12, 0.20)
        self.bob_y = random.uniform(0.10, 0.16)

        # Orbit Parameters
        self.orbit_a = random.uniform(14.5, 16.5)
        self.orbit_b = random.uniform(4.8, 5.6)
        self.tilt_x = random.uniform(0.28, 0.42)
        self.tilt_z = random.uniform(-0.16, 0.16)

        # Good looking 60fps movement
        self.orbit_speed = random.uniform(0.025, 0.035) * random.choice([-1, 1])

        # Satellite Components
        self.craft_left = "<3"
        self.craft_right = "3>"

    def transform_orbit(self, x, y, z):
        y1 = y * math.cos(self.tilt_x) - z * math.sin(self.tilt_x)
        z1 = y * math.sin(self.tilt_x) + z * math.cos(self.tilt_x)
        x2 = x * math.cos(self.tilt_z) - y1 * math.sin(self.tilt_z)
        y2 = x * math.sin(self.tilt_z) + y1 * math.cos(self.tilt_z)
        return x2, y2, z1

    def heart_function(self, x, y, z):
        a = x*x + 2.25 * y*y + z*z - 1.0
        return (a * a * a) - (x*x * z*z*z) - (0.1125 * y*y * z*z*z)

    def get_heart_surface(self, px, py, scale):
        hx = px / (scale * 1.1)
        hz = -py / (scale * 1.0)

        y_front = -2.0
        y_back = 2.0
        steps = 30
        dy = (y_back - y_front) / steps

        hit_y = None
        y_curr = y_front
        for _ in range(steps):
            if self.heart_function(hx, y_curr, hz) <= 0.0:
                y_lo = y_curr - dy
                y_hi = y_curr
                for _ in range(5):
                    y_mid = (y_lo + y_hi) * 0.5
                    if self.heart_function(hx, y_mid, hz) <= 0.0:
                        y_hi = y_mid
                    else:
                        y_lo = y_mid
                hit_y = (y_lo + y_hi) * 0.5
                break
            y_curr += dy

        if hit_y is None:
            return None, None

        eps = 0.01
        gx = self.heart_function(hx + eps, hit_y, hz) - self.heart_function(hx - eps, hit_y, hz)
        gy = self.heart_function(hx, hit_y + eps, hz) - self.heart_function(hx, hit_y - eps, hz)
        gz = self.heart_function(hx, hit_y, hz + eps) - self.heart_function(hx, hit_y - eps, hz)

        snx = gx
        sny = -gz
        snz = -gy

        n_len = math.sqrt(snx*snx + sny*sny + snz*snz)
        if n_len < 1e-6:
            return None, None

        normal = (snx/n_len, sny/n_len, snz/n_len)
        depth = -hit_y * scale
        return depth, normal

    def purple_gradient(self, intensity, is_highlight=False):
        if is_highlight:
            return (255, 190, 255)
        if intensity < 0.5:
            t = intensity / 0.5
            r = int(90 + (170 - 90) * t)
            g = int(20 + (40 - 20) * t)
            b = int(140 + (235 - 140) * t)
        else:
            t = (intensity - 0.5) / 0.5
            r = int(170 + (240 - 170) * t)
            g = int(40 + (160 - 40) * t)
            b = int(235 + (255 - 235) * t)
        return (r, g, b)

def render_frame(world, t):
    frame = [[' ' for _ in range(WIDTH)] for _ in range(HEIGHT)]
    color_buf = [[None for _ in range(WIDTH)] for _ in range(HEIGHT)]
    z_buf = [[-999.0 for _ in range(WIDTH)] for _ in range(HEIGHT)]

    # floating-point heart center
    cx = WIDTH / 2.0 + math.sin(t * world.bob_x) * 0.4
    cy = HEIGHT / 2.0 + math.cos(t * world.bob_y) * 0.2
    pulse_phase = (t * world.pulse_speed) % (2.0 * math.pi)
    pulse = math.sin(pulse_phase)
    scale_mod = 1.0 + (pulse ** 3) * world.pulse_intensity if pulse > 0 else 1.0
    heart_scale = world.base_scale * scale_mod

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
                color_buf[py][px] = (110, 45, 160) if oz > 0 else (65, 25, 95)
                z_buf[py][px] = oz

    # Heart Mesh
    lx, ly, lz = world.light
    box_w = int(heart_scale * 1.5)
    box_h = int((heart_scale * 1.5) / ASPECT)

    for dy in range(-box_h, box_h + 1):
        y_idx = int(round(cy + dy))
        if not (0 <= y_idx < HEIGHT):
            continue
        py = dy * ASPECT

        for dx in range(-box_w, box_w + 1):
            x_idx = int(round(cx + dx))
            if not (0 <= x_idx < WIDTH):
                continue
            px = dx

            pz, norm = world.get_heart_surface(px, py, heart_scale)
            if pz is not None:
                nx, ny, nz = norm
                dot = max(0.0, nx * lx + ny * ly + nz * lz)

                vx, vy, vz = 0.0, 0.0, 1.0
                hx, hy, hz = lx + vx, ly + vy, lz + vz
                hlen = math.sqrt(hx*hx + hy*hy + hz*hz)
                spec = (max(0.0, (nx*hx + ny*hy + nz*hz) / hlen) ** 7) * 0.35
                rim = ((1.0 - max(0.0, nz)) ** 2) * 0.18

                intensity = min(1.0, dot * 0.75 + spec + rim)
                idx = int(intensity * (len(RAMP_REV) - 1))
                ch = RAMP_REV[max(0, min(len(RAMP_REV) - 1, idx))]

                frame[y_idx][x_idx] = ch
                color_buf[y_idx][x_idx] = world.purple_gradient(intensity, is_highlight=(spec > 0.25))
                z_buf[y_idx][x_idx] = pz

    # Satellite
    sat_rx = world.orbit_a * math.cos(t)
    sat_ry = world.orbit_b * math.sin(t)
    sx, sy, sz = world.transform_orbit(sat_rx, sat_ry, 0.0)

    # Tangent vector
    direction = 1.0 if world.orbit_speed > 0 else -1.0
    vel_rx = -world.orbit_a * math.sin(t) * direction
    vel_ry = world.orbit_b * math.cos(t) * direction
    vx, vy, vz = world.transform_orbit(vel_rx, vel_ry, 0.0)

    moving_left = vx < 0
    craft_str = world.craft_left if moving_left else world.craft_right

    # Calculate exact origin of satellite
    base_px = round(cx + sx)
    base_py = round(cy + sy / ASPECT)

    for offset, char in enumerate(craft_str):
        col = base_px + (offset - 1)
        row = base_py
        if 0 <= col < WIDTH and 0 <= row < HEIGHT:
            if sz > z_buf[row][col]:
                frame[row][col] = char
                color_buf[row][col] = (255, 120, 255)
                z_buf[row][col] = sz

    # Single-write buffer flush
    output = ["\x1b[H"]
    for row in range(HEIGHT):
        line = []
        for col in range(WIDTH):
            ch = frame[row][col]
            rgb = color_buf[row][col]
            if ch != ' ' and rgb is not None:
                r, g, b = rgb
                line.append(f"\x1b[38;2;{r};{g};{b}m{ch}")
            else:
                line.append(' ')
        output.append("".join(line) + "\x1b[0m\n")

    sys.stdout.write("".join(output))
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
        sys.stdout.write("\x1b[?25h\x1b[0m\n")

if __name__ == "__main__":
    main()
# Goodbye World! :D
