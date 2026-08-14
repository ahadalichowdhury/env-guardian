#!/usr/bin/env bash
# Remove env-guardian / config-sync from cargo bin and clear session alias.
set -euo pipefail

echo "Removing cargo binaries..."
rm -f "$HOME/.cargo/bin/env-guardian" "$HOME/.cargo/bin/config-sync"

# Remove shell alias if set in current instructions (session or zshrc)
if alias env-guardian 2>/dev/null; then
  unalias env-guardian 2>/dev/null || true
  echo "Removed shell alias: env-guardian"
fi

if grep -q "alias env-guardian" "$HOME/.zshrc" 2>/dev/null; then
  echo "Found alias in ~/.zshrc — remove that line manually if needed."
fi

echo "Done. Fresh install:"
echo "  cargo install env-guardian --version 0.1.1 --force"
echo "  export PATH=\"\$HOME/.cargo/bin:\$PATH\""
echo "  env-guardian --version"
