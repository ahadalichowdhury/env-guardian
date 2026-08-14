# EnvGuardian (ConfigSync Pro)

Secure CLI for managing `.env` files — validate keys, encrypt secrets, block git leaks, detect drift, and share env files safely with your team.

**Install:** [crates.io/crates/env-guardian](https://crates.io/crates/env-guardian) · **Repo:** [github.com/ahadalichowdhury/env-guardian](https://github.com/ahadalichowdhury/env-guardian)

Two binaries ship in one package: `env-guardian` and `config-sync` (same tool, use either).

```bash
env-guardian --help
config-sync --help     # identical binary
```

---

## What does it do?

| Problem | Solution |
|---------|----------|
| Missing env keys in `.env` | `check` compares `.env` vs `.env.example` |
| Secrets used in code but not defined | `check` scans your codebase |
| Accidentally committing `.env` to git | `hook install` blocks commits |
| Sharing secrets with teammates | `share create` (E2E encrypted) |
| Local vs server config mismatch | `drift check` |
| Multiple environments (dev/staging/prod) | `-p development` profile flag |

---

## Prerequisites

| Feature | Requirement |
|---------|-------------|
| `cargo install` | [Rust](https://rustup.rs) + `~/.cargo/bin` in PATH |
| Binary download | No Rust required |
| `hook install` | Git repo (`git init`) |
| Drift (Vercel) | `VERCEL_TOKEN` env var |
| Drift (AWS) | AWS CLI + credentials |

---

## Installation

### Option A — cargo install (recommended)

```bash
cargo install env-guardian
export PATH="$HOME/.cargo/bin:$PATH"   # add to ~/.zshrc for persistence
env-guardian --version
```

### Option B — Download binary

1. Go to [GitHub Releases](https://github.com/ahadalichowdhury/env-guardian/releases)
2. Download for your platform:

| Platform | File |
|----------|------|
| Mac (Apple Silicon) | `env-guardian-*-aarch64-apple-darwin.tar.gz` |
| Mac (Intel) | `env-guardian-*-x86_64-apple-darwin.tar.gz` |
| Linux | `env-guardian-*-x86_64-unknown-linux-gnu.tar.gz` |
| Windows | `env-guardian-*-x86_64-pc-windows-msvc.zip` |

```bash
tar -xzf env-guardian-*-aarch64-apple-darwin.tar.gz
export PATH="$PWD/env-guardian-*-aarch64-apple-darwin:$PATH"
env-guardian --version
```

### Option C — From source

```bash
git clone https://github.com/ahadalichowdhury/env-guardian.git
cd env-guardian
cargo install --path .
```

---

## Quick start

```bash
cd my-api
env-guardian init --with-example
cp .env.example .env
# edit .env with real values
env-guardian check
git init
env-guardian hook install
```

### Which files go in git?

| File | Commit? | Why |
|------|---------|-----|
| `.envguardian.toml` | Yes | Project config |
| `.env.example` | Yes | Placeholder keys (no secrets) |
| `.env` | **Never** | Contains real secrets |
| `.env.enc` | Yes | Encrypted — safe to commit |
| `env-guardian.pub` | Yes | Public share key |
| `env-guardian.key` | **Never** | Private share key |
| `*.share` | Yes | Encrypted for one recipient |
| `.envguardian.snapshot` | Optional | Drift baseline (values visible) |

---

## Profiles

Most commands accept `-p` / `--profile` to target a specific environment file:

| Profile | Env file | Encrypted file |
|---------|----------|----------------|
| `default` | `.env` | `.env.enc` |
| `development` | `.env.development` | `.env.development.enc` |
| `staging` | `.env.staging` | `.env.staging.enc` |
| `production` | `.env.production` | `.env.production.enc` |

```bash
env-guardian check -p staging
env-guardian encrypt -p production
env-guardian tui -p development
```

Global option on all commands: `--root <path>` — project folder (default: current directory).

---

## Detailed guides

### 1. `init` — project setup

Creates `.envguardian.toml` in your project root and optionally generates example env files.

#### Flags

| Flag | Description |
|------|-------------|
| `--name <NAME>` | Project name in config (default: current folder name) |
| `--with-example` | Create `.env.example` template |
| `--with-profiles` | Create `.env.development`, `.env.staging`, `.env.production` examples |
| `--force` | Overwrite existing `.envguardian.toml` |

#### Example — new Node.js API project

```bash
cd my-api
env-guardian init --with-example --name "My API"
```

**Creates:**

```
my-api/
├── .envguardian.toml
└── .env.example
```

**`.env.example` (auto-generated):**

```env
# EnvGuardian .env.example template
# Copy to matching .env and fill in values
DATABASE_URL=
API_KEY=
PORT=3000
```

**Next steps printed by the tool:**

```bash
cp .env.example .env
# edit .env with real values
env-guardian check
```

#### Example — multi-environment setup

```bash
env-guardian init --with-profiles --force
```

**Creates:**

```
my-api/
├── .envguardian.toml
├── .env.development.example
├── .env.staging.example
└── .env.production.example
```

Copy each example to its matching `.env` file and fill in values per environment.

---

### 2. `check` — validate env keys

Compares your `.env` against `.env.example` and scans the codebase for env var references (`process.env`, `os.getenv`, `std::env::var`, `getenv`, etc.).

#### Flags

| Flag | Description |
|------|-------------|
| `-p, --profile <NAME>` | Target profile (default: `default` → `.env`) |
| `--strict` | Treat warnings (EXTRA, EMPTY) as errors — use in CI |
| `--no-scan` | Skip codebase scan |
| `--root <PATH>` | Project root directory |

#### Example — setup files

**`.env.example`:**

```env
DATABASE_URL=
API_KEY=
PORT=3000
```

**`.env` (incomplete):**

```env
DATABASE_URL=postgres://localhost:5432/mydb
PORT=3000
```

**`src/config.ts`:**

```typescript
const apiKey = process.env.API_KEY;
```

#### Run check

```bash
env-guardian check
```

**Example output:**

```
Checking profile: default
  env: .env
  example: .env.example

  MISSING (1):
    • API_KEY — in .env.example, not in .env

  UNDEFINED_IN_ENV (1):
    • API_KEY — used in src/config.ts:1, not in .env or .env.example

✗ Check failed — 1 error(s), 0 warning(s)
```

#### Fix and re-check

Add `API_KEY` to `.env`:

```env
DATABASE_URL=postgres://localhost:5432/mydb
API_KEY=sk_live_abc123
PORT=3000
```

```bash
env-guardian check
```

**Example output:**

```
Checking profile: default
  env: .env
  example: .env.example

✓ All checks passed
```

#### Output status reference

| Status | Severity | Meaning | Fix |
|--------|----------|---------|-----|
| **MISSING** | Error | Key in `.env.example` but missing from `.env` | Add key to `.env` |
| **UNDEFINED_IN_ENV** | Error | Used in code but not in `.env` or `.env.example` | Add key to both files |
| **EXTRA** | Warning | Key in `.env` but not in `.env.example` | Add key to `.env.example` |
| **EMPTY** | Warning | Key exists in `.env` but value is blank | Fill in the value |

#### CI usage

```bash
env-guardian check --strict --no-scan
```

`--strict` makes EXTRA and EMPTY fail the build. `--no-scan` skips codebase scan (faster in CI when you only care about file consistency).

---

### 3. `encrypt` / `decrypt` — local vault

Encrypt `.env` with a master password using AES-256-GCM + Argon2id. The encrypted `.env.enc` file is safe to commit to git.

#### Flags

| Flag | Description |
|------|-------------|
| `-p, --profile <NAME>` | Target profile |
| `--file <PATH>` | Custom input file (overrides profile default) |
| `--output <PATH>` | Custom output file (overrides profile default) |

#### Example — encrypt before pushing to git

**Start with `.env`:**

```env
DATABASE_URL=postgres://prod.db.internal:5432/app
API_KEY=sk_live_super_secret
JWT_SECRET=my-jwt-secret
```

```bash
env-guardian encrypt
```

Prompts for a master password (twice). Creates `.env.enc`.

```bash
git add .env.enc .env.example .envguardian.toml
git commit -m "Add encrypted env"
git push
```

**On a new machine (clone repo):**

```bash
git clone https://github.com/you/my-api.git
cd my-api
env-guardian decrypt
# enter master password
```

Restores `.env` locally. Never commit `.env`.

#### Example — production profile with custom paths

```bash
env-guardian encrypt -p production
# reads .env.production → writes .env.production.enc

env-guardian encrypt --file ./secrets/prod.env --output ./secrets/prod.env.enc
```

```bash
env-guardian decrypt -p production
env-guardian decrypt --file ./secrets/prod.env.enc --output ./secrets/prod.env
```

#### Typical team workflow

```
Developer A (has secrets)          Git repo                    Developer B (new joiner)
─────────────────────────          ────────                    ────────────────────────
.env (local only)                  .env.enc (committed)        env-guardian decrypt
env-guardian encrypt               .env.example (committed)    → .env restored locally
git push                           .envguardian.toml           master password from team vault
```

---

### 4. `hook` — prevent git leaks

Installs a pre-commit hook in `.git/hooks/pre-commit` that blocks commits containing `.env` or other secret files.

#### Commands

| Command | Description |
|---------|-------------|
| `hook install` | Install pre-commit hook |
| `hook run` | Run hook checks manually |
| `hook uninstall` | Remove the hook |

#### Example — block accidental `.env` commits

```bash
cd my-api
git init
env-guardian hook install
```

**Output:**

```
✓ Installed pre-commit hook at .git/hooks/pre-commit
```

**Test it:**

```bash
echo "SECRET=leaked" >> .env
git add .env
git commit -m "oops"
```

**Git rejects the commit:**

```
EnvGuardian: blocked commit — secret files staged:
  .env
Remove them from the index: git reset HEAD <file>
```

**Fix:**

```bash
git reset HEAD .env
```

#### Uninstall

```bash
env-guardian hook uninstall
```

---

### 5. `tui` — interactive terminal editor

Opens a ratatui-based terminal UI for viewing and editing env vars without opening a text editor.

#### Flags

| Flag | Description |
|------|-------------|
| `-p, --profile <NAME>` | Edit a specific profile's env file |

#### Example

```bash
env-guardian tui
env-guardian tui -p staging    # edits .env.staging
```

#### Key bindings

| Key | Action |
|-----|--------|
| `j` / `k` | Move selection up / down |
| `Enter` | Edit the selected value |
| `n` | Add a new key |
| `d` | Delete selected key |
| `p` | Switch profile |
| `c` | Run `check` and show results |
| `q` | Quit (saves changes) |

#### Typical use

```bash
# Quick edit without leaving the terminal
env-guardian tui
# press 'n' → add NEW_FEATURE_FLAG=true
# press 'c' → verify check passes
# press 'q' → save and exit
env-guardian check
```

---

### 6. `ci` — GitHub Actions

Generates a GitHub Actions workflow that runs `env-guardian check --strict` on every push and pull request.

#### Commands

| Command | Description |
|---------|-------------|
| `ci install` | Write `.github/workflows/env-guardian.yml` |
| `ci install --force` | Overwrite existing workflow |
| `ci print` | Print workflow YAML to stdout (no file written) |

#### Example — add CI to your repo

```bash
cd my-api
env-guardian ci install
```

**Creates `.github/workflows/env-guardian.yml`:**

```yaml
name: EnvGuardian
on:
  push:
    branches: [main, master]
  pull_request:

jobs:
  env-consistency:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo install env-guardian
      - run: env-guardian check --strict --no-scan
```

```bash
git add .github/workflows/env-guardian.yml
git commit -m "Add EnvGuardian CI check"
git push
```

Every PR that adds a key to code without updating `.env.example` will fail CI.

**Preview without writing:**

```bash
env-guardian ci print
```

---

### 7. `drift` — detect config drift

Compare your local `.env` against a saved snapshot, another file, Vercel project env, or AWS SSM parameters.

#### `drift snapshot` — save baseline

| Flag | Description |
|------|-------------|
| `-p, --profile <NAME>` | Profile to snapshot |
| `-o, --output <PATH>` | Output file (default: `.envguardian.snapshot`) |

#### `drift check` — compare

| Flag | Description |
|------|-------------|
| `-p, --profile <NAME>` | Local profile to compare |
| `--snapshot <PATH>` | Compare against a saved snapshot file |
| `--remote-env <PATH>` | Compare against another local `.env` file |
| `--vercel-project <ID>` | Compare against Vercel project env (needs `VERCEL_TOKEN`) |
| `--vercel-team <ID>` | Vercel team ID (optional) |
| `--aws-ssm-path <PATH>` | AWS SSM parameter prefix (needs AWS CLI) |
| `--aws-region <REGION>` | AWS region for SSM |

---

#### Example A — snapshot + CI drift check

**Step 1: Save baseline after deploying production env:**

```bash
env-guardian drift snapshot -p production --output prod.snapshot
git add prod.snapshot
git commit -m "Add production env snapshot"
```

**Step 2: Later, someone changes local `.env.production`:**

```env
DATABASE_URL=postgres://new-host:5432/app
API_KEY=sk_live_abc123
NEW_KEY=added_locally
```

**Step 3: Run drift check:**

```bash
env-guardian drift check -p production --snapshot prod.snapshot
```

**Example output:**

```
Drift check — profile: production | local: .env.production | remote: prod.snapshot

  MISSING_LOCAL (1):
    • NEW_KEY — in remote, not in local

  VALUE_MISMATCH (1):
    • DATABASE_URL — local ≠ remote

✗ 2 drift item(s) detected
```

---

#### Example B — compare two local files

You received a `.env` from a teammate or pulled from a server:

```bash
env-guardian drift check --remote-env ./deploy/.env.remote
```

Compares your local `.env` against `./deploy/.env.remote`.

---

#### Example C — Vercel project

```bash
export VERCEL_TOKEN=your_vercel_token

env-guardian drift check --vercel-project my-next-app

env-guardian drift check \
  --vercel-project my-next-app \
  --vercel-team team_abc123 \
  -p production
```

Compares local `.env.production` against env vars configured in your Vercel project dashboard.

---

#### Example D — AWS SSM Parameter Store

```bash
env-guardian drift check \
  --aws-ssm-path /myapp/production/ \
  --aws-region us-east-1 \
  -p production
```

Requires AWS CLI configured (`aws configure` or IAM role). Fetches parameters under `/myapp/production/` and compares against local `.env.production`.

---

### 8. `share` — zero-knowledge team sharing (E2E)

Send env files to a teammate encrypted end-to-end. Only the recipient's private key can decrypt. Uses X25519 key exchange + HKDF + AES-256-GCM.

The email/Slack channel never sees plaintext secrets.

---

#### `share keygen` — generate keypair

| Flag | Description |
|------|-------------|
| `-o, --output-dir <DIR>` | Directory for keys (default: current directory) |

```bash
mkdir -p ~/.env-guardian-keys
env-guardian share keygen -o ~/.env-guardian-keys
```

**Output:**

```
✓ Generated keypair in ~/.env-guardian-keys
  Public:  ~/.env-guardian-keys/env-guardian.pub (share with teammates)
  Private: ~/.env-guardian-keys/env-guardian.key (keep secret — never commit)
```

| File | Share? | Purpose |
|------|--------|---------|
| `env-guardian.pub` | Yes | Teammates use this to encrypt env for you |
| `env-guardian.key` | **Never** | Only you use this to decrypt shares |

Each teammate generates their own keypair once and shares only their `.pub` file.

---

#### `share create` — encrypt env for a recipient

| Flag | Required | Description |
|------|----------|-------------|
| `--recipient <PATH>` | Yes | Recipient's `env-guardian.pub` file |
| `--input <PATH>` | No | Source env file (default: profile's `.env`) |
| `--output <PATH>` | No | Share package path (default: `.env.<profile>.share`) |
| `-p, --profile <NAME>` | No | Profile name (used when `--input` is not set) |

---

#### `share open` — decrypt a received share

| Flag | Required | Description |
|------|----------|-------------|
| `--share <PATH>` | Yes | The share package file |
| `--key <PATH>` | Yes | Your `env-guardian.key` private key |
| `--output <PATH>` | No | Output env file (default: `decrypted.env`) |

---

#### Full walkthrough — sender and receiver

**Characters:**
- **Alice** (team lead) — has production secrets, sends to Bob
- **Bob** (new developer) — needs production env on his machine

---

**Bob (receiver) — one-time setup:**

```bash
mkdir -p ~/.env-guardian-keys
env-guardian share keygen -o ~/.env-guardian-keys
```

Bob sends `~/.env-guardian-keys/env-guardian.pub` to Alice (Slack, email, commit to internal repo).

---

**Alice (sender) — encrypt for Bob:**

Alice has Bob's public key at `./bob.env-guardian.pub`.

**`.env.production`:**

```env
DATABASE_URL=postgres://prod.db.internal:5432/app
API_KEY=sk_live_super_secret
JWT_SECRET=my-jwt-secret
STRIPE_KEY=sk_test_abc
```

```bash
env-guardian share create \
  --recipient ./bob.env-guardian.pub \
  --input .env.production \
  --output bob-prod.share
```

**Output:**

```
✓ Created zero-knowledge share: bob-prod.share
· Encrypted for recipient — only their private key can decrypt
  Send bob-prod.share to teammate (email, Slack, etc.)
```

Alice sends `bob-prod.share` to Bob (encrypted — safe over any channel).

---

**Bob (receiver) — decrypt:**

```bash
env-guardian share open \
  --share bob-prod.share \
  --key ~/.env-guardian-keys/env-guardian.key \
  --output .env.production
```

**Output:**

```
✓ Decrypted share → .env.production
⚠ Do not commit decrypted secrets to git
```

```bash
cat .env.production
```

```
DATABASE_URL=postgres://prod.db.internal:5432/app
API_KEY=sk_live_super_secret
JWT_SECRET=my-jwt-secret
STRIPE_KEY=sk_test_abc
```

Bob now has production env locally. He never shares his private key with anyone.

---

#### Local test script (same machine)

Run this to verify the full flow works:

```bash
mkdir -p ~/env-share-test && cd ~/env-share-test

# 1. Generate receiver keypair
mkdir -p receiver-keys
env-guardian share keygen -o ./receiver-keys

# 2. Create sample env
cat > .env.production << 'EOF'
DATABASE_URL=postgres://localhost/mydb
API_KEY=secret-key-123
JWT_SECRET=my-jwt-secret
EOF

# 3. Encrypt for receiver
env-guardian share create \
  --recipient ./receiver-keys/env-guardian.pub \
  --input .env.production \
  --output prod.share

# 4. Decrypt with receiver's private key
env-guardian share open \
  --share prod.share \
  --key ./receiver-keys/env-guardian.key \
  --output .env.received

# 5. Verify
diff .env.production .env.received
# (no output = files match)
```

---

#### Share vs encrypt — when to use which?

| Method | Who can decrypt | Use case |
|--------|-----------------|----------|
| `encrypt` / `decrypt` | Anyone with master password | Solo dev, team shared password in vault |
| `share create` / `open` | Only recipient's private key | Per-person E2E, no shared password |

---

## Command reference

| Command | Description |
|---------|-------------|
| `init` | Create `.envguardian.toml` |
| `check` | Validate keys vs example + codebase |
| `encrypt` | Lock `.env` → `.env.enc` |
| `decrypt` | Restore `.env.enc` → `.env` |
| `hook install` | Block `.env` git commits |
| `hook run` | Run hook manually |
| `hook uninstall` | Remove hook |
| `tui` | Interactive terminal editor |
| `ci install` | Add GitHub Actions workflow |
| `ci print` | Print workflow YAML |
| `drift check` | Compare local vs remote/snapshot |
| `drift snapshot` | Save env snapshot |
| `share keygen` | Generate E2E keypair |
| `share create` | Encrypt env for recipient |
| `share open` | Decrypt received share |

Run `env-guardian <command> --help` for all flags on any command.

---

## Troubleshooting

| Issue | Fix |
|-------|-----|
| `command not found` | `export PATH="$HOME/.cargo/bin:$PATH"` — add to `~/.zshrc` |
| Old version / missing commands (`share`, `drift`, `ci`) | `cargo install env-guardian --force` then `env-guardian --version` |
| Shell alias overrides binary | `unalias env-guardian` — don't alias to old `target/release` build |
| `check` MISSING keys | Add keys from `.env.example` to `.env` |
| `check` EXTRA keys | Add keys from `.env` to `.env.example` |
| `not a git repository` | Run `git init` before `hook install` |
| `decryption failed` (encrypt/decrypt) | Wrong master password |
| `decryption failed` (share) | Wrong private key — share was encrypted for someone else |
| `unexpected argument '-o'` on `share create` | Use `--output` instead of `-o` on older versions |
| `share create` error | Pass recipient's `.pub` file with `--recipient` |

---

## Development

```bash
cargo test
cargo build --release
```

## License

MIT
