# Compile the on-chain program and IDL.
build:
    NO_DNA=1 anchor build

# LiteSVM Rust tests (Anchor.toml default)
rust:
    anchor test
    
# Run Rust tests with logs.
rust-logs:
    anchor test --script test-rust-logs

# Start a standalone surfnet in this terminal (no TUI). Pair with `just ts-surfpool`.
surfpool:
    surfpool start --no-tui

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

# TypeScript Mocha tests against a surfpool already running (e.g. `just surfpool`).
ts-surfpool:
    #!/usr/bin/env bash
    set -euo pipefail
    yarn
    if ! curl -s http://127.0.0.1:8899 >/dev/null; then
        echo "error: surfpool is not reachable at http://127.0.0.1:8899" >&2
        echo "start it in another terminal with: just surfpool" >&2
        exit 1
    fi
    anchor test --script test-ts --skip-local-validator --skip-deploy
