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
    # surfpool inherits this shell's stdout/stderr, so its logs would interleave with
    # the mocha reporter. Send them to a file instead and only surface it on failure.
    log=target/surfpool.log
    mkdir -p "$(dirname "$log")"
    : > "$log"
    # start surfpool with offline mode (do not fork mainnet, it takes longer and this script will fail as it for 8899 port answer not for full surfpool runbook to finish)
    NO_DNA=1 surfpool start --offline --yes --no-tui --no-studio >"$log" 2>&1 &
    surfpool_pid=$!
    trap 'kill ${surfpool_pid} 2>/dev/null || true' EXIT
    # NO_DNA=1 surfpool start --no-tui --no-studio >"$log" 2>&1 &
    echo "surfpool logs -> $log"
    until curl -s http://127.0.0.1:8899 >/dev/null; do
        if ! kill -0 ${surfpool_pid} 2>/dev/null; then
            echo "error: surfpool exited early, last lines of $log:" >&2
            tail -n 40 "$log" >&2
            exit 1
        fi
        sleep 0.2
    done
    # the RPC answers before the deployment runbook finishes, so wait for the program to be executable
    just wait-for-deployment
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
    just wait-for-deployment
    anchor test --script test-ts --skip-local-validator --skip-deploy

# Block until the deployment runbook made the program executable on the surfnet.
[private]
wait-for-deployment:
    #!/usr/bin/env bash
    set -euo pipefail
    program_id=$(grep -m1 '^r3_session_keys' Anchor.toml | cut -d'"' -f2)
    for _ in $(seq 1 120); do
        if curl -s http://127.0.0.1:8899 -X POST -H 'Content-Type: application/json' \
            -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"getAccountInfo\",\"params\":[\"${program_id}\",{\"encoding\":\"base64\"}]}" \
            | grep -q '"executable":true'; then
            exit 0
        fi
        sleep 0.5
    done
    echo "error: ${program_id} was not deployed on the surfnet after 60s" >&2
    exit 1
