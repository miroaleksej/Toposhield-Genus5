# TopoShield Path Visualization

This repository includes a Python script that **animates the private key path** of the **TopoShield ZKP** signature scheme on the **Poincaré disk model** of hyperbolic geometry. The private key in TopoShield is a reduced word `γ` of length 20 in the fundamental group π₁(ℳ) of a genus‑5 hyperbolic surface. This tool renders that abstract path as a concrete geometric trajectory, making the "topological key" visually tangible.

---

## 🎯 Purpose

- **Educational**: Show how a cryptographic secret lives as a geodesic path in hyperbolic space.
- **Demonstration**: Include in talks, papers, or READMEs to illustrate the geometric nature of TopoShield.
- **Debugging**: Visually verify that generated paths are reduced and non‑trivial.

---

## 📦 Requirements

- Python 3.8+
- `matplotlib`
- `numpy`
- `ffmpeg` (for MP4 animation)

Install dependencies:

```bash
pip install matplotlib numpy
# On Ubuntu/Debian:
sudo apt install ffmpeg
# On macOS (with Homebrew):
brew install ffmpeg
```

---

## 🚀 Quick Start

1. **Generate a witness** (e.g., by running the TopoShield example):

   ```bash
   cargo run --bin prove-example --release
   # This creates witness.json with fields "gamma" and "delta"
   ```

2. **Visualize the private path `gamma`**:

   ```bash
   python3 scripts/visualize_path.py witness.json
   ```

3. **Outputs**:
   - `path_on_poincare_disk.png` — static image of the full path.
   - `path_animation.mp4` — animated construction of the path (1 frame per step).

---

## 📁 File Structure

```
toposhield/
├── scripts/
│   └── visualize_path.py      # Main visualization script
├── witness.json               # Example witness (output from prove-example)
└── ...
```

The script expects a JSON file containing a field `"gamma"` that is a list of 20 integers in the range `[0, 19]`, matching the generator indices used in `holonomy_path_enhanced.circom`.

Example `witness.json` snippet:
```json
{
  "gamma": [0, 5, 1, 6, 2, 7, 3, 8, 4, 9, 10, 15, 11, 16, 12, 17, 13, 18, 14, 19],
  "delta": [ ... ]
}
```

If no file is provided, the script uses a built-in test path.

---

## 🔧 How It Works

1. **Generators**: The script embeds the same 20 SL(2, ℝ) matrices used in `manifold.rs` (10 generators + 10 inverses).
2. **Action on ℍ**: Starting from the point `i` in the upper half-plane ℍ, it applies each Möbius transformation corresponding to the generator index.
3. **Projection**: The resulting sequence of points in ℍ is conformally mapped to the Poincaré disk via  
   \[
   z \mapsto \frac{z - i}{z + i}.
   \]
4. **Geodesics**: Between consecutive points, the script draws the unique hyperbolic geodesic (either a circular arc orthogonal to the boundary or a diameter).
5. **Animation**: Using `matplotlib.animation`, it renders a frame-by-frame buildup of the path.

---

## 🖼️ Output Examples

### Static Image (`path_on_poincare_disk.png`)
![Static path on Poincaré disk](path_on_poincare_disk.png)

- **Red dots**: vertices of the path.
- **Blue curves**: hyperbolic geodesic segments.
- **Green dot**: origin (reference point).

### Animation (`path_animation.mp4`)
A 10‑second video showing the path being traced step by step. Ideal for presentations.

---

## 🛠 Customization

You can modify the script to:
- Change the starting point (default: `i` in ℍ).
- Adjust animation speed (`interval=500` ms per frame).
- Export to GIF (requires `pillow`):
  ```python
  ani.save("path_animation.gif", writer="pillow", fps=2)
  ```

---

## 📚 Theory Behind the Visualization

In TopoShield:
- The **private key** is a word `γ = g₁g₂…g₂₀` in π₁(ℳ), where each `gᵢ ∈ {a₁,…,b₅, a₁⁻¹,…,b₅⁻¹}`.
- The **holonomy representation** maps each generator to an SL(2, ℝ) matrix acting on ℍ by Möbius transformations.
- The **path in ℍ** (and thus in the Poincaré disk) is the orbit of the basepoint under this action.
- **Security** relies on the fact that, given only the final holonomy matrix `Hol(γ)`, reconstructing the original path `γ` is equivalent to solving the isomorphism problem for hyperbolic surfaces — an NP‑hard task.

This visualization **makes that orbit visible**.

---

## 📝 License

MIT License. See `LICENSE` for details.

---

> **"Your private key is not a number — it’s a journey through hyperbolic space."**  
> — TopoShield Philosophy
