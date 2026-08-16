# LiteSVM Rust tests (Anchor.toml default)
rust:
    anchor test

rust-logs:
    anchor test --script test-rust-logs

# TypeScript Mocha tests. Surfpool is started here because
# skip_local_validator = true keeps `anchor test` in-process for Rust.
ts:
    #!/usr/bin/env bash
    set -euo pipefail
    yarn
    # start surfpool with offline mode (do not fork mainnet, it takes longer and this script will fail as it for 8899 port answer not for full surfpool runbook to finish)
    NO_DNA=1 surfpool start --offline --yes --no-tui --no-studio & trap 'kill $! 2>/dev/null || true' EXIT
    # NO_DNA=1 surfpool start --no-tui --no-studio & trap 'kill $! 2>/dev/null || true' EXIT
    until curl -s http://127.0.0.1:8899 >/dev/null; do sleep 0.2; done
    anchor test --script test-ts --skip-local-validator --skip-deploy

# TODO: add target to run on alredy started surfpool for transaction inspection