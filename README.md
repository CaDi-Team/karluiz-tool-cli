# karluiz-tool-cli

A fast, single-binary CLI written in Rust for consuming karluiz tools.

## Installation

```bash
cargo install --path .
```

The binary is installed as **`ktool`**.

## Usage

### 1 — Authenticate

```bash
ktool login
# Enter your KENV API token: ****
# ✓ Token saved to /home/you/.config/ktool/config.toml.
```

Your token is stored in `~/.config/ktool/config.toml` and reused automatically.

### 2 — Set default app & environment

```bash
ktool kenv --set-app=karluiz-calc --set-env=prod
# ✓ Config updated — app: karluiz-calc, env: prod.
```

You can update either flag independently:

```bash
ktool kenv --set-env=staging
ktool kenv --set-app=another-app
```

### 3 — List secrets

```bash
ktool kenv list
# DATABASE_URL=po***rl
# SECRET_KEY=sk***xy
# API_TOKEN=***
```

Secret values are **obfuscated by default** (first 2 + last 2 characters visible).

To see the raw JSON response:

```bash
ktool kenv list --json
```

### Show current context

```bash
ktool kenv
# Current context — app: karluiz-calc, env: prod
# Run `ktool kenv list` to fetch secrets.
```

## Config file

```
~/.config/ktool/config.toml
```

```toml
token = "your-api-token"
app   = "karluiz-calc"
env   = "prod"
```

## Development

```bash
cargo build          # debug build
cargo test           # unit tests
cargo clippy         # lint
cargo build --release  # optimised binary → target/release/ktool
```
