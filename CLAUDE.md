# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

BattleNetManager is a Windows desktop application for managing multiple Battle.net accounts. It saves and switches between Battle.net local login configurations so users can quickly switch accounts without re-entering credentials. The UI and documentation are in Chinese (Simplified); code uses English identifiers.

## Architecture

Two-part hybrid application where a SolidJS frontend runs inside a Tauri 2.0 shell:

- **`webui/`** — SolidJS frontend built with Vite. The main UI is in `src/App.tsx` with components in `src/components/` (`Header`, `Sidebar`, `AccountGrid`, `AccountCard`, `AccountModal`, `UpdateModal`). Styling uses Tailwind CSS v4 (Vite plugin mode) with custom theme tokens and dark mode support defined in `src/app.css`. Icons come from `lucide-solid`. The `src/bridge.ts` file provides a `MockBridge` for standalone browser development and a `TauriBridge` for the Tauri desktop environment. Communication with the Rust backend uses Tauri's `invoke()` IPC. Shared TypeScript types live in `src/types.ts`.

- **`src-tauri/`** — Tauri 2.0 Rust backend. `src/main.rs` is the binary entry point; it imports from `src/lib.rs` (`battle_net_manager_lib` crate) which re-exports `commands` and `BattleNetCore`. `main.rs` sets up the app with system tray, close-to-tray behavior, window management, and the `tauri-plugin-updater` plugin. `src/commands.rs` exposes Tauri commands callable from the frontend. `src/battle_net/` contains all business logic split across submodules:
  - `core.rs` — `BattleNetCore` struct: account/group CRUD, account switching, legacy data migration, region-aware session management orchestration.
  - `models.rs` — Serde data models (`AccountInfo`, `GroupInfo`, result types, migration state, registry backup models). Uses PascalCase serde renames to match the legacy C# data format.
  - `config.rs` — Battle.net config file manipulation (new account login configs, cross-region login configs with auth field stripping).
  - `process.rs` — Battle.net process enumeration (via `tasklist`), graceful close (via PowerShell `WM_CLOSE`), force kill (via `taskkill`), launch, and exit waiting.
  - `region.rs` — Region constants, normalization, inference from `Battle.net.config` JSON, cross-region detection, and per-region settings (login addresses, allowed regions/locales).
  - `session_state.rs` — Session snapshot save/restore: captures local state files (`Account`, `BrowserCaches`, `Cache`, `CachedData.db`) and registry auth values into an atomic candidate → commit directory structure.
  - `registry.rs` — Windows Registry operations: Battle.net install path lookup, auth state snapshot save/restore/clear, raw registry value serialization/deserialization (base64 for binary, UTF-16 for strings), backup pruning.
  - `mod.rs` — Module declarations and `BattleNetCore` re-export.

- **`src-tauri/build.rs`** — Standard `tauri_build::build()` call.
- **`src-tauri/capabilities/default.json`** — Tauri 2.0 capability file granting the `main` window permissions for core operations, shell open, and window management (drag, close, minimize, hide, show, focus, center, unminimize).

User data lives at `%LOCALAPPDATA%\BattleNetManager\Data` (e.g. `accounts.json`, `groups.json`, `migration-state.json`, per-account directories containing `Battle.net.config` and `BattleNetSessionState/`). The Battle.net client config is read from `%APPDATA%\Battle.net\Battle.net.config`. Local state backups and registry backups are stored under `%LOCALAPPDATA%\BattleNetManager\BattleNetLocalStateBackups\` and `%LOCALAPPDATA%\BattleNetManager\BattleNetRegistryBackups\` respectively (pruned to 3 most recent).

## Region System

Accounts are tagged with a region: `cn`, `asia`, `americas`, `europe`, or empty (unset). The region is inferred from the saved `Battle.net.config` by recursively scanning for fields containing "Region" in their key name, with priority: `AllowedRegions` > `LastLoginRegion`/`WebRegion` > any other region field. Battle.net region codes are mapped as: `CN`→`cn`, `US`→`americas`, `EU`→`europe`, `KR`/`TW`/`SG`→`asia`.

Cross-region switches (e.g. CN→Asia) trigger different session state handling than same-region switches. When switching to a tagged-region account with existing session state, the saved snapshot is restored. For cross-region switches without saved state, a clean login config is written (auth fields stripped, region settings applied). Untagged-region accounts skip all session state logic and use a fast kill-and-replace path. The default region for new accounts and for normalizing old data is `cn`.

## Session State Management

The `session_state.rs` module implements an atomic snapshot system for preserving Battle.net login sessions across account switches:

- **Snapshot structure**: Each account directory contains a `BattleNetSessionState/` folder with `Local/` (copies of `Account`, `BrowserCaches`, `Cache`, `CachedData.db` from `%LOCALAPPDATA%\Battle.net\`) and `Registry/registry-values.json` (serialized auth registry values from `HKCU\Software\Blizzard Entertainment\Battle.net\` subkeys: `UnifiedAuth`, `Identity`, `EncryptionKey`, and `Launch Options\URI_TOKEN`).
- **Atomic commit**: Snapshots are written to a `BattleNetSessionStateCandidate/` directory first, validated for usability (must contain registry auth data and local state files), then atomically renamed to `BattleNetSessionState/` with backup rotation for rollback.
- **Restore**: Deletes existing local state, copies snapshot files back to `%LOCALAPPDATA%\Battle.net\`, and restores registry values.
- **Usage**: Triggered during `save_current_account_to_group`, `switch_account` (for the leaving account and target account), and `refresh_account_session_state`.

## Auto-Update System

The app includes built-in auto-update via `tauri-plugin-updater`:

- **Signing keys**: `src-tauri/updater.key` (private) and `updater.key.pub` (public) are used to sign/verify update bundles.
- **Update endpoint**: Configured in `tauri.conf.json` under `plugins.updater`, pointing to `https://github.com/track23/battle-net-manager/releases/latest/download/latest.json`.
- **Commands**: `check_update` returns `UpdateInfo` (version, notes, date) or `None`; `install_update` downloads and installs the update.
- **Frontend**: On startup, `App.tsx` calls `checkForUpdate()` which queries the bridge. If an update is available, `UpdateModal.tsx` displays version info, release notes, and a download/install button.

## i18n

The frontend uses a custom i18n system in `webui/src/i18n/`. Locale is detected in the frontend via `navigator.language` — Chinese for any `zh`-prefixed locale, English otherwise. Translations live in `webui/src/i18n/locales/zh.ts` and `en.ts`. The `I18nProvider` wraps the app and exposes a `t(key)` function through `useI18n()`. On the Rust side, `sys-locale` is used in `main.rs` only to set the tray menu text, window title, and tooltip in the appropriate language.

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

The `beforeDevCommand` and `beforeBuildCommand` in `tauri.conf.json` automatically build the SolidJS frontend. Build output is at `src-tauri/target/release/bundle/`. The bundle target is NSIS only. The release profile uses `lto = "thin"`, `opt-level = "s"`, `strip = true`, and `panic = "abort"`.

## Development Workflow

Frontend-only development: run `npm run dev` in `webui/` and open in a browser. The `MockBridge` in `src/bridge.ts` provides in-memory mock data so the UI works without the desktop shell.

Full-stack development: run `cargo tauri dev` in `src-tauri/`. This starts the Vite dev server and opens the Tauri window.

## Key Patterns

- **Bridge pattern**: Frontend calls Rust commands through `src/bridge.ts` → `@tauri-apps/api/core` `invoke()` → `src/commands.rs` → `src/battle_net/core.rs`. All bridge methods are async from the frontend perspective.
- **Bridge accessor**: `getBridge()` in `src/bridge.ts` auto-detects Tauri vs browser environment (checks `window.__TAURI_INTERNALS__`) and returns the appropriate bridge implementation. Both bridge implementations are singletons.
- **Static build**: Vite produces static HTML/CSS/JS. Tauri serves from `webui/dist/`.
- **Native window decorations**: The window uses native OS decorations (`decorations: true` in `tauri.conf.json`). CSS `.drag-region` / `.no-drag` classes exist for potential custom drag areas. Window dragging uses Tauri's `window.start_dragging()` via the `drag_window` command.
- **Close-to-tray**: Window hides on close, stays in system tray. Tray double-click or context menu to show/exit.
- **Dark mode**: Toggleable dark/light mode persisted in `localStorage`. Implemented via a `dark` class on `<html>` and CSS custom properties in `app.css` using Tailwind's `@custom-variant dark`.
- **JSON string bridge**: Rust commands return JSON strings (not typed objects) to match the original C# bridge behavior. Frontend parses with `JSON.parse()`.
- **JSON field naming**: Rust models use PascalCase serde renames (e.g. `#[serde(rename = "Id")]`) matching the legacy C# data format. Tauri commands use `rename_all = "camelCase"` for JS-friendly parameter names.
- **SolidJS reactivity**: Uses `createSignal`, `createMemo`, `createEffect`, and `onMount` for state management. Components receive props and callbacks; no global state store. The `I18nProvider` context is the only app-level context.
- **Path alias**: Vite config maps `~` to `/src` (e.g., `import { x } from "~/lib/utils"`).
- **Fixed dev port**: Vite dev server runs on port `1420` (configured in both `vite.config.ts` and `tauri.conf.json` `devUrl`). Must match for Tauri dev to work.
- **Icon library**: `lucide-solid` provides all UI icons (search, menu, edit, delete, etc.).
- **External URL safety**: The `open_external_url` command only opens URLs with `https`/`http` scheme and `github.com` host, using the `opener` crate.

## Requirements

- Node.js 20+
- Rust 1.70+ (with MSVC toolchain on Windows)
- Windows x64 only

## No Tests or CI

There are no test suites or CI/CD pipelines in this project.
