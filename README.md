# EnvGuardian (ConfigSync Pro)

**Languages:** [English](#english) · [বাংলা](#bangla)

Secure CLI for managing `.env` files — validate keys, encrypt secrets, block git leaks, detect drift, and share env files safely with your team.

**Install:** [crates.io/crates/env-guardian](https://crates.io/crates/env-guardian) · **Repo:** [github.com/ahadalichowdhury/env-guardian](https://github.com/ahadalichowdhury/env-guardian)

Two binaries ship in one package: `env-guardian` and `config-sync` (same tool, use either).

```bash
env-guardian --help    # full command list
config-sync --help     # same tool, alias binary
```

---

## English

### What does it do?

| Problem | Solution |
|---------|----------|
| Missing env keys in `.env` | `check` compares `.env` vs `.env.example` |
| Secrets used in code but not defined | `check` scans your codebase |
| Accidentally committing `.env` to git | `hook install` blocks commits |
| Sharing secrets with teammates | `share create` (E2E encrypted) |
| Local vs server config mismatch | `drift check` |
| Multiple environments (dev/staging/prod) | `-p development` profile flag |

### Prerequisites

| Method | Requirement |
|--------|-------------|
| `cargo install` | [Rust](https://rustup.rs) + `~/.cargo/bin` in PATH |
| Binary download | No Rust required |
| `hook install` | Git repo (`git init`) |
| Drift (Vercel) | `VERCEL_TOKEN` env var |
| Drift (AWS) | AWS CLI + credentials |

### Installation

**Option A — cargo install (recommended)**

```bash
cargo install env-guardian
export PATH="$HOME/.cargo/bin:$PATH"   # add to ~/.zshrc if needed
env-guardian --version
```

**Option B — Download binary**

1. [GitHub Releases](https://github.com/ahadalichowdhury/env-guardian/releases)
2. Pick your platform:

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

**Option C — From source**

```bash
git clone https://github.com/ahadalichowdhury/env-guardian.git
cd env-guardian && cargo install --path .
```

### Quick start

```bash
cd your-project
env-guardian init --with-example
cp .env.example .env          # skip if .env already exists
# fill in real values in .env
env-guardian check
git init && env-guardian hook install
```

| File | Commit to git? |
|------|----------------|
| `.envguardian.toml` | Yes |
| `.env.example` | Yes |
| `.env` | **Never** |
| `.env.enc` | Yes (encrypted) |

### Common commands

```bash
env-guardian check                    # validate keys
env-guardian check --strict           # CI mode (warnings = fail)
env-guardian check -p production      # profile
env-guardian encrypt / decrypt        # local vault
env-guardian tui                      # interactive editor
env-guardian hook install             # block .env commits
env-guardian ci install               # GitHub Actions workflow
env-guardian drift check --snapshot .envguardian.snapshot
env-guardian share keygen -o ./keys
```

### Help

```bash
env-guardian --help
env-guardian check --help
env-guardian drift check --help
env-guardian share create --help
```

### Troubleshooting

| Issue | Fix |
|-------|-----|
| `command not found` | `export PATH="$HOME/.cargo/bin:$PATH"` |
| `check` MISSING keys | Add keys from `.env.example` to `.env` |
| `check` EXTRA keys | Add keys from `.env` to `.env.example` |
| `not a git repository` | Run `git init` before `hook install` |
| `decryption failed` | Wrong master password |

---

## বাংলা

### এটি কী করে?

| সমস্যা | সমাধান |
|--------|--------|
| `.env`-এ key মিসিং | `check` — `.env` ও `.env.example` মিলায় |
| কোডে ব্যবহৃত secret define নেই | `check` — কোডবেস স্ক্যান |
| ভুলবশত `.env` git-তে commit | `hook install` — commit ব্লক |
| টিমমেটের সাথে secret শেয়ার | `share create` — E2E এনক্রিপ্ট |
| লোকাল vs সার্ভার config আলাদা | `drift check` |
| dev / staging / prod এনভায়রনমেন্ট | `-p development` profile |

### প্রয়োজনীয়তা

| পদ্ধতি | কী লাগবে |
|--------|---------|
| `cargo install` | Rust + PATH-এ `~/.cargo/bin` |
| Binary download | Rust লাগবে না |
| `hook install` | Git repo |
| Drift (Vercel) | `VERCEL_TOKEN` |
| Drift (AWS) | AWS CLI |

### ইনস্টল

```bash
cargo install env-guardian

# PATH সেট (Mac/Linux) — স্থায়ী করতে ~/.zshrc-তে যোগ করুন
export PATH="$HOME/.cargo/bin:$PATH"
env-guardian --version
```

Binary: [GitHub Releases](https://github.com/ahadalichowdhury/env-guardian/releases) থেকে ডাউনলোড করুন।

### দ্রুত শুরু

```bash
cd your-project
env-guardian init --with-example
cp .env.example .env          # .env নেই তাহলে
# .env-তে real values ভরুন
env-guardian check
git init && env-guardian hook install
```

| ফাইল | Git-তে commit? |
|------|----------------|
| `.envguardian.toml` | হ্যাঁ |
| `.env.example` | হ্যাঁ |
| `.env` | **কখনো না** |
| `.env.enc` | হ্যাঁ (এনক্রিপ্টেড) |

### `check` ফেইল হলে কী করবেন?

| Output | মানে | করণীয় |
|--------|------|--------|
| **MISSING** | `.env.example`-এ আছে, `.env`-এ নেই | `.env`-তে key যোগ করুন |
| **EXTRA** | `.env`-এ আছে, `.env.example`-এ নেই | `.env.example`-তে key যোগ করুন |
| **EMPTY** | key আছে, value খালি | value ভরুন |

### মূল কমান্ড

```bash
env-guardian check                    # key মিলানো
env-guardian check --strict           # CI — warning-ও fail
env-guardian check -p production      # প্রোফাইল
env-guardian encrypt                  # .env → .env.enc
env-guardian decrypt                  # .env.enc → .env
env-guardian tui                      # ইন্টারেক্টিভ এডিটর
env-guardian hook install             # .env commit ব্লক
env-guardian ci install               # GitHub Actions
env-guardian drift check --snapshot .envguardian.snapshot
env-guardian share keygen -o ./keys   # E2E শেয়ারিং কী
```

### Help দেখুন

```bash
env-guardian --help
config-sync --help          # একই টুল
env-guardian check --help
env-guardian encrypt --help
```

### সাধারণ সমস্যা

| সমস্যা | সমাধান |
|--------|--------|
| `command not found` | `export PATH="$HOME/.cargo/bin:$PATH"` |
| `check` fail — MISSING | `.env`-তে key যোগ করুন |
| `check` fail — EXTRA | `.env.example`-তে key যোগ করুন |
| `not a git repository` | `git init` চালান |
| `decryption failed` | ভুল master password |

### গুরুত্বপূর্ণ নিয়ম

- `.env` **কখনো** git-তে commit করবেন না
- `.env.example` commit করা নিরাপদ
- `.env.enc` commit করা নিরাপদ (এনক্রিপ্টেড)
- `env-guardian.key` **কখনো** শেয়ার করবেন না

---

## All commands (English + বাংলা)

| Command | English | বাংলা |
|---------|---------|-------|
| `init` | Setup project config | প্রজেক্ট সেটআপ |
| `check` | Validate `.env` keys | key মিলানো |
| `encrypt` | Lock `.env` as `.env.enc` | এনক্রিপ্ট |
| `decrypt` | Restore `.env` from `.env.enc` | ডিক্রিপ্ট |
| `hook install` | Block `.env` git commits | git leak বন্ধ |
| `tui` | Interactive terminal editor | টার্মিনাল এডিটর |
| `ci install` | GitHub Actions workflow | CI সেটআপ |
| `drift check` | Compare local vs remote | drift চেক |
| `drift snapshot` | Save env snapshot | snapshot সেভ |
| `share keygen` | Generate E2E keypair | কী জেনারেট |
| `share create` | Encrypt for teammate | টিমমেটের জন্য এনক্রিপ্ট |
| `share open` | Decrypt received share | share ডিক্রিপ্ট |

Global: `--root <path>` — project folder (default: current directory)

### Profiles

| Profile | Env file | Encrypted |
|---------|----------|-----------|
| `default` | `.env` | `.env.enc` |
| `development` | `.env.development` | `.env.development.enc` |
| `staging` | `.env.staging` | `.env.staging.enc` |
| `production` | `.env.production` | `.env.production.enc` |

### TUI keys

| Key | English | বাংলা |
|-----|---------|-------|
| `j` / `k` | Navigate | উপর / নিচ |
| `Enter` | Edit value | এডিট |
| `n` | New key | নতুন key |
| `d` | Delete key | ডিলিট |
| `p` | Switch profile | প্রোফাইল বদল |
| `c` | Run check | চেক |
| `q` | Quit | বের হন |

---

## Team sharing (E2E)

```bash
env-guardian share keygen -o ./keys
env-guardian share create --recipient teammate.pub -p production -o prod.share
env-guardian share open --share prod.share --key ./keys/env-guardian.key -o .env.production
```

টিমমেটের সাথে `prod.share` ফাইল পাঠান — শুধু recipient-ের private key দিয়ে খুলা যাবে।

---

## Development

```bash
cargo test
cargo build --release
```

## License

MIT
