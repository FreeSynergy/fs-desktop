# CLAUDE.md – FreeSynergy.Desktop

## What is this?

FreeSynergy.Desktop – a KDE-like desktop environment for the FreeSynergy ecosystem.
Built with Dioxus (Desktop + Web + TUI from a single codebase).

Each `fs-*` crate can run as a standalone window or embedded in the desktop shell.

## Rules

- Language in files: **English** (comments, code, variable names)
- Language in chat: **German**
- OOP everywhere: traits over match blocks, types carry their own behavior
- No CHANGELOG.md (removed for token savings)
- After every feature: commit directly

## Repository Structure

```
crates/
  fs-gui-workspace/      → Desktop shell: taskbar, window manager, wallpaper
  fs-container-app/ → Container/Service/Bot management (formerly "Conductor")
  fs-store/      → Package manager (discovery, install, updates)
  fs-studio/     → Plugin/Module/i18n creator (+AI optional)
  fs-settings/   → System settings (appearance, language, service roles)
  fs-profile/    → User profile
  fs-app/        → Main launcher binary
```

## Library Dependencies (fs-libs)

All shared libraries live in `../fs-libs/`. Never duplicate their logic here.

| Library         | Purpose |
|---|---|
| `fs-types`     | Resource/Capability traits, Meta, TypeRegistry |
| `fs-error`     | FsError, Repairable trait, ValidationIssue |
| `fs-config`    | TOML loader/saver with backup + auto-repair |
| `fs-i18n`      | Snippet-based i18n (t(), t_with()) |
| `fs-theme`     | Theme system (theme.toml → TUI palette + CSS) |
| `fs-help`      | Context-sensitive help topics |
| `fs-health`    | Generic health check framework |
| `fs-store`     | Universal store client |
| `fs-container` | Container abstraction (Podman via bollard) |
| `fs-plugin-sdk` | WASM plugin SDK |
| `fs-auth`      | OAuth2 + JWT + Permissions |
| `fs-core`      | FormAction, SelectionResult |

## Architecture

Desktop communicates with FreeSynergy.Node via:
- Direct library calls to `fs-container` (Lib) for container operations
- `fsn` CLI subprocess for Node-specific operations
- Local SQLite database (shared, accessed via `fs-db`)

Desktop does NOT import Node-internal crates directly.

## Window System

All dialogs and views are `Window` objects. See `fs-gui-workspace/src/window.rs`.

## Layout System (E1 + E2)

### Shell Layout (fs-gui-workspace)
CSS Grid desktop with four areas: `header | sidebar | main | taskbar`.

```
grid-template-areas: 'header header' 'sidebar main' 'taskbar taskbar';
grid-template-rows:  60px 1fr 48px;
grid-template-columns: {sidebar_width} 1fr;
```

- **ShellHeader** (`header.rs`): Brand + Breadcrumbs + AvatarMenu (60px)
- **ShellSidebar** (`sidebar.rs`): App navigation, collapsible 240px → 48px
- **Taskbar** (`taskbar.rs`): Slots via `TaskbarLauncherBtn`, `TaskbarSeparator`, `TaskbarApps`, `SystemTray`, `Clock`
- **WindowFrame** (`window_frame.rs`): Glassmorphism (backdrop-filter + rgba bg)

### App Layouts (app_shell.rs)
- `AppShell` — root wrapper, injects transition CSS, 3 modes: `Window | Standalone | Tui`
- `ScreenWrapper` — max-width + padding + scroll
- `LayoutA` — full-width column (fs-store, fs-studio)
- `LayoutB` — sidebar (master) + detail pane (fs-container-app, fs-settings)
- `LayoutC` — centered card (fs-profile, login)

Page transitions: `.fs-page-enter` (slideInRight), `.fs-page-fade` (fadeInUp). Respects `prefers-reduced-motion`.

### SplitView (`split_view.rs`)
Resizable horizontal split: `SplitState::Collapsed | Half | FullRight`. Drag handle + double-click to cycle.

## Service Roles

Extended MIME-type system for functions (not files). See `fs-settings/src/service_roles.rs`.
Example: `auth = "kanidm"`, `mail = "stalwart"`, `git = "forgejo"`.

## CSS Variables Prefix

Always `--fs-` (e.g., `--fs-color-primary`, `--fs-font-family`).

## Branding

- "by KalEl" in header
- Cyan + White for FreeSynergy.Node colors
