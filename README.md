![](https://github.com/joshuardecker/xylok-view/blob/master/showcase.gif?raw=true)

# Xylok View

A fast, cross-platform desktop viewer for DISA Security Technical Implementation Guides (STIGs). Developed by Joshua Decker for Xylok, it is designed to integrate with the Xylok internal suite and serve as a modern alternative to the official DISA STIG Viewer.

The project is currently in beta, but has reached a stable, usable state for daily workflows.

## Supported Formats

Xylok View can load and display benchmarks from multiple sources:

- **XCCDF v1.1** — The standard DISA STIG XML format.
- **CKL** — DISA checklist XML files.
- **CKLB** — The JSON-based successor to CKL.
- **Xylok packed TOML** — An internal Xylok format.

## Technical Highlights

- **Cross-platform native UI** built with [Iced](https://iced.rs/), a Rust GUI framework that renders on Linux (Wayland/X11), macOS, and Windows from a single codebase.
- **Multi-format parsing pipeline** that auto-detects and loads XCCDF, CKL, CKLB, and custom Xylok TOML benchmarks without manual intervention.
- **Benchmark Cache and Compression** that uses zstd compression, greatly shrinking large benchmark disk footprints.
- **Deployed to Every Platform** using standard formats for installation.

## Build & Run

Prerequisites: [Rust](https://rustup.rs/) toolchain (latest stable)

```bash
cargo run --release
```

## Contributing

This software is in active development. If you have suggestions or find a bug, feel free to open an issue or reach out.
