# AGENTS.md

This file provides guidance to AI coding agents when working with code in this repository.

## Project Overview

Stig View is a Rust desktop application for viewing DISA Security Technical Implementation Guides (STIGs). It supports Xylok packed TOML, XCCDF v1.1, CKL, and CKLB formats.

The goal when using AI to develop this application is **not** for AI to do it alone. Instead AI is used as a tool to improve existing code, and discuss future ideas and how to best implement them.

## Commands

```bash
# Build (release)
cargo build --release -p stig-view-desktop

# Run (development)
cargo run -p stig-view-desktop

# Tests (core library only — no desktop tests)
cargo test -p stig-view-core

# Build all crates
cargo build
```

## Architecture

Cargo workspace with two crates:

- **`core/`** (`stig-view-core`) — GUI-agnostic business logic, kept separate for potential future reuse (e.g. a web frontend).
- **`desktop/`** (`stig-view-desktop`) — Iced desktop application.

## Agent Guidelines

- **Make minimal changes.** Prefer small, focused diffs.
- **Follow existing style.** Match surrounding code formatting and patterns.
- **Do not refactor for the sake of it.** Only change code directly related to the task.
- **Do not add dependencies without a good reason.**
- **Do not add features unless explicitly asked.** Bug fixes and optimizations are welcome when scoped to a specific issue.
- **If a build or test fails after your change, you must fix it before finishing.**
- **When in doubt, ask.** Do not assume intent.

## What AI Should Not Do

- Do not treat this as an AI-only project. All architectural decisions, roadmap, and feature design belong to the human maintainer.
- Do not rewrite entire modules or change public APIs unless specifically requested.
- Do not commit or push code unless explicitly asked.
- Do not add new file formats, parsers, or major UI sections without prior discussion.

## Useful Context

- `core/src/lib.rs` defines the canonical `Benchmark` and `Rule` types.
- `core/src/detection.rs` handles file format sniffing.
- `desktop/src/app/app.rs` contains the Iced update loop and message handling.
- `desktop/src/ui/mod.rs` contains the Iced view code.
- `desktop/src/app/command.rs` implements regex-based rule filtering.
