#!/usr/bin/env python3
# Animate TopoShield private key (gamma path) on the Poincaré disk
# Requires: matplotlib, numpy, ffmpeg

import numpy as np
import matplotlib.pyplot as plt
import matplotlib.animation as animation
from matplotlib.patches import Circle
import json
import sys
import os

# Генераторы SL(2,R) для genus=5 (только A1–B5, как в manifold.rs)
GENERATORS_UPPER = [
    np.array([[2, 1], [1, 1]], dtype=float),
    np.array([[3, 2], [1, 1]], dtype=float),
    np.array([[5, 3], [2, 1]], dtype=float),
    np.array([[7, 4], [3, 2]], dtype=float),
    np.array([[11, 7], [4, 3]], dtype=float),
    np.array([[13, 8], [5, 3]], dtype=float),
    np.array([[17, 11], [7, 4]], dtype=float),
    np.array([[19, 12], [8, 5]], dtype=float),
    np.array([[23, 14], [9, 6]], dtype=float),
    np.array([[21, 13], [8, 5]], dtype=float),
]

def invert_matrix(M):
    a, b = M[0, 0], M[0, 1]
    c, d = M[1, 0], M[1, 1]
    return np.array([[d, -b], [-c, a]], dtype=float)

# Строим полный набор из 20 генераторов (0–19)
GENERATORS = []
for i in range(10):
    GENERATORS.append(GENERATORS_UPPER[i])
for i in range(10):
    GENERATORS.append(invert_matrix(GENERATORS_UPPER[i]))

def upper_halfplane_to_disk(z):
    return (z - 1j) / (z + 1j)

def apply_mobius(M, z):
    a, b, c, d = M[0,0], M[0,1], M[1,0], M[1,1]
    return (a*z + b) / (c*z + d)

def animate_gamma_path(gamma, output_path="path_animation.mp4"):
    # Вычисляем точки в верхней полуплоскости
    current = 1j
    points_upper = [current]
    for idx in gamma:
        M = GENERATORS[idx]
        current = apply_mobius(M, current)
        points_upper.append(current)
    
    # Конвертируем в диск Пуанкаре
    points_disk = []
    for z in points_upper:
        if np.isfinite(z):
            points_disk.append(upper_halfplane_to_disk(z))
        else:
            points_disk.append(complex(0, 0))  # fallback

    # Подготовка фигуры
    fig, ax = plt.subplots(figsize=(8, 8))
    ax.set_aspect('equal')
    ax.set_xlim(-1.1, 1.1)
    ax.set_ylim(-1.1, 1.1)
    ax.axis('off')

    # Граница диска
    boundary = Circle((0, 0), 1, color='black', fill=False, linewidth=1)
    ax.add_patch(boundary)

    # Начальные элементы
    line, = ax.plot([], [], 'b-', linewidth=2, zorder=3)
    point, = ax.plot([], [], 'ro', markersize=6, zorder=4)
    trail, = ax.plot([], [], 'b.', markersize=3, alpha=0.6, zorder=2)

    xs = [z.real for z in points_disk]
    ys = [z.imag for z in points_disk]

    def init():
        line.set_data([], [])
        point.set_data([], [])
        trail.set_data([], [])
        return line, point, trail

    def update(frame):
        if frame == 0:
            trail.set_data([xs[0]], [ys[0]])
            point.set_data([xs[0]], [ys[0]])
            line.set_data([xs[0]], [ys[0]])
        else:
            # Трасса всех предыдущих точек
            trail.set_data(xs[:frame+1], ys[:frame+1])
            # Текущая точка
            point.set_data([xs[frame]], [ys[frame]])
            # Линия до текущей точки
            line.set_data(xs[:frame+1], ys[:frame+1])
        return line, point, trail

    ani = animation.FuncAnimation(
        fig, update, frames=len(xs),
        init_func=init, blit=True, interval=500, repeat=False
    )

    # Сохранение анимации
    ani.save(output_path, writer='ffmpeg', fps=2, dpi=150)
    plt.close(fig)
    print(f"✅ Animation saved to {output_path}")

if __name__ == "__main__":
    if len(sys.argv) > 1:
        with open(sys.argv[1], 'r') as f:
            data = json.load(f)
        gamma = data.get("gamma", data.get("private_key", []))
    else:
        gamma = [0, 5, 1, 6, 2, 7, 3, 8, 4, 9, 10, 15, 11, 16, 12, 17, 13, 18, 14, 19]
    
    animate_gamma_path(gamma)
