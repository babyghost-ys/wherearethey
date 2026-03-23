# wherearethey

Find where your CLI tools were installed from.

```
$ wherearethey '*cli*'

  Pattern "*cli*" — 7 matches

  gemini-cli
  source:  npm

  gnutls-cli
  source:  brew
  version: gnutls-cli 3.8.10

  huggingface-cli
  source:  unknown
  version: huggingface_hub version: 0.29.1
  ...
```

Don't remember the exact name? Use wildcards. `wherearethey '*cli*'` searches binary names, package manager tool names, and your aliases all at once.

```
$ wherearethey ffmpeg

  ffmpeg
  path:    /opt/homebrew/bin/ffmpeg
  target:  /opt/homebrew/Cellar/ffmpeg/7.1.1/bin/ffmpeg
  source:  brew
  version: ffmpeg 7.1.1
```

Know the exact name? Get the full breakdown — path, symlink target, source, and version.

---

## Quick start

```sh
# Homebrew (recommended)
brew tap babyghost-ys/tap
brew install wherearethey

# Or from source (requires Rust)
git clone https://github.com/babyghost-ys/wherearethey.git
cd wherearethey && cargo install --path .
```

---

## Usage

```sh
wherearethey ffmpeg              # Look up a single binary
wherearethey '*cli*'             # Wildcard: find all matching tools
wherearethey 'cargo-*'           # Wildcard: all cargo subcommands
wherearethey Gemini              # Look up by friendly name (alias)
wherearethey --all               # List every tool, grouped by source
wherearethey --unmanaged         # Find binaries no package manager claims
wherearethey --json              # JSON output (works with any of the above)
```

### Wildcard search

Use `*` (any characters) and `?` (single character) to search by pattern. Quote the pattern so your shell doesn't expand it.

Wildcard search matches against:

- Binary names in your `$PATH`
- Package/tool names from all 18 supported package managers
- Friendly names (aliases) you've set

### Friendly names

Give binaries memorable names:

```sh
wherearethey name gemini-cli Gemini
wherearethey Gemini              # now looks up gemini-cli
```

Manage them with `name --list` and `name --remove <name>`. Stored in `~/.wherearethey/aliases.json`, matched case-insensitively.

### Install tracking

Optionally track future installs by adding this to `~/.zshrc`:

```sh
eval "$(wherearethey hook zsh)"
```

This wraps brew, npm, cargo, pip, and other commands so every install/uninstall is logged. View with `wherearethey history`, clear with `wherearethey history --clear`.

---

## Supported sources (18 package managers + 10 path-based)

**Scanned by `--all` and wildcard search:**
Homebrew, npm, pnpm, Bun, Deno, Cargo, Go, pipx, uv, pip, Ruby gems, Composer, .NET tools, Nix, MacPorts, Conda, mise, gh extensions

**Detected by path heuristics (single lookups):**
rustup, asdf, nvm, proto, sdkman, ghcup, pkgx, Mint, Xcode CLT, macOS system

---

## How it works

1. **Single lookup** — resolves the binary with `which`, follows symlinks, and matches the path against known install locations.
2. **Wildcard search** — scans PATH, queries all package managers, and checks aliases for pattern matches.
3. **Full scan** (`--all`) — queries each package manager and scans known bin directories.
4. **Unmanaged** (`--unmanaged`) — compares every PATH binary against the full scan; anything unclaimed is flagged.
5. **Tracking** (`hook zsh`) — shell wrappers log install/uninstall events to `~/.wherearethey/history.json`.

---

## Uninstall

```sh
# Homebrew
brew uninstall wherearethey
brew untap babyghost-ys/tap       # optional

# From source
cargo uninstall wherearethey
```

Remove tracked data: `rm -rf ~/.wherearethey`
Remove the shell hook line from `~/.zshrc` if added.

---

## Requirements

- macOS (Apple Silicon and Intel)
- Rust 2024 edition (building from source only)

## Licence

GPL-3.0 — see [LICENSE](LICENSE).
