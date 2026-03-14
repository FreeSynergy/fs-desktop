# CLAUDE.md – FreeSynergy.Desktop

## What is this?

FreeSynergy.Desktop – a KDE-like desktop environment for the FreeSynergy ecosystem.
Built with Dioxus (Desktop + Web + TUI from a single codebase).

Each `fsd-*` crate can run as a standalone window or embedded in the desktop shell.

## Rules

- Language in files: **English** (comments, code, variable names)
- Language in chat: **German**
- OOP everywhere: traits over match blocks, types carry their own behavior
- No CHANGELOG.md (removed for token savings)
- After every feature: commit directly

## Repository Structure

```
crates/
  fsd-shell/      → Desktop shell: taskbar, window manager, wallpaper
  fsd-conductor/  → Container/Service/Bot management (formerly "Admin")
  fsd-store/      → Package manager (discovery, install, updates)
  fsd-studio/     → Plugin/Module/i18n creator (+AI optional)
  fsd-settings/   → System settings (appearance, language, service roles)
  fsd-profile/    → User profile
  fsd-app/        → Main launcher binary
```

## Library Dependencies (FreeSynergy.Lib)

All shared libraries live in `../FreeSynergy.Lib/`. Never duplicate their logic here.

| Library         | Purpose |
|---|---|
| `fsn-types`     | Resource/Capability traits, Meta, TypeRegistry |
| `fsn-error`     | FsnError, Repairable trait, ValidationIssue |
| `fsn-config`    | TOML loader/saver with backup + auto-repair |
| `fsn-i18n`      | Snippet-based i18n (t(), t_with()) |
| `fsn-theme`     | Theme system (theme.toml → TUI palette + CSS) |
| `fsn-help`      | Context-sensitive help topics |
| `fsn-health`    | Generic health check framework |
| `fsn-store`     | Universal store client |
| `fsn-container` | Container abstraction (Podman via bollard) |
| `fsn-plugin-sdk` | WASM plugin SDK |
| `fsn-auth`      | OAuth2 + JWT + Permissions |
| `fsn-core`      | FormAction, SelectionResult |

## Architecture

Desktop communicates with FreeSynergy.Node via:
- Direct library calls to `fsn-container` (Lib) for container operations
- `fsn` CLI subprocess for Node-specific operations
- Local SQLite database (shared, accessed via `fsn-db`)

Desktop does NOT import Node-internal crates directly.

## Window System

All dialogs and views are `Window` objects. See `fsd-shell/src/window.rs`.

## Service Roles

Extended MIME-type system for functions (not files). See `fsd-settings/src/service_roles.rs`.
Example: `auth = "kanidm"`, `mail = "stalwart"`, `git = "forgejo"`.

## CSS Variables Prefix

Always `--fsn-` (e.g., `--fsn-color-primary`, `--fsn-font-family`).

## Branding

- "by KalEl" in header
- Cyan + White for FreeSynergy.Node colors
