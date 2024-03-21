<div align="center">

  # [PLAY!](https://wilzet.github.io/fluid-simulation/)
  <a href="https://wilzet.github.io/fluid-simulation/"><img height="100" src="https://github.com/wilzet/fluid-simulation/blob/main/src/app/favicon/android-chrome-512x512.png"></a>
</div>

# Fluid Simulation

<div align="center"></a>

  [![Version](https://img.shields.io/github/package-json/v/wilzet/fluid-simulation.svg?color=blue)]()
  [![MIT License](https://img.shields.io/badge/license-MIT-red.svg)](https://github.com/wilzet/fluid-simulation/LICENSE.md)
  [![Build](https://github.com/wilzet/fluid-simulation/actions/workflows/build.yml/badge.svg)]()
  [![Deployment](https://github.com/wilzet/fluid-simulation/actions/workflows/deploy.yml/badge.svg)](https://wilzet.github.io/fluid-simulation)
</div>

A fluid simulation running in the browser using WebGL and Rust. This project was a learning experience in how to use Rust to write WebAssembly and interact with the WebGL API. Also a lot about GitHub Actions and GitHub Pages was learnt.

> [!NOTE]  
> A simulation quality of "Ultra+" or "Ultra" may significantly slow down mobile devices due to the simulation running at, or nearly at, native resolution. Use these settings with caution.

## Dev
1. Make sure to install the required tools:
   - [The `Rust` toolchain](https://github.com/rust-lang/rustup)
   - [`wasm-pack`](https://github.com/rustwasm/wasm-pack)
   - [`python`](https://www.python.org/)
   - [`Node.js`](https://nodejs.org/en)
2. Clone [this](https://github.com/wilzet/fluid-simulation) git repository.
   ```bash
   git clone https://github.com/wilzet/fluid-simulation
   ```
3. Run the following commands to start a dev server for the project:
   ```bash
   python src/glsl-to-rust-stringify.py   # Generate the `shaders.rs` file
   wasm-pack build                        # Build the fluid simulation package
   npm install                            # Install required npm packages
   npm run dev                            # Start a dev server on localhost:8080
   
   # npm run build   # Make a production build of the project in the /public directory
   ```

## Resources
- [NVIDIA GPU GEMS: Chapter 38. Fast Fluid Dynamics Simulation on the GPU](https://developer.nvidia.com/gpugems/gpugems/part-vi-beyond-triangles/chapter-38-fast-fluid-dynamics-simulation-gpu)
- [WebGL-Fluid-Simulation - PavelDoGreat](https://github.com/PavelDoGreat/WebGL-Fluid-Simulation/)
- [But How DO Fluid Simulations Work? - Gonkee [VIDEO]](https://www.youtube.com/watch?v=qsYE1wMEMPA)
