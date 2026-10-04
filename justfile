# Cross-platform task runner for humans and agents. `just --list` shows every
# recipe. Keep each recipe a thin call to a portable tool or a script under
# scripts/. See https://github.com/simpsonm09-org/simpsonm09-repo-standard/blob/main/docs/task-runner.md
set windows-shell := ["powershell.exe", "-NoLogo", "-NoProfile", "-Command"]

# List the recipes.
default:
    @just --list

# Install the pinned tools.
install:
    mise install

# Fetch the crate dependencies from the lockfile.
deps:
    mise exec -- cargo fetch

# Run every linter over the tracked files.
lint:
    mise exec -- flint run --full

# Fix what the linters can fix.
lint-fix:
    mise exec -- flint run --fix

# Run the AI-slop gate.
aislop:
    npx --yes aislop@0.16.1 ci

# Run the test suite. `mise run test` holds the repository test command.
test:
    mise run test

# Run the tests and write an lcov report to coverage/lcov.info.
coverage:
    mise exec -- node scripts/coverage.mjs

# Regenerate docs/openapi.json from the handlers and the DTOs.
spec:
    mise exec -- cargo run --quiet --bin write-openapi

# Serve the API on http://127.0.0.1:3000.
serve:
    mise run serve

# Lint and test.
verify: lint test

# Prune remote-tracking refs and delete local branches merged into main.
prune:
    node scripts/prune.mjs
