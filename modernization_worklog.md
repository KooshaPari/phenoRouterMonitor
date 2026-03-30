# Modernization & Optimization Worklog - 2026-03-29

## Context
Goal: Improve overall User/Developer Experience (UX/DX) on macOS 26 and terminal. Forcible modernization and optimization via package swaps and modern tooling (ripgrep, fd, uv, bun, tsgo, etc.).

## Audit Results (Current State)
- **Shell**: Zsh (with `.zshrc` and `.zshrc.local`)
- **Search**: `rg` installed at `/opt/homebrew/bin/rg`, `fd` at `/opt/homebrew/bin/fd`.
- **Package Managers**: `uv`, `bun`, `brew`.
- **Modern Replacements Found**:
    - `ls` -> `eza` (aliased in `.zshrc.local`)
    - `cat` -> `bat` (aliased in `.zshrc.local`)
    - `cd` -> `zoxide` (initialized in `.zshrc.local`)
    - `find` -> `fd` (suggested in `.zshrc.local` but commented out)
    - `grep` -> `rg` (not aliased)
    - `top` -> `btop` (installed, not aliased)
    - `du` -> `dust` (installed, not aliased)
    - `df` -> `duf` (installed, not aliased)
    - `diff` -> `delta` (installed, usually configured via git)
    - `fzf` installed and initialized.
    - `yazi` (terminal file manager) installed.
    - `atuin` (shell history) installed.
- **Missing/New Targets**:
    - `tsgo`: Found in npm as `@rslint/tsgo` (TypeScript 7 native compiler port). Extremely fast.
    - `fastfetch`: Modern `neofetch` replacement.
    - `bottom` (`btm`): Alternative to `btop`.
    - `gping`: Ping with a graph.
    - `doge`: Modern `dig` replacement (`dog`).
    - `ouch`: Modern compression/decompression tool.
    - `hx` (Helix): Modern editor (not found, though `nvim` is set as `FORGE_EDITOR`).

## Proposed Actions

### 1. Terminal UX/DX Improvements
- [x] Implement `grep='rg'` and `find='fd'` aliases.
- [x] Enable `FORGE_ENABLE_ZSH_RPROMPT_ASYNC=1` for better prompt performance.
- [x] Install and configure `tsgo` for TypeScript projects.
- [x] Install missing modern tools: `fastfetch`, `gping`, `ouch`, `doggo` (or `dog`), `bottom`.
- [x] Configure `atuin` for better history management if not fully active.
- [ ] Configure `yazi` as the default file manager.
- [x] Setup `eza` and `bat` more aggressively.

### 2. System Optimization
- [x] Audit `Brewfile` to ensure all modern tools are tracked.
- [x] Update `uv` and `bun` to latest versions.
- [x] Replace standard `grep`/`find` in scripts where safe.

## Progress Log
- **2026-03-29**: Initial audit and worklog creation. Found `rg`, `fd`, `uv`, `bun` are already present but under-utilized.
- **2026-03-29**: Updated `.zshrc.local` with aliases for `grep`, `find`, `top`, `du`, `df`, `ping`, `dig`, `neofetch`, `tsc`, `pip`, and `venv`.
- **2026-03-29**: Updated `Brewfile` with modern 2026 tools and successfully ran `brew bundle`.
- **2026-03-29**: Installed `tsgo` via npm.
