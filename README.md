# bs

[![CI](https://github.com/sounak98/betterstack-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/sounak98/betterstack-cli/actions/workflows/ci.yml)
[![Release](https://github.com/sounak98/betterstack-cli/actions/workflows/release.yml/badge.svg)](https://github.com/sounak98/betterstack-cli/releases)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Fast, AI-friendly CLI for [Better Stack](https://betterstack.com).

## Install

**Shell (macOS / Linux):**

```sh
curl -fsSL https://raw.githubusercontent.com/sounak98/betterstack-cli/main/install.sh | sh
```

**From source:**

```sh
cargo install --git https://github.com/sounak98/betterstack-cli
```

**Self-update:**

```sh
bs upgrade
```

## Setup

```sh
bs auth init
```

Get your API tokens at [betterstack.com/settings/api-tokens](https://betterstack.com/settings/api-tokens/0). Token input is hidden for security.

## Usage

### Monitors

```sh
bs monitors list                        # List all monitors
bs monitors list --status down          # Filter by status
bs monitors get 12345                   # Get monitor details
bs monitors create --url https://example.com --name "My Site"
bs monitors pause 12345                 # Pause a monitor
bs monitors resume 12345                # Resume a monitor
bs monitors delete 12345                # Delete a monitor
```

### Output formats

```sh
bs monitors list                        # Table (default)
bs monitors list -o json                # JSON (great for piping to jq)
bs monitors list -o csv                 # CSV
```

### AI / scripting

The JSON output mode makes `bs` easy to use with AI tools and scripts:

```sh
bs monitors list -o json | jq '.[].Status'
bs monitors get 12345 -o json
```

## Configuration

Config is stored at `~/.config/bs/config.toml`.

| Flag / Env | Description |
|---|---|
| `--token` / `BETTERSTACK_UPTIME_TOKEN` | Uptime API token |
| `--team` / `BS_TEAM` | Team name (multi-team accounts) |
| `-o` / `BS_OUTPUT` | Output format: `table`, `json`, `csv` |
| `--no-color` / `NO_COLOR` | Disable colored output |
| `-q` / `--quiet` | Minimal output |

## Development

```sh
make check      # fmt + clippy + test
make build      # Debug build
make install    # Release build + install to ~/.local/bin
```

## License

MIT
