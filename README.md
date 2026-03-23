# Mandelbulb

A ray-marched Mandelbulb fractal renderer written in Rust from scratch.

![Mandelbulb render](mandelbulb.png)

## Overview

This program renders a 3D Mandelbulb fractal using sphere tracing (ray marching) with a distance estimator derived from the Green's function of the Mandelbrot iteration extended to three dimensions.

### How it works

1. **Camera setup** — A virtual camera is positioned in 3D space looking at the origin, with configurable field of view.
2. **Ray marching** — For each pixel, a ray is cast into the scene. The ray advances in steps, where each step size is determined by the distance estimator.
3. **Distance estimator** — At each point along the ray, the Mandelbulb iteration `z_{n+1} = z^d + c` is run using spherical coordinates to compute the 3D "power" operation. The running derivative `dr` tracks orbit expansion, yielding the estimate `0.5 * r * ln(r) / dr`.
4. **Surface normals** — Estimated via central differences of the distance estimator (6 DE evaluations per hit).
5. **Shading** — Lambertian diffuse lighting with position-based rainbow coloring that wraps around the vertical axis, revealing the fractal's rotational symmetry.

## Building and running

Requires Rust and Cargo.

```bash
cargo run --release
```

**Always use `--release`** — debug mode is 10-50x slower due to unoptimized floating-point math.

The output is saved as `mandelbulb.png` in the project directory.

## Configuration

All rendering parameters are in `src/config.rs`:

| Parameter | Default | Description |
|-----------|---------|-------------|
| `EYE` | (2.5, 2.5, 2.5) | Camera position |
| `TARGET` | (0, 0, 0) | Look-at point |
| `FOV` | π/4 | Field of view |
| `IMG_WIDTH` | 800 | Image width in pixels |
| `IMG_HGT` | 600 | Image height in pixels |
| `POWER` | 8.0 | Mandelbulb exponent |
| `MAX_ITER` | 20 | DE iteration limit |
| `BAILOUT` | 2.0 | Orbit escape radius |
| `MAX_STEPS` | 500 | Ray march step limit |
| `SURFACE_EPS` | 0.0001 | Surface hit threshold |
| `MAX_DIST` | 10.0 | Max ray travel distance |
| `LIGHT_POS` | (2, 4, 3) | Light source position |

## Project structure

```
src/
  main.rs          Entry point
  config.rs        Rendering parameters
  mandelbulb.rs    Ray marcher, DE, shading, and image output
  point3d.rs       3D vector type with arithmetic operators
```

## Dependencies

- [image](https://crates.io/crates/image) — PNG output
