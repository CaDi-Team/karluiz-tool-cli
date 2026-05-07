# karluiz-tool-cli

A fast, single-binary CLI written in Rust for consuming karluiz tools.

## Installation

### Download a pre-built binary (recommended)

Go to the [Releases page](../../releases) and download the archive for your platform:

| Platform | Archive |
|---|---|
| Linux x86_64 | `ktool-linux-x86_64.tar.gz` |
| Linux ARM64 | `ktool-linux-arm64.tar.gz` |
| macOS Intel | `ktool-macos-x86_64.tar.gz` |
| macOS Apple Silicon | `ktool-macos-arm64.tar.gz` |

```bash
# Example: Linux x86_64
curl -L https://github.com/CaDi-Team/karluiz-tool-cli/releases/latest/download/ktool-linux-x86_64.tar.gz \
  | tar -xz
sudo mv ktool /usr/local/bin/
```

### Build from source

Requires [Rust](https://rustup.rs) ≥ 1.85.

```bash
cargo install --locked --path .
```

The binary is installed as **`ktool`**.

---

## Quick Start

```bash
# 1. Authenticate
ktool auth kenv login kenv_your_token_here

# 2. Create a local config in your project directory
cd my-project/
ktool kenv init --project=orbital --app=backend --env=production

# 3. Fetch secrets
ktool kenv list
```

---

## Authentication

Store your kENV API token (created in the [kENV token panel](https://karluiz.com/tools/env)):

```bash
ktool auth kenv login kenv_abc123...
# Token saved to /home/you/.ktool/tokens/kenv.json.
```

Check your stored token status:

```bash
ktool auth kenv whoami
# Authenticated with kenv token: ke***23 (valid)

ktool auth status
# kenv  [ok] configured  stored 2025-01-15T10:30:00Z
```

Remove stored tokens:

```bash
ktool auth kenv logout   # remove only the kenv token
ktool auth logout        # remove all stored tokens
```

### Environment variable

Alternatively, set `$KENV_API_TOKEN` without running `login`:

```bash
export KENV_API_TOKEN=kenv_abc123...
ktool kenv list
```

The resolution order is: stored token file → `$KENV_API_TOKEN` → error.

---

## Context System

`ktool` uses a layered context system to know which project, app, and
environment to use. You never need to pass them as flags on every command.

### Resolution order (highest to lowest priority)

1. **Local `ktool.toml`** — a file in your project directory (walks up)
2. **Named global context** — the active context in `~/.ktool/config.toml`
3. **Global defaults** — the flat `project`/`app`/`env` fields in `~/.ktool/config.toml`

Per-command flags (`--project`, `--app`, `--env`) always override everything.

---

## Local project file (asdf-style)

Drop a `ktool.toml` in any project directory and `ktool` will pick it up
automatically — no manual switching needed. This works like asdf's
`.tool-versions`: it walks up the directory tree until it finds the file.

```bash
cd my-project/
ktool kenv init --project=orbital --app=backend --env=production
```

Creates `./ktool.toml`:

```toml
project = "orbital"
app     = "backend"
env     = "production"
```

Now any `ktool kenv` command run inside `my-project/` or any of its
subdirectories will automatically use these values.

Options for `ktool kenv init`:

| Flag | Description |
|---|---|
| `--project=<slug>` | Project slug to write (defaults to resolved context value) |
| `--app=<name>` | App name to write (defaults to resolved context value) |
| `--env=<name>` | Environment name to write (defaults to resolved context value) |
| `--force` | Overwrite an existing `ktool.toml` |

---

## Named Contexts (kubectl-style)

Named contexts let you store multiple project/app/env combinations under a
short name and switch between them instantly — like `kubectl config use-context`.

```bash
# Create contexts
ktool kenv context set work    --project=orbital --app=backend --env=production
ktool kenv context set staging --project=orbital --app=backend --env=staging
ktool kenv context set personal --project=my-slug --app=frontend --env=dev

# List all contexts (* marks the active one)
ktool kenv context list
#   work     project=orbital app=backend   env=production
# * staging  project=orbital app=backend   env=staging
#   personal project=my-slug app=frontend  env=dev

# Switch context
ktool kenv context use work

# Show what's currently active and where values come from
ktool kenv context current
# Active context source: context 'work'
#   project : orbital
#   app     : backend
#   env     : production

# Update fields in an existing context
ktool kenv context set work --env=staging

# Delete a context
ktool kenv context delete personal
```

### Storing global defaults (no named context)

If you don't need named contexts, use the flat flags directly:

```bash
ktool kenv --set-project=orbital --set-app=backend --set-env=production
```

These flags update the active named context if one is set, or the flat
global defaults otherwise — for backward compatibility.

---

## Fetching Variables

```bash
ktool kenv list
# DATABASE_URL=po***rl
# API_KEY=sk***xy
# SECRET_TOKEN=***
```

Secret values are **obfuscated by default** (first 2 + last 2 characters).

```bash
# Raw JSON output
ktool kenv list --json

# Detailed format: shows which variables are inherited from _shared
ktool kenv list --detailed
# DATABASE_URL=po***rl
# SHARED_SECRET=se***et [shared]
```

---

## Managing Apps

```bash
# List all apps in the project
ktool kenv apps list
# Apps in project 'orbital':
#   backend  (envs: production, staging, dev)
#   frontend (envs: production, staging)

# Create a new app
ktool kenv apps create my-new-service
# App 'my-new-service' created in project 'orbital'.
```

---

## Managing Environments

```bash
# List environments for the current app (from context)
ktool kenv envs list
# Environments for 'orbital/backend':
#   production (12 variable(s))
#   staging    (10 variable(s))
#   dev        (8 variable(s))

# Override the app for a single command
ktool kenv envs list --app=frontend

# Create a new environment
ktool kenv envs create canary
# Environment 'canary' created in 'orbital/backend'.

# Delete an environment (and all its variables)
ktool kenv envs delete canary
# Environment 'canary' deleted from 'orbital/backend'.

# Override project and app inline
ktool kenv envs create uat --project=orbital --app=frontend
```

> **Note:** The `_shared` environment is reserved and cannot be deleted via API.

---

## Managing Variables

### Set (bulk upsert)

```bash
# Set one or more variables (prompts for confirmation)
ktool kenv vars set DATABASE_URL=postgres://user:pass@host/db API_KEY=sk-abc123

# This will set 2 variable(s) in orbital/backend/production:
#   DATABASE_URL
#   API_KEY
#
# Existing keys will be overwritten. Continue? [y/N]

# Skip the confirmation prompt
ktool kenv vars set MY_VAR=hello --yes
ktool kenv vars set MY_VAR=hello -y

# Override context for a single command
ktool kenv vars set MY_VAR=hello --project=orbital --app=backend --env=staging
```

Keys are automatically **uppercased**. Values may contain `=` characters (only
the first `=` is used as the delimiter, so `TOKEN=abc==` sets `TOKEN` to `abc==`).

### Delete

```bash
ktool kenv vars delete OLD_KEY
# Variable 'OLD_KEY' deleted from orbital/backend/production.

# Override context inline
ktool kenv vars delete OLD_KEY --app=frontend --env=staging
```

---

## Per-command Overrides

Every command that uses project/app/env supports inline override flags.
These take the highest priority — they override local files, named contexts,
and global defaults for that single invocation without changing anything persisted.

```bash
--project=<slug>   # override project slug
--app=<name>       # override app name
--env=<name>       # override environment name
```

Examples:

```bash
# List vars in a different environment without switching context
ktool kenv list --env=staging

# Create an environment in a specific app+project
ktool kenv envs create canary --project=other-project --app=api

# Set a variable targeting a specific env without changing context
ktool kenv vars set FEATURE_FLAG=true --env=staging -y
```

---

## Config Files Reference

### Global config: `~/.ktool/config.toml`

```toml
# Currently active named context (optional)
current_context = "work"

# Named contexts
[contexts.work]
project = "orbital"
app     = "backend"
env     = "production"

[contexts.staging]
project = "orbital"
app     = "backend"
env     = "staging"

# Legacy flat defaults (used if no named context is active)
# project = "orbital"
# app     = "backend"
# env     = "production"
```

### Local project file: `ktool.toml` (any directory)

```toml
project = "orbital"   # required for most commands
app     = "backend"   # optional (falls back to global context)
env     = "production" # optional (falls back to global context)
```

All fields are optional — only the fields present override the global context.

### Token storage: `~/.ktool/tokens/kenv.json`

Stored automatically by `ktool auth kenv login`. File permissions are set to
`0600` on Unix systems. Do not edit this file manually.

---

## Complete Command Reference

```
ktool kenv                                   # show resolved context and source
ktool kenv --set-project=<slug>              # update active context's project
ktool kenv --set-app=<name>                  # update active context's app
ktool kenv --set-env=<name>                  # update active context's env

ktool kenv init [--project] [--app] [--env] [--force]
                                             # write ktool.toml in current directory

ktool kenv context list                      # list all named contexts (* = active)
ktool kenv context current                   # show active context + source
ktool kenv context use <name>                # switch active context
ktool kenv context set <name> [--project] [--app] [--env]
                                             # create or update a named context
ktool kenv context delete <name>             # remove a named context

ktool kenv list [--json] [--detailed]        # fetch variables

ktool kenv apps list                         # list apps in project
ktool kenv apps create <name>                # create a new app

ktool kenv envs list [--project] [--app]
ktool kenv envs create <name> [--project] [--app]
ktool kenv envs delete <name> [--project] [--app]

ktool kenv vars set KEY=VALUE ... [--project] [--app] [--env] [-y/--yes]
ktool kenv vars delete <KEY>   [--project] [--app] [--env]

ktool auth kenv login <TOKEN>                # validate and store API token
ktool auth kenv logout                       # remove stored token
ktool auth kenv whoami                       # show token status
ktool auth status                            # show auth status for all providers
ktool auth logout                            # remove all stored tokens

ktool update                                 # self-update binary from GitHub Releases
ktool version                                # print version
```

---

## Using `ktool` in GitHub Actions

### Typical workflow

```yaml
- name: Install ktool
  run: |
    curl -L https://github.com/CaDi-Team/karluiz-tool-cli/releases/latest/download/ktool-linux-x86_64.tar.gz \
      | tar -xz
    sudo mv ktool /usr/local/bin/

- name: Fetch secrets
  env:
    KENV_API_TOKEN: ${{ secrets.KENV_API_TOKEN }}
  run: |
    # No login needed — token is picked up from $KENV_API_TOKEN
    ktool kenv list \
      --project=orbital \
      --app=my-app \
      --env=production \
      --json > secrets.json
```

> Use per-command `--project`/`--app`/`--env` flags in CI rather than relying
> on a `ktool.toml` file, so the values are explicit and version-controlled in
> the workflow YAML.

### Bulk-push secrets from a `.env` file

```bash
# Parse a .env file and push all variables in one shot
ktool kenv vars set $(grep -v '^#' .env | xargs) \
  --project=orbital --app=my-app --env=production --yes
```

---

## Releases & CI

A new release is created automatically when a tag of the form `v*` is pushed:

```bash
git tag v1.0.0
git push origin v1.0.0
```

The release workflow cross-compiles for all four platforms, attaches the archives to the GitHub Release, and retains them as workflow artifacts for 90 days.

---

## Development

```bash
cargo build          # debug build
cargo test           # unit + integration tests
cargo clippy         # lint
cargo build --release  # optimised binary → target/release/ktool
```

---

## Credits — Karluiz

A huge shout-out and all credit where it truly belongs: to my good friend **[Karluiz](https://karluiz.com/)**.

This CLI exists only because Karluiz built the actual tools and services behind it. `ktool` is just a thin Rust wrapper to consume his work from the terminal — the real engineering, the ideas, and the hard work are **100% his**. I refuse to take credit for what he created, and I want anyone reading this to know exactly who made it possible.

Karluiz is a developer who has been coding since age 7 on a Commodore 64 — over 30 years of passion poured into every project. His site is a love letter to that journey: retro pixel aesthetics, 8-bit RPG mini-games, and a growing collection of free developer tools that he builds and shares with the community. From CRM systems to hotel management platforms to his suite of ktools, everything he ships is built with genuine passion and generosity.

**His tools are free.** Go check them out, explore his projects, read his blog, and see what a developer driven by pure love for the craft looks like:

**[karluiz.com](https://karluiz.com/)**

<p align="center">
  <a href="https://karluiz.com/">
    <img src="docs/images/karluiz-tools.png" alt="Karluiz Tools — a growing collection of free developer tools" width="600" />
  </a>
</p>

> *"Every line of code is written with the same passion I felt at age 7."* — Karluiz

Thank you, Karluiz. This project wouldn't exist without your work. Readers: do yourself a favor and visit his page — you won't regret it.
