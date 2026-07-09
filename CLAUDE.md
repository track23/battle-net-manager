# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

BattleNetManager is a Windows desktop application for managing multiple Battle.net accounts. It saves and switches between Battle.net local login configurations so users can quickly switch accounts without re-entering credentials. The UI and documentation are in Chinese (Simplified); code uses English identifiers.

## Architecture

Two-part hybrid application where a SolidJS frontend runs inside a Tauri 2.0 shell:

- **`webui/`** — SolidJS frontend built with Vite. The main UI is in `src/App.tsx` with components in `src/components/`. Styling uses Tailwind CSS v4 (Vite plugin mode). The `src/bridge.ts` file provides a `MockBridge` for standalone browser development and a `TauriBridge` for the Tauri desktop environment. Communication with the Rust backend uses Tauri's `invoke()` IPC.

- **`src-tauri/`** — Tauri 2.0 Rust backend. `src/main.rs` sets up the app with system tray, close-to-tray behavior, and window management. `src/commands.rs` exposes Tauri commands callable from the frontend. `src/battle_net/` contains all business logic (account/group CRUD, Battle.net process management, Windows Registry auto-start, legacy data migration).

User data lives at `%LOCALAPPDATA%\BattleNetManager\Data` (e.g. `accounts.json`, `groups.json`, per-account `Battle.net.config` files). The Battle.net client config is read from `%APPDATA%\Battle.net\Battle.net.config`.

## Region System

Accounts are tagged with a region: `cn`, `asia`, `americas`, `europe`, or empty (unset). The region is inferred from the saved `Battle.net.config` by scanning for fields like `AllowedRegions`, `LastLoginRegion`, and `WebRegion`. Cross-region switches (e.g. CN→Asia) trigger different session state handling than same-region switches. Default region for new accounts is `asia`.

## Build Commands

### Frontend (webui)

```bash
cd webui
npm install
npm run dev        # dev server with HMR
npm run build      # production build to dist/
npm run preview    # preview production build
```

### Tauri App (src-tauri)

```bash
cd src-tauri
cargo tauri dev    # run with frontend dev server (hot reload)
cargo tauri build  # production build with installer
```

The `beforeDevCommand` and `beforeBuildCommand` in `tauri.conf.json` automatically build the SolidJS frontend. Build output is at `src-tauri/target/release/bundle/`.

## Development Workflow

Frontend-only development: run `npm run dev` in `webui/` and open in a browser. The `MockBridge` in `src/bridge.ts` provides in-memory mock data so the UI works without the desktop shell.

Full-stack development: run `cargo tauri dev` in `src-tauri/`. This starts the Vite dev server and opens the Tauri window.

## Key Patterns

- **Bridge pattern**: Frontend calls Rust commands through `src/bridge.ts` → `@tauri-apps/api/core` `invoke()` → `src/commands.rs` → `src/battle_net/core.rs`. All bridge methods are async from the frontend perspective.
- **Bridge accessor**: `getBridge()` in `src/bridge.ts` auto-detects Tauri vs browser environment and returns the appropriate bridge implementation.
- **Static build**: Vite produces static HTML/CSS/JS. Tauri serves from `webui/dist/`.
- **Borderless window**: Custom title bar with drag region, custom minimize/close buttons. Window dragging uses Tauri's `window.start_dragging()`.
- **Close-to-tray**: Window hides on close, stays in system tray. Tray double-click or context menu to show/exit.
- **JSON string bridge**: Rust commands return JSON strings (not typed objects) to match the original C# bridge behavior. Frontend parses with `JSON.parse()`.
- **JSON field naming**: Rust models use PascalCase serde renames (e.g. `#[serde(rename = "Id")]`) matching the legacy C# data format. Tauri commands use `rename_all = "camelCase"` for JS-friendly parameter names.
- **SolidJS reactivity**: Uses `createSignal`, `createMemo`, `createEffect`, and `onMount` for state management. Components receive props and callbacks; no global state store.
- **Path alias**: Vite config maps `~` to `/src` (e.g., `import { x } from "~/lib/utils"`).
- **Fixed dev port**: Vite dev server runs on port `1420` (configured in both `vite.config.ts` and `tauri.conf.json` `devUrl`). Must match for Tauri dev to work.

## Requirements

- Node.js 20+
- Rust 1.70+ (with MSVC toolchain on Windows)
- Windows x64 only

## No Tests or CI

There are no test suites or CI/CD pipelines in this project.
