# EnvGuardian (ConfigSync Pro)

Secure CLI for managing `.env` files — key validation, codebase scanning, AES-256-GCM vault encryption, pre-commit hooks, multi-environment profiles, interactive TUI, CI integration, drift detection, and zero-knowledge team sharing.

**Binaries:** `env-guardian` and `config-sync` (alias, same tool).

## Features

### Phase 1 (MVP)
- Compare `.env` vs `.env.example` (missing / extra / empty keys)
- Codebase scan (`process.env`, `os.getenv`, `std::env::var`, …)
- Encrypt `.env` → `.env.enc` (Argon2id + AES-256-GCM)
- Decrypt `.env.enc` → `.env`

### Phase 2
- Multi-environment profiles (`development`, `staging`, `production`)
- Pre-commit hook — blocks plaintext `.env` commits
- Interactive TUI — browse, edit, delete env vars

### Phase 3
- **CI/CD** — GitHub Actions workflow installer
- **Drift detection** — local vs snapshot, Vercel API, AWS SSM (CLI)
- **Zero-knowledge sharing** — X25519 E2E encrypted share packages

## Install (public users)

### Option 1 — cargo install (recommended for developers)

After publishing to [crates.io](https://crates.io):

```bash
cargo install env-guardian
env-guardian --version
```

From GitHub (before crates.io):

```bash
cargo install env-guardian --git https://github.com/YOUR_USERNAME/env-guardian --tag v0.1.0
```

### Option 2 — Download binary (no Rust needed)

1. Open GitHub → **Releases**
2. Download `env-guardian-0.1.0-<platform>.tar.gz` (or `.zip` on Windows)
3. Extract and add to PATH:

```bash
tar -xzf env-guardian-0.1.0-aarch64-apple-darwin.tar.gz
export PATH="$PWD/env-guardian-0.1.0-aarch64-apple-darwin:$PATH"
env-guardian --version
```

### Option 3 — Homebrew (optional, after tap setup)

```bash
brew install YOUR_USERNAME/tap/env-guardian
```

## Install (build from source)

```bash
git clone https://github.com/YOUR_USERNAME/env-guardian
cd env-guardian
cargo install --path .
```

## Quick start

```bash
env-guardian init --with-example --with-profiles
cp .env.example .env
env-guardian check
env-guardian hook install
env-guardian ci install
```

## Commands

| Command | Description |
|---------|-------------|
| `env-guardian init` | Create `.envguardian.toml` |
| `env-guardian check` | Validate env keys + codebase |
| `env-guardian check -p production --strict` | Profile check for CI |
| `env-guardian encrypt / decrypt` | Local vault |
| `env-guardian hook install` | Git pre-commit hook |
| `env-guardian tui` | Interactive editor |
| `env-guardian ci install` | GitHub Actions workflow |
| `env-guardian ci print` | Print workflow YAML |
| `env-guardian drift check` | Compare local vs remote/snapshot |
| `env-guardian drift snapshot` | Save snapshot for CI |
| `env-guardian share keygen` | Generate E2E keypair |
| `env-guardian share create` | Encrypt file for teammate |
| `env-guardian share open` | Decrypt received share |

## Phase 3 — CI (GitHub Actions)

```bash
env-guardian ci install
```

Creates `.github/workflows/env-guardian.yml` — runs `check --strict` on push/PR.

Optional drift in CI — save a snapshot first:

```bash
env-guardian drift snapshot -o .envguardian.snapshot
git add .envguardian.snapshot
```

## Phase 3 — Drift detection

Compare local `.env` against a remote source:

```bash
# vs local snapshot file
env-guardian drift check --snapshot .envguardian.snapshot

# vs another env file
env-guardian drift check --remote-env /path/to/server.env

# vs Vercel (needs VERCEL_TOKEN)
export VERCEL_TOKEN=...
env-guardian drift check --vercel-project my-project-id

# vs AWS SSM (needs AWS CLI + credentials)
env-guardian drift check --aws-ssm-path /myapp/prod/ --aws-region us-east-1

# save current local as snapshot
env-guardian drift snapshot -o .envguardian.snapshot
```

Drift types: `MISSING_LOCAL`, `MISSING_REMOTE`, `VALUE_MISMATCH` (values hidden in output).

## Phase 3 — Zero-knowledge sharing

End-to-end encrypted sharing — server never sees plaintext.

```bash
# Each teammate generates keys once
env-guardian share keygen -o ./keys
# Share env-guardian.pub publicly, keep env-guardian.key secret

# Sender encrypts for recipient
env-guardian share create --recipient teammate.pub -p production
# → .env.production.share (send via Slack/email)

# Recipient decrypts
env-guardian share open --share .env.production.share --key ./keys/env-guardian.key
# → decrypted.env
```

Crypto: X25519 ECDH + HKDF + AES-256-GCM.

## Profiles

| Profile | Env | Encrypted |
|---------|-----|-----------|
| `default` | `.env` | `.env.enc` |
| `development` | `.env.development` | `.env.development.enc` |
| `staging` | `.env.staging` | `.env.staging.enc` |
| `production` | `.env.production` | `.env.production.enc` |

## TUI keys

`j/k` navigate · `Enter` edit · `n` new · `d` delete · `p` profile · `c` check · `q` quit

## বাংলা — Phase 3

```bash
env-guardian ci install                    # GitHub Actions
env-guardian drift snapshot                # CI snapshot
env-guardian drift check --snapshot .envguardian.snapshot
env-guardian share keygen                  # কী জেনারেট
env-guardian share create --recipient pub  # টিমমেটের জন্য এনক্রিপ্ট
env-guardian share open --share file.share --key private.key
```

## Development

```bash
cargo test    # 25 tests
cargo build --release
```

## License

MIT
