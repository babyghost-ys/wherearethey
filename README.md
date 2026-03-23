# wherearethey

Find where your CLI tools were installed from.

Ever typed `which ffmpeg` and got `/opt/homebrew/bin/ffmpeg` but still had no idea which package manager put it there? **wherearethey** resolves that. Point it at any binary and it tells you the source — brew, cargo, npm, pip, go, and 13 more.

## Install

### From source (requires Rust)

```sh
# Install Rust if you haven't already
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/peterleung/wherearethey.git
cd wherearethey
cargo install --path .
```

The binary lands in `~/.cargo/bin/wherearethey`. Make sure `~/.cargo/bin` is in your `PATH`.

### Uninstall

```sh
cargo uninstall wherearethey
```

To also remove tracked history:

```sh
rm -rf ~/.wherearethey
```

If you added the shell hook to `~/.zshrc`, remove this line:

```sh
eval "$(wherearethey hook zsh)"
```

---

## Usage

```sh
wherearethey ffmpeg              # Look up a single binary
wherearethey --all               # List every detected tool, grouped by source
wherearethey --unmanaged         # Find binaries not managed by any package manager
wherearethey --json              # Output as JSON (combine with any flag above)
wherearethey hook zsh            # Print shell hooks for install tracking
wherearethey history             # Show tracked install history
wherearethey history --clear     # Clear history
wherearethey name <bin> <name>   # Give a binary a friendly name
wherearethey name --list         # List all friendly names
wherearethey name --remove <n>   # Remove a friendly name
```

### Example

```
$ wherearethey ffmpeg

  ffmpeg
  path:    /opt/homebrew/bin/ffmpeg
  target:  /opt/homebrew/Cellar/ffmpeg/7.1.1/bin/ffmpeg
  source:  brew
  version: ffmpeg 7.1.1
```

### Friendly names

Give binaries human-readable names so you can look them up without remembering the exact package name:

```sh
wherearethey name gemini-cli Gemini
wherearethey Gemini   # looks up gemini-cli
```

Names are stored in `~/.wherearethey/aliases.json` and matched case-insensitively.

### Shell hooks (optional)

Track future installs automatically by adding this to your `~/.zshrc`:

```sh
eval "$(wherearethey hook zsh)"
```

This wraps common package manager commands (brew, npm, cargo, pip, etc.) so every install and uninstall is logged to `~/.wherearethey/history.json`. View with `wherearethey history`.

---

## Supported package managers

| # | Source | Detection method |
|---|--------|-----------------|
| 1 | **Homebrew** | Scans `/opt/homebrew/bin` symlinks into Cellar |
| 2 | **npm** (global) | `npm list -g --parseable` |
| 3 | **pnpm** (global) | `pnpm list -g --parseable` |
| 4 | **Bun** (global) | Scans `~/.bun/bin` |
| 5 | **Deno** | Scans `~/.deno/bin` |
| 6 | **Cargo** (Rust) | `cargo install --list` |
| 7 | **Go** | Scans `$GOBIN` or `~/go/bin` |
| 8 | **pipx** | `pipx list --short` |
| 9 | **uv** | `uv tool list` |
| 10 | **pip** (user) | `pip3 list --user --format=json` |
| 11 | **Ruby gems** | `gem list --local` |
| 12 | **Composer** (PHP) | Scans `~/.composer/vendor/bin` |
| 13 | **.NET tools** | Scans `~/.dotnet/tools` |
| 14 | **Nix** | Scans `~/.nix-profile/bin` |
| 15 | **MacPorts** | `port installed` |
| 16 | **Conda** | `conda list --json` |
| 17 | **mise** | `mise list --current --json` |
| 18 | **gh extensions** | `gh extension list` |

Single-binary lookups also detect these via path heuristics (no scanning required):

| Source | Detected from path containing |
|--------|-------------------------------|
| rustup | `/rustup` |
| asdf | `/.asdf/` |
| nvm | `/.nvm/` |
| proto | `/.proto/` |
| sdkman | `/.sdkman/` |
| ghcup | `/.ghcup/` |
| pkgx | `/.pkgx/` |
| Mint | `/.mint/bin` |
| Xcode CLT | `/Library/Developer/CommandLineTools/` |
| macOS system | `/usr/bin` |

---

## How it works

1. **Single lookup** (`wherearethey rg`) — resolves the binary with `which`, follows symlinks, and matches the resolved path against known install locations.
2. **Full scan** (`--all`) — queries each package manager's own listing command and scans known bin directories.
3. **Unmanaged detection** (`--unmanaged`) — compares every binary in `$PATH` against the full scan results; anything unclaimed is not managed by any known package manager.
4. **Install tracking** (`hook zsh`) — shell function wrappers intercept install/uninstall commands and log them to `~/.wherearethey/history.json`.

---

## Requirements

- macOS (tested on Apple Silicon and Intel)
- Rust 2024 edition (for building from source)

---

## Licence

GPL-3.0 — see [LICENSE](LICENSE) for details.
