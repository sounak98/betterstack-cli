# bs

[![CI](https://github.com/sounak98/betterstack-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/sounak98/betterstack-cli/actions/workflows/ci.yml)
[![Release](https://github.com/sounak98/betterstack-cli/actions/workflows/release.yml/badge.svg)](https://github.com/sounak98/betterstack-cli/releases)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Fast, AI-friendly CLI for [Better Stack](https://betterstack.com). Built for humans, scripts, and AI agents.

## Why a CLI?

Better Stack already has an [MCP server](https://betterstack.com/docs/getting-started/integrations/mcp/). So why build a CLI?

**Works everywhere, not just MCP clients.** Every AI coding tool can run shell commands: Claude Code, Cursor, Windsurf, Aider, Copilot. A CLI needs zero integration setup. MCP servers need per-client configuration that you repeat for every editor.

**No context window bloat.** MCP servers dump their full tool schema into the agent's context. The [GitHub MCP server costs ~55k tokens](https://github.com/modelcontextprotocol/modelcontextprotocol/issues/1576) before a single question is asked. A CLI returns only the data you asked for, and agents can filter it further with pipes.

**Composable.** Chain `bs` with `jq`, `grep`, `gh`, `kubectl`, or any other tool:

```sh
bs monitors list -o json | jq '.[] | select(.Status == "down") | .Name'
```

MCP servers are isolated. They can't pipe into each other.

**Works in CI/CD.** Drop `bs` into any pipeline with an env var. MCP servers have no CI story.

**No intermediary.** `bs` talks directly to the Better Stack API with a token stored locally in `~/.config/bs/`. MCP servers route through a hosted intermediary with OAuth, adding a dependency and a point of failure.

**Open source, single binary.** `curl | sh` to install. No runtime, no Docker, no OAuth flow.

**Useful for humans too.** MCP is exclusively an AI-to-AI interface. `bs` works just as well in your terminal.

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
bs monitors list                        # Table (default, with colors)
bs monitors list -o json                # JSON (great for piping to jq)
bs monitors list -o csv                 # CSV
```

### AI agents

The JSON output makes `bs` a natural fit for agentic workflows:

```sh
# AI agent checks which monitors are down
bs monitors list -o json | jq '.[] | select(.Status == "down")'

# Pipe into other tools
bs monitors list -o json | jq '.[].URL' | xargs -I{} curl -s -o /dev/null -w "%{http_code} {}\n" {}
```

Any AI agent with shell access can use `bs` immediately, no MCP configuration, no tool schemas, no context overhead.

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
