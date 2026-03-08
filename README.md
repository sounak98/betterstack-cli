# betterstack-cli (`bs`)

[![CI](https://github.com/sounak98/betterstack-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/sounak98/betterstack-cli/actions/workflows/ci.yml)
[![Release](https://github.com/sounak98/betterstack-cli/actions/workflows/release.yml/badge.svg)](https://github.com/sounak98/betterstack-cli/releases)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Fast, AI-friendly CLI for [Better Stack](https://betterstack.com). The binary is called `bs`. Built for humans, scripts, and AI agents.

<p align="center">
  <img src="assets/demo/demo.gif" alt="bs logs tail demo" width="800">
</p>

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

Get your API tokens at [betterstack.com/settings/api-tokens](https://betterstack.com/settings/api-tokens/0). For log querying, you'll also need ClickHouse SQL credentials from [Connect remotely](https://betterstack.com/docs/logs/query-api/connect-remotely/#getting-started) or ask your Better Stack admin.

## Usage

### Sources

```sh
bs sources list                                      # List all log sources
bs sources get 12345                                 # Get source details + token
bs sources update 12345 --name "Production API"      # Rename a source
bs sources update 12345 --vrl @transform.vrl         # Apply VRL transform from file
bs sources update 12345 --pause                      # Pause ingestion
```

VRL transformations can be managed as files, version-controlled alongside your infra:

```sh
# transform.vrl
.level = .message_json.level
.message = .message_json.message

bs sources update 12345 --vrl @transform.vrl
```

### Logs

```sh
bs logs tail --source 12345                          # Live tail (like kubectl logs -f)
bs logs tail --source 12345 --query 'level = ERROR'  # Tail with filter
bs logs tail --source 12345 --since 30m              # Backfill last 30 minutes, then tail
bs logs tail --source 12345 -o json | jq '.message'  # Pipe to jq
bs logs query 'level = ERROR AND status >= 500' --source 12345 --since 1h
```

Queries use the [Better Stack Live Tail query language](https://betterstack.com/docs/logs/using-logtail/live-tail-query-language/).

### Monitors

```sh
bs monitors list                        # List all monitors
bs monitors list --status down          # Filter by status
bs monitors get 12345                   # Get monitor details
bs monitors create --url https://example.com --name "My Site"
```

### Monitor Groups

```sh
bs monitor-groups list                              # List all groups
bs monitor-groups create --name "Production"        # Create a group
bs monitor-groups update 123 --name "Staging"       # Rename
bs monitor-groups delete 123
```

### Incidents

```sh
bs incidents list --status started
bs incidents get 12345                  # Shows inline timeline + comments
bs incidents ack 12345
bs incidents resolve 12345
bs incidents timeline 12345             # Full colored timeline
bs incidents comments add 12345 --content "Investigating"
```

### Heartbeats

```sh
bs heartbeats list                                  # List all heartbeats
bs heartbeats list --status down                    # Filter by status
bs heartbeats create --name "Nightly backup" --period 86400 --grace 3600
bs heartbeats pause 12345                           # Pause monitoring
bs heartbeats resume 12345                          # Resume monitoring
```

### Heartbeat Groups

```sh
bs heartbeat-groups list
bs heartbeat-groups create --name "Background Jobs"
```

### On-Call Calendars

```sh
bs oncall list                                      # List calendars
bs oncall who                                       # Who's on call right now
bs oncall get 12345                                 # Calendar details + on-call users
bs oncall events 12345                              # List calendar events
```

### Escalation Policies

```sh
bs policies list
bs policies get 12345                               # Shows policy steps
bs policies create --name "Critical" --steps '[{"step_members":[{"type":"current_on_call"}],"wait_before_escalation":300}]'
```

### Severities

```sh
bs severities list
bs severities create --name "Critical" --sms --call # Email on by default
bs severities update 12345 --no-email --sms         # Toggle notifications
```

### Status Pages

```sh
bs status-pages list
bs status-pages create --name "Acme Status" --subdomain acme --timezone UTC
bs status-pages get 12345                           # Shows inline sections + resources
```

**Sections:**

```sh
bs status-pages sections list 12345
bs status-pages sections create 12345 --name "Core Services" --position 0
```

**Resources (add monitors to the page):**

```sh
bs status-pages resources create 12345 --resource-id 67890 --public-name "Website"
bs status-pages resources list 12345                # Shows status + availability
```

**Reports (incidents/maintenance on status page):**

```sh
bs status-pages reports create 12345 \
  --title "API degraded" \
  --message "Investigating increased latency." \
  --affected '[{"status_page_resource_id":"67890","status":"degraded"}]'

bs status-pages reports get 12345 843181            # Shows inline update timeline
bs status-pages reports add-update 12345 843181 \
  --message "Fix deployed." --notify                # Auto-carries affected resources
bs status-pages reports list 12345
```

### Output formats

Every command supports `-o json` for piping to `jq`, AI tools, or scripts:

```sh
bs monitors list -o json | jq '.[] | select(.Status == "down")'
bs logs tail --source 12345 -o json | jq '.level, .message'
```

## Examples

**Incident response from the terminal:**

```sh
# See what's firing
bs incidents list --status started

# Get the full picture: timeline, comments, who's been notified
bs incidents get 12345

# Acknowledge and leave a note
bs incidents ack 12345
bs incidents comments add 12345 --content "Looking into this, appears to be a Redis connection issue"

# Escalate to the on-call team if needed
bs incidents escalate 12345 --type Schedule --schedule-id 67890 --call --sms

# Resolve when done
bs incidents resolve 12345
```

**Post a status page incident in one pipeline:**

```sh
# Create the incident report with an initial message
bs status-pages reports create 12345 \
  --title "Payment processing degraded" \
  --message "We are investigating reports of failed payments." \
  --affected '[{"status_page_resource_id":"67890","status":"degraded"}]'

# Post updates as you learn more (affected resources auto-carried)
bs status-pages reports add-update 12345 843181 \
  --message "Root cause identified: third-party payment gateway timeout."

# Resolve by setting the end time
bs status-pages reports update 12345 843181 --ends-at "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
```

**Pipe into other tools:**

```sh
# Get all down monitors as a Slack-friendly list
bs monitors list --status down -o json | jq -r '.[] | "- \(.Name) (\(.URL))"'

# Count errors per source in the last hour
bs logs query 'level = ERROR' --since 1h -o json | jq 'group_by(.source) | map({source: .[0].source, count: length})'

# Export incidents to CSV for a post-mortem spreadsheet
bs incidents list --from 2026-03-01 --to 2026-03-07 -o csv > incidents.csv

# Find heartbeats that haven't checked in
bs heartbeats list --status down -o json | jq '.[].Name'
```

**CI/CD integration:**

```sh
# Pause monitors during deployment
bs monitors update 12345 --pause
deploy-my-app
bs monitors update 12345 --resume

# Create a maintenance window on your status page
bs status-pages reports create 12345 \
  --title "Scheduled deployment" \
  --report-type maintenance \
  --message "Deploying v2.4.0, brief downtime expected." \
  --starts-at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  --ends-at "$(date -u -d '+30 minutes' +%Y-%m-%dT%H:%M:%SZ 2>/dev/null || date -u -v+30M +%Y-%m-%dT%H:%M:%SZ)"
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
