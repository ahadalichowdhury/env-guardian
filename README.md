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

---

## File safety legend

Use this when reading flag tables below:

| Label | Meaning |
|-------|---------|
| <span style="color:#2ea043"><b>✅ SAFE TO SHARE</b></span> | Send via Slack, email, git — no secrets inside |
| <span style="color:#d1242f"><b>🔒 NEVER SHARE</b></span> | Real secrets or private keys — local only |
| <span style="color:#bf8700"><b>🔐 ENCRYPTED</b></span> | Safe to transfer — only the right key/password opens it |
| <span style="color:#0969da"><b>📁 AUTO-READ</b></span> | Tool reads this file automatically (no flag needed) |
| <span style="color:#656d76"><b>📝 YOU CREATE</b></span> | Tool writes this file for you |

### Which files go in git?

| File | Commit? | Safety |
|------|---------|--------|
| `.envguardian.toml` | Yes | <span style="color:#2ea043"><b>✅ SAFE</b></span> — project config |
| `.env.example` | Yes | <span style="color:#2ea043"><b>✅ SAFE</b></span> — placeholder keys only |
| `.env` | **Never** | <span style="color:#d1242f"><b>🔒 SECRETS</b></span> — real values |
| `.env.enc` | Yes | <span style="color:#bf8700"><b>🔐 ENCRYPTED</b></span> — needs master password |
| `env-guardian.pub` | Yes | <span style="color:#2ea043"><b>✅ SAFE</b></span> — public key |
| `env-guardian.key` | **Never** | <span style="color:#d1242f"><b>🔒 PRIVATE KEY</b></span> |
| `*.share` | Yes | <span style="color:#bf8700"><b>🔐 ENCRYPTED</b></span> — for one recipient only |
| `.envguardian.snapshot` | Optional | <span style="color:#bf8700"><b>⚠️ PLAIN TEXT</b></span> — values visible |

---

## Profiles

Most commands accept **`-p` / `--profile`** to target a specific environment file:

| Profile | Env file (secrets) | Encrypted file |
|---------|-------------------|----------------|
| `default` | `.env` | `.env.enc` |
| `development` | `.env.development` | `.env.development.enc` |
| `staging` | `.env.staging` | `.env.staging.enc` |
| `production` | `.env.production` | `.env.production.enc` |

```bash
env-guardian check -p staging      # reads .env.staging
env-guardian encrypt -p production # reads .env.production
```

**Global option (all commands):** `--root <path>` — project folder (default: current directory).

---

## Detailed guides

Each section explains **what every flag means**, **what file/path to put there**, and **whether that file is safe to share**.

---

### 1. `init` — project setup

Creates config files in your project. **No input files needed** — you only pass flags.

#### Flags explained

| Flag | Required? | What to put | What happens |
|------|-----------|-------------|--------------|
| `--name <NAME>` | No | Text string, e.g. `"My API"` | Sets project name inside `.envguardian.toml` |
| `--with-example` | No | Just add the flag | <span style="color:#656d76"><b>📝 CREATES</b></span> `.env.example` |
| `--with-profiles` | No | Just add the flag | <span style="color:#656d76"><b>📝 CREATES</b></span> `.env.development.example`, `.env.staging.example`, `.env.production.example` |
| `--force` | No | Just add the flag | Overwrites existing `.envguardian.toml` |

#### Files involved

| File | Role | Safety |
|------|------|--------|
| `.envguardian.toml` | <span style="color:#656d76"><b>📝 OUTPUT</b></span> — tool creates | <span style="color:#2ea043"><b>✅ Commit to git</b></span> |
| `.env.example` | <span style="color:#656d76"><b>📝 OUTPUT</b></span> — if `--with-example` | <span style="color:#2ea043"><b>✅ Commit to git</b></span> |
| `.env` | <span style="color:#0969da"><b>📁 YOU create manually</b></span> — copy from example | <span style="color:#d1242f"><b>🔒 Never commit</b></span> |

#### Example

```bash
cd my-api
env-guardian init --with-example --name "My API"
cp .env.example .env
# edit .env with real secrets locally
```

**Creates:**

```
my-api/
├── .envguardian.toml    ← config (safe for git)
└── .env.example         ← template (safe for git)
```

**Multi-environment:**

```bash
env-guardian init --with-profiles --force
```

Creates `.env.development.example`, `.env.staging.example`, `.env.production.example` — copy each to `.env.development`, `.env.staging`, `.env.production`.

---

### 2. `check` — validate env keys

Compares **`.env`** vs **`.env.example`** and scans codebase for env var usage. **No file flags** — tool auto-reads files based on profile.

#### Flags explained

| Flag | Required? | What to put | What happens |
|------|-----------|-------------|--------------|
| `-p, --profile <NAME>` | No | Profile name: `production`, `staging`, etc. | <span style="color:#0969da"><b>📁 AUTO-READS</b></span> `.env.production` + `.env.production.example` |
| `--strict` | No | Just add the flag | Warnings (EXTRA, EMPTY) also fail — use in CI |
| `--no-scan` | No | Just add the flag | Skip codebase scan (faster) |
| `--root <PATH>` | No | Project folder path | Run check in another directory |

#### Files involved (auto-read)

| File | Role | Safety |
|------|------|--------|
| `.env` (or profile env) | <span style="color:#0969da"><b>📁 INPUT</b></span> — your real secrets | <span style="color:#d1242f"><b>🔒 Never share</b></span> |
| `.env.example` | <span style="color:#0969da"><b>📁 INPUT</b></span> — key template | <span style="color:#2ea043"><b>✅ Safe</b></span> |
| `src/**/*.ts`, etc. | <span style="color:#0969da"><b>📁 SCANNED</b></span> — codebase | — |

#### Example

**`.env.example`** (<span style="color:#2ea043">safe</span>):

```env
DATABASE_URL=
API_KEY=
PORT=3000
```

**`.env`** (<span style="color:#d1242f">secrets</span> — missing `API_KEY`):

```env
DATABASE_URL=postgres://localhost:5432/mydb
PORT=3000
```

```bash
env-guardian check
```

**Output:**

```
  MISSING (1):
    • API_KEY — in .env.example, not in .env
✗ Check failed
```

#### Status colors

| Status | Severity | Meaning | Fix |
|--------|----------|---------|-----|
| <span style="color:#d1242f"><b>MISSING</b></span> | Error | Key in example but not in `.env` | Add to `.env` |
| <span style="color:#d1242f"><b>UNDEFINED_IN_ENV</b></span> | Error | Used in code but not defined | Add to `.env` + `.env.example` |
| <span style="color:#bf8700"><b>EXTRA</b></span> | Warning | Key in `.env` but not in example | Add to `.env.example` |
| <span style="color:#bf8700"><b>EMPTY</b></span> | Warning | Key exists but value is blank | Fill the value |

```bash
env-guardian check --strict --no-scan   # CI mode
```

---

### 3. `encrypt` / `decrypt` — local vault

Lock `.env` with a **master password**. Anyone with the password can decrypt.

#### `encrypt` flags explained

| Flag | Required? | What path to put | File type | Safety |
|------|-----------|------------------|-----------|--------|
| `-p, --profile <NAME>` | No | Profile name | Uses profile's `.env` automatically | — |
| `--file <PATH>` | No | e.g. `./secrets/prod.env` | <span style="color:#d1242f"><b>🔒 Plain env with secrets</b></span> | Input — your secrets |
| `--output <PATH>` | No | e.g. `./secrets/prod.env.enc` | <span style="color:#bf8700"><b>🔐 Encrypted blob</b></span> | Output — safe for git |

**If you skip `--file`:** reads `.env` (or profile env like `.env.production`).  
**If you skip `--output`:** writes `.env.enc` (or profile encrypted file).

#### `decrypt` flags explained

| Flag | Required? | What path to put | File type | Safety |
|------|-----------|------------------|-----------|--------|
| `-p, --profile <NAME>` | No | Profile name | Uses profile's `.env.enc` automatically | — |
| `--file <PATH>` | No | e.g. `./secrets/prod.env.enc` | <span style="color:#bf8700"><b>🔐 Encrypted input</b></span> | From git / teammate |
| `--output <PATH>` | No | e.g. `./secrets/prod.env` | <span style="color:#d1242f"><b>🔒 Plain env output</b></span> | Local secrets file |

#### Example workflow

```bash
# ENCRYPT — reads .env, writes .env.enc
env-guardian encrypt
# prompts: Enter master password (twice)

# Custom paths
env-guardian encrypt \
  --file ./my.env \
  --output ./my.env.enc

# Profile shortcut
env-guardian encrypt -p production
# reads .env.production → writes .env.production.enc
```

```bash
git add .env.enc .env.example     # ✅ safe files only
git commit -m "Add encrypted env"
```

**New machine:**

```bash
env-guardian decrypt              # reads .env.enc → writes .env
# enter master password from team vault (1Password, etc.)
```

| Who can decrypt? | Anyone with **master password** |
|------------------|--------------------------------|

---

### 4. `hook` — prevent git leaks

Blocks git commits that stage secret files. **No file flags** — operates on your git repo.

#### Commands

| Command | What it does | Files touched |
|---------|--------------|---------------|
| `hook install` | <span style="color:#656d76"><b>📝 CREATES</b></span> `.git/hooks/pre-commit` | Hook script in git |
| `hook run` | Runs check manually | Scans staged files |
| `hook uninstall` | Removes the hook | Deletes hook script |

#### Example

```bash
git init
env-guardian hook install
```

**Test — try to commit `.env`:**

```bash
git add .env
git commit -m "oops"
# ❌ BLOCKED: secret files staged: .env
```

```bash
git reset HEAD .env   # fix
```

---

### 5. `tui` — interactive terminal editor

Opens a terminal UI to edit env vars. **No file flags** — edits profile env file directly.

#### Flags explained

| Flag | Required? | What to put | What file is edited |
|------|-----------|-------------|-------------------|
| `-p, --profile <NAME>` | No | `production`, `staging`, etc. | <span style="color:#0969da"><b>📁 EDITS</b></span> `.env.production` (or profile env) |

```bash
env-guardian tui              # edits .env
env-guardian tui -p staging   # edits .env.staging
```

| Key | Action |
|-----|--------|
| `j` / `k` | Move up / down |
| `Enter` | Edit value |
| `n` | New key |
| `d` | Delete key |
| `p` | Switch profile |
| `c` | Run check |
| `q` | Quit and save |

**File saved:** same `.env` file — <span style="color:#d1242f"><b>🔒 never commit</b></span>.

---

### 6. `ci` — GitHub Actions

Generates a CI workflow file. **No input files** — creates workflow in your repo.

#### Commands explained

| Command | What it does | Output file |
|---------|--------------|-------------|
| `ci install` | <span style="color:#656d76"><b>📝 CREATES</b></span> workflow | `.github/workflows/env-guardian.yml` |
| `ci install --force` | Overwrites existing workflow | Same path |
| `ci print` | Prints YAML to terminal | No file written |

```bash
env-guardian ci install
git add .github/workflows/env-guardian.yml
git commit -m "Add EnvGuardian CI"
```

CI runs `env-guardian check --strict --no-scan` on every push/PR.

---

### 7. `drift` — detect config drift

Compare **local `.env`** against another source (snapshot, file, Vercel, AWS).

#### `drift snapshot` — save baseline

| Flag | Required? | What to put | What happens |
|------|-----------|-------------|--------------|
| `-p, --profile <NAME>` | No | Profile name | <span style="color:#0969da"><b>📁 READS</b></span> `.env.production` (etc.) |
| `-o, --output <PATH>` | No | e.g. `prod.snapshot` | <span style="color:#656d76"><b>📝 WRITES</b></span> plain copy of env |

```bash
env-guardian drift snapshot -p production -o prod.snapshot
```

| Output file | Safety |
|-------------|--------|
| `prod.snapshot` | <span style="color:#bf8700"><b>⚠️ PLAIN TEXT</b></span> — values visible, use for drift only. For secrets use `share`. |

#### `drift check` — compare

| Flag | Required? | What to put | Compares against |
|------|-----------|-------------|------------------|
| `-p, --profile <NAME>` | No | Profile name | <span style="color:#0969da"><b>📁 LOCAL:</b></span> `.env.production` |
| `--snapshot <PATH>` | One of these | e.g. `prod.snapshot` | Saved snapshot file |
| `--remote-env <PATH>` | One of these | e.g. `./deploy/.env.remote` | Another local `.env` file |
| `--vercel-project <ID>` | One of these | e.g. `my-next-app` | Vercel dashboard env (needs `VERCEL_TOKEN`) |
| `--vercel-team <ID>` | No | e.g. `team_abc123` | Vercel team scope |
| `--aws-ssm-path <PATH>` | One of these | e.g. `/myapp/prod/` | AWS SSM parameters (needs AWS CLI) |
| `--aws-region <REGION>` | With AWS | e.g. `us-east-1` | AWS region |

**Pick ONE compare source:** `--snapshot`, `--remote-env`, `--vercel-project`, or `--aws-ssm-path`.

#### Examples

```bash
# Save baseline
env-guardian drift snapshot -p production -o prod.snapshot

# Compare local vs snapshot
env-guardian drift check -p production --snapshot prod.snapshot

# Compare local vs another file you received
env-guardian drift check --remote-env ./teammate/.env

# Compare local vs Vercel
export VERCEL_TOKEN=xxx
env-guardian drift check --vercel-project my-app -p production

# Compare local vs AWS SSM
env-guardian drift check \
  --aws-ssm-path /myapp/production/ \
  --aws-region us-east-1 \
  -p production
```

---

### 8. `share` — zero-knowledge team sharing (E2E)

Send env to a teammate so **only their private key** can decrypt. Email/Slack never sees plaintext.

#### Three file types in share flow

| File | Who has it | What it is | Share? |
|------|------------|------------|--------|
| `env-guardian.pub` | Each person | Public key | <span style="color:#2ea043"><b>✅ YES — send to teammates</b></span> |
| `env-guardian.key` | Each person | Private key | <span style="color:#d1242f"><b>🔒 NEVER — keep on your machine only</b></span> |
| `*.share` | Sender → Receiver | Encrypted env package | <span style="color:#bf8700"><b>🔐 YES — safe over any channel</b></span> |
| `.env.production` | Sender | Plain secrets | <span style="color:#d1242f"><b>🔒 NEVER — use share instead</b></span> |

#### Who sends what to whom

```
BOB (receiver)                              ALICE (sender)
──────────────                              ──────────────

1. share keygen
   env-guardian.pub  ──────────────────►  save as bob.env-guardian.pub
   env-guardian.key  (Bob keeps secret)

                                           2. has .env.production (secrets)
                                           3. share create
                                              --recipient bob.env-guardian.pub
                                              --input .env.production
                                              --output bob-prod.share

                                           bob-prod.share ──────────────► BOB

4. share open
   --share bob-prod.share
   --key env-guardian.key
   --output .env.production
   → Bob gets .env.production locally ✓
```

---

#### `share keygen` — generate keypair (once per person)

| Flag | Required? | What to put | What is created |
|------|-----------|-------------|-----------------|
| `-o, --output-dir <DIR>` | No | Folder path, e.g. `~/.env-guardian-keys` | Keys saved inside that folder |

```bash
mkdir -p ~/.env-guardian-keys
env-guardian share keygen -o ~/.env-guardian-keys
```

**Creates:**

| File | Path example | Share? |
|------|--------------|--------|
| `env-guardian.pub` | `~/.env-guardian-keys/env-guardian.pub` | <span style="color:#2ea043"><b>✅ Send to teammates</b></span> |
| `env-guardian.key` | `~/.env-guardian-keys/env-guardian.key` | <span style="color:#d1242f"><b>🔒 Never share or commit</b></span> |

---

#### `share create` — sender encrypts for recipient

```bash
env-guardian share create \
  --recipient ./bob.env-guardian.pub \
  --input .env.production \
  --output bob-prod.share
```

| Flag | Required? | What path to put | Explanation |
|------|-----------|------------------|-------------|
| **`--recipient`** | <span style="color:#d1242f"><b>YES</b></span> | `./bob.env-guardian.pub` | <span style="color:#2ea043"><b>Recipient's PUBLIC key</b></span> — Bob sent you his `.pub` file. Any filename works. |
| **`--input`** | No | `.env.production` | <span style="color:#d1242f"><b>YOUR env file with secrets</b></span> — the file you want to encrypt and send. Never send this directly. |
| **`--output`** | No | `bob-prod.share` | <span style="color:#bf8700"><b>Encrypted package</b></span> — send THIS file to Bob (Slack, email, etc.) |
| `-p, --profile` | No | `production` | Only if you skip `--input` — auto-reads `.env.production` |

**Without `--input`:**

```bash
env-guardian share create --recipient ./bob.pub -p production
# reads .env.production → writes .env.production.share (default output name)
```

---

#### `share open` — receiver decrypts

```bash
env-guardian share open \
  --share bob-prod.share \
  --key ~/.env-guardian-keys/env-guardian.key \
  --output .env.production
```

| Flag | Required? | What path to put | Explanation |
|------|-----------|------------------|-------------|
| **`--share`** | <span style="color:#d1242f"><b>YES</b></span> | `bob-prod.share` | <span style="color:#bf8700"><b>Encrypted package</b></span> Alice sent you — download and save locally |
| **`--key`** | <span style="color:#d1242f"><b>YES</b></span> | `~/.env-guardian-keys/env-guardian.key` | <span style="color:#d1242f"><b>YOUR private key</b></span> — from your `keygen` step. Never share this. |
| **`--output`** | No | `.env.production` | Where to write decrypted env. Default: `decrypted.env` |

**Without `--output`:** writes to `decrypted.env` in current folder.

---

#### Full Alice → Bob walkthrough

**Bob (one-time setup):**

```bash
mkdir -p ~/.env-guardian-keys
env-guardian share keygen -o ~/.env-guardian-keys
# sends env-guardian.pub to Alice ✅
```

**Alice (sender):**

```bash
# Bob's public key saved as:
# ./bob.env-guardian.pub

# Alice's secrets (never send this file):
cat .env.production
# DATABASE_URL=postgres://prod.db.internal:5432/app
# API_KEY=sk_live_super_secret

env-guardian share create \
  --recipient ./bob.env-guardian.pub \
  --input .env.production \
  --output bob-prod.share

# sends bob-prod.share to Bob ✅
```

**Bob (decrypt):**

```bash
env-guardian share open \
  --share bob-prod.share \
  --key ~/.env-guardian-keys/env-guardian.key \
  --output .env.production

cat .env.production   # secrets restored locally
# ⚠ Do not commit .env.production to git
```

---

#### Local test (same machine)

```bash
mkdir -p ~/env-share-test && cd ~/env-share-test

mkdir -p receiver-keys
env-guardian share keygen -o ./receiver-keys

cat > .env.production << 'EOF'
DATABASE_URL=postgres://localhost/mydb
API_KEY=secret-key-123
EOF

env-guardian share create \
  --recipient ./receiver-keys/env-guardian.pub \
  --input .env.production \
  --output prod.share

env-guardian share open \
  --share prod.share \
  --key ./receiver-keys/env-guardian.key \
  --output .env.received

diff .env.production .env.received   # no output = success
```

---

#### Share vs encrypt — which to use?

| Method | Who can decrypt | Best for |
|--------|-----------------|----------|
| `encrypt` / `decrypt` | Anyone with **master password** | Solo dev, shared team password in vault |
| `share create` / `open` | Only **recipient's private key** | Per-person E2E, no shared password |

---

## Master flag cheat sheet

Quick reference — **what file goes in each option**:

| Command | Flag | Put this path / value |
|---------|------|----------------------|
| **Global** | `--root` | Project folder, e.g. `/path/to/my-api` |
| **init** | `--name` | Text: `"My API"` |
| **init** | `--with-example` | (flag only) creates `.env.example` |
| **init** | `--with-profiles` | (flag only) creates profile examples |
| **check** | `-p` | Profile: `production` → reads `.env.production` |
| **encrypt** | `--file` | <span style="color:#d1242f">Plain env input</span>, e.g. `./my.env` |
| **encrypt** | `--output` | <span style="color:#bf8700">Encrypted output</span>, e.g. `./my.env.enc` |
| **decrypt** | `--file` | <span style="color:#bf8700">Encrypted input</span>, e.g. `./my.env.enc` |
| **decrypt** | `--output` | <span style="color:#d1242f">Plain env output</span>, e.g. `./my.env` |
| **tui** | `-p` | Profile → edits that profile's `.env` |
| **drift snapshot** | `-o` | Output path, e.g. `prod.snapshot` |
| **drift check** | `--snapshot` | Snapshot file, e.g. `prod.snapshot` |
| **drift check** | `--remote-env` | Another `.env` file, e.g. `./remote.env` |
| **drift check** | `--vercel-project` | Vercel project name/ID (not a file) |
| **drift check** | `--aws-ssm-path` | SSM prefix, e.g. `/myapp/prod/` |
| **share keygen** | `-o` | Folder for keys, e.g. `~/.env-guardian-keys` |
| **share create** | `--recipient` | <span style="color:#2ea043">Recipient's `.pub` file</span> |
| **share create** | `--input` | <span style="color:#d1242f">Your secrets `.env` file</span> |
| **share create** | `--output` | <span style="color:#bf8700">Share package</span>, e.g. `bob-prod.share` |
| **share open** | `--share` | <span style="color:#bf8700">Received share file</span>, e.g. `bob-prod.share` |
| **share open** | `--key` | <span style="color:#d1242f">Your `.key` private key file</span> |
| **share open** | `--output` | <span style="color:#d1242f">Decrypted env output</span>, e.g. `.env.production` |

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

Run `env-guardian <command> --help` for all flags.

---

## Troubleshooting

| Issue | Fix |
|-------|-----|
| `command not found` | `export PATH="$HOME/.cargo/bin:$PATH"` |
| Old version / missing commands | `cargo install env-guardian --force` |
| Shell alias overrides binary | `unalias env-guardian` |
| `check` MISSING keys | Add keys from `.env.example` to `.env` |
| `decryption failed` (encrypt) | Wrong master password |
| `decryption failed` (share) | Wrong private key — share was for someone else |
| `share create` error | Pass `--recipient` with recipient's `.pub` file |

---

## Development

```bash
cargo test
cargo build --release
```

## License

MIT
