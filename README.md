# DevForge

DevForge is a privacy-first local desktop developer supertool built on Tauri, Rust, Svelte, and TypeScript.

## Run locally

### Prerequisites

- Node.js 20+
- Rust toolchain with `cargo`
- Tauri system dependencies for your OS

### Install

```bash
npm install
```

### Run the frontend only

```bash
npm run dev
```

This starts the Vite app at `http://localhost:1420`.

### Run the desktop app

```bash
npm run desktop:dev
```

This single command starts the Vite frontend and then launches the Tauri backend/window using the `beforeDevCommand` defined in [src-tauri/tauri.conf.json](C:/Users/katwa/Documents/Code/dev-x/src-tauri/tauri.conf.json).

### Build

```bash
npm run build
npm run tauri build
```

### Notes

- The frontend currently falls back to seeded mock bootstrap data if the Tauri backend is not available.
- On a fresh machine, install the Tauri OS prerequisites before trying `npm run desktop:dev`.

## Current status

This repository now contains the Phase 1 architecture scaffold:

- `src/`: frontend application shell, command palette UI, typed IPC client, and command-state stores
- `src-tauri/src/`: Rust entrypoint, app state, command registry service, and bootstrap IPC command
- `src-tauri/tauri.conf.json`: desktop shell configuration

## Architecture notes

- The frontend is organized around a command-driven shell rather than feature-specific pages.
- Backend responsibilities are split into `commands`, `services`, `state`, and `models` so upcoming utilities can be added as isolated modules.
- `bootstrap_app` now hydrates the shell from SQLite-backed state, including the active workspace profile, persistent app settings, and recent command history.
- The storage boundary lives in `src-tauri/src/services/storage.rs` and initializes schema plus default records on startup.

## Next recommended slices

1. Add persistent storage for settings, history, and profiles.
2. Implement global shortcut and tray lifecycle management.
3. Replace seeded command definitions with a pluggable command/module registry.
