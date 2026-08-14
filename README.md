# EnvGuardian (ConfigSync Pro)

Secure CLI for managing `.env` files — validate keys, encrypt secrets, block git leaks, detect drift, and share env files safely with your team.

**Install:** [crates.io/crates/env-guardian](https://crates.io/crates/env-guardian) · **Repo:** [github.com/ahadalichowdhury/env-guardian](https://github.com/ahadalichowdhury/env-guardian)

Two binaries ship in one package: `env-guardian` and `config-sync` (same tool, use either).

---

## What does it do?

| Problem | EnvGuardian solution |
|---------|----------------------|
| Missing env keys in `.env` | `check` compares `.env` vs `.env.example` |
| Secrets used in code but not defined | `check` scans your codebase |
| Accidentally committing `.env` to git | `hook install` blocks commits |
| Sharing secrets with teammates | `share create` (E2E encrypted) |
| Local vs server config mismatch | `drift check` |
| Multiple environments (dev/staging/prod) | `-p development` profile flag |

---

## Prerequisites

| Install method | You need |
|----------------|----------|
| `cargo install` | [Rust](https://rustup.rs) (`cargo` on PATH) |
| Binary download | Nothing (no Rust required) |
| `hook install` | Git repo (`git init` in project) |
| `drift` (Vercel) | `VERCEL_TOKEN` environment variable |
| `drift` (AWS) | AWS CLI + credentials configured |

---

## Installation

### Option A — cargo install (recommended)

```bash
cargo install env-guardian
env-guardian --version
```

Requires Rust. Installs `env-guardian` and `config-sync` to `~/.cargo/bin` — ensure that directory is in your PATH.

### Option B — Download binary (no Rust)

1. Go to [GitHub Releases](https://github.com/ahadalichowdhury/env-guardian/releases)
2. Download the file for your platform:

| Platform | File name |
|----------|-----------|
| Mac (Apple Silicon) | `env-guardian-0.1.0-aarch64-apple-darwin.tar.gz` |
| Mac (Intel) | `env-guardian-0.1.0-x86_64-apple-darwin.tar.gz` |
| Linux | `env-guardian-0.1.0-x86_64-unknown-linux-gnu.tar.gz` |
| Windows | `env-guardian-0.1.0-x86_64-pc-windows-msvc.zip` |

3. Extract and run:

**macOS / Linux:**
```bash
tar -xzf env-guardian-0.1.0-aarch64-apple-darwin.tar.gz
cd env-guardian-0.1.0-aarch64-apple-darwin
./env-guardian --version
# Optional: add to PATH permanently
export PATH="$PWD:$PATH"
```

**Windows (PowerShell):**
```powershell
Expand-Archive env-guardian-0.1.0-x86_64-pc-windows-msvc.zip
cd env-guardian-0.1.0-x86_64-pc-windows-msvc
.\env-guardian.exe --version
```

### Option C — Build from source

```bash
git clone https://github.com/ahadalichowdhury/env-guardian.git
cd env-guardian
cargo install --path .
```

---

## Quick start (first project)

Run these inside your project folder:

```bash
# 1. Create config + example templates
env-guardian init --with-example --with-profiles

# 2. Create your local .env from the example
cp .env.example .env

# 3. Edit .env — fill in real values (DATABASE_URL, API_KEY, etc.)
#    Use your editor: nano .env  OR  env-guardian tui

# 4. Validate everything matches
env-guardian check

# 5. Block accidental .env commits (requires git)
git init
env-guardian hook install

# 6. (Optional) Add CI check on GitHub
env-guardian ci install
```

**Files created by `init`:**

| File | Commit to git? | Purpose |
|------|----------------|---------|
| `.envguardian.toml` | Yes | Tool configuration |
| `.env.example` | Yes | Template (no secrets) |
| `.env` | **Never** | Your real secrets |
| `.env.enc` | Yes (optional) | Encrypted backup |

---

## Everyday commands

### Check env consistency

```bash
env-guardian check                    # default profile
env-guardian check -p development     # specific profile
env-guardian check --strict           # warnings = fail (use in CI)
env-guardian check --no-scan          # skip codebase scan
```

Exit code `0` = pass, `1` = fail (use in scripts/CI).

### Encrypt / decrypt (local vault)

```bash
env-guardian encrypt                  # .env → .env.enc (prompts for master password)
env-guardian decrypt                  # .env.enc → .env

env-guardian encrypt -p production    # profile-specific files
```

Keep your master password safe — it cannot be recovered.

### Interactive editor (TUI)

```bash
env-guardian tui
env-guardian tui -p staging
```

| Key | Action |
|-----|--------|
| `j` / `k` or ↑/↓ | Navigate variables |
| `Enter` | Edit selected value |
| `n` | New key |
| `d` | Delete key |
| `p` | Switch profile |
| `c` | Run check |
| `q` | Quit |

### Profiles (dev / staging / production)

```bash
env-guardian check -p development
env-guardian encrypt -p production
env-guardian decrypt -p production
```

| Profile | Env file | Encrypted file |
|---------|----------|----------------|
| `default` | `.env` | `.env.enc` |
| `development` | `.env.development` | `.env.development.enc` |
| `staging` | `.env.staging` | `.env.staging.enc` |
| `production` | `.env.production` | `.env.production.enc` |

---

## Team sharing (zero-knowledge)

Secrets are encrypted for the recipient only — no server sees plaintext.

```bash
# Step 1: Each person generates keys once
env-guardian share keygen -o ./keys
# Share: keys/env-guardian.pub  (public)
# Keep secret: keys/env-guardian.key  (never commit!)

# Step 2: Sender encrypts for recipient
env-guardian share create \
  --recipient ./teammate.env-guardian.pub \
  -p production \
  -o production.share

# Step 3: Send production.share via Slack/email

# Step 4: Recipient decrypts
env-guardian share open \
  --share production.share \
  --key ./keys/env-guardian.key \
  --output .env.production
```

---

## Drift detection

Compare local `.env` with a remote source:

```bash
# Save snapshot for later comparison / CI
env-guardian drift snapshot -o .envguardian.snapshot

# Compare local vs snapshot
env-guardian drift check --snapshot .envguardian.snapshot

# Compare vs another file
env-guardian drift check --remote-env ./server-backup.env

# Compare vs Vercel
export VERCEL_TOKEN=your_token
env-guardian drift check --vercel-project YOUR_PROJECT_ID

# Compare vs AWS SSM (requires AWS CLI)
env-guardian drift check --aws-ssm-path /myapp/prod/ --aws-region us-east-1
```

---

## CI / GitHub Actions

```bash
env-guardian ci install    # creates .github/workflows/env-guardian.yml
env-guardian ci print      # preview workflow YAML
```

The workflow runs `env-guardian check --strict --no-scan` on every push and PR.

---

## Git pre-commit hook

```bash
git init
env-guardian hook install
```

Blocks commits that include plaintext `.env` files. Allows `.env.example` and `.env.enc`.

```bash
env-guardian hook uninstall   # remove hook
```

---

## Troubleshooting

| Issue | Solution |
|-------|----------|
| `command not found: env-guardian` | Run `cargo install env-guardian` or add binary folder to PATH |
| `not a git repository` (hook) | Run `git init` first |
| `check` fails with MISSING | Add missing keys to `.env` (see `.env.example`) |
| `decryption failed` | Wrong master password or corrupted `.env.enc` |
| `VERCEL_TOKEN required` | `export VERCEL_TOKEN=...` before drift check |
| Hook blocks `.env.example` | Should not happen — file a bug if it does |

---

## বাংলা — দ্রুত গাইড

### ইনস্টল

```bash
cargo install env-guardian
```

### প্রজেক্ট সেটআপ

```bash
env-guardian init --with-example --with-profiles
cp .env.example .env
# .env ফাইলে সিক্রেট ভরুন
env-guardian check
```

### মূল কমান্ড

| কমান্ড | কাজ |
|--------|-----|
| `env-guardian check` | `.env` ও `.env.example` মিলানো |
| `env-guardian encrypt` | `.env` এনক্রিপ্ট → `.env.enc` |
| `env-guardian decrypt` | `.env.enc` ডিক্রিপ্ট → `.env` |
| `env-guardian tui` | টার্মিনালে এডিটর |
| `env-guardian hook install` | git-এ `.env` কমিট ব্লক |
| `env-guardian share keygen` | শেয়ারিং কী বানানো |
| `env-guardian drift check` | লোকাল vs সার্ভার অসঙ্গতি |

### গুরুত্বপূর্ণ নিয়ম

- `.env` **কখনো git-তে commit করবেন না**
- `.env.example` commit করা নিরাপদ (সিক্রেট নেই)
- `.env.enc` commit করা নিরাপদ (এনক্রিপ্টেড)
- `env-guardian.key` কখনো শেয়ার করবেন না

---

## All commands

| Command | Description |
|---------|-------------|
| `env-guardian init` | Create `.envguardian.toml` |
| `env-guardian init --with-example` | Also create `.env.example` |
| `env-guardian init --with-profiles` | Create dev/staging/prod templates |
| `env-guardian check` | Validate keys + codebase scan |
| `env-guardian encrypt` / `decrypt` | Local AES-256-GCM vault |
| `env-guardian hook install` | Git pre-commit hook |
| `env-guardian tui` | Interactive terminal UI |
| `env-guardian ci install` | GitHub Actions workflow |
| `env-guardian drift check` | Detect config drift |
| `env-guardian drift snapshot` | Save env snapshot |
| `env-guardian share keygen` | Generate E2E keypair |
| `env-guardian share create` | Encrypt file for teammate |
| `env-guardian share open` | Decrypt received share |

Global option: `--root <path>` — project directory (default: current folder).

---

## Development

```bash
cargo test
cargo build --release
```

## License

MIT
