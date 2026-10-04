#!/usr/bin/env node
// cargo-llvm-cov --output-path does not create its parent directory, so make
// coverage/ first, then write the lcov report the fleet patch-coverage gate reads.

import { spawnSync } from 'node:child_process';
import { mkdirSync } from 'node:fs';

mkdirSync('coverage', { recursive: true });

// Branch coverage passes the unstable `-Z coverage-options=branch` to rustc and
// the repository pins a stable toolchain, so allow that one unstable flag for
// this run only. The lcov then carries BRDA records.
const result = spawnSync('cargo', ['llvm-cov', '--branch', '--lcov', '--output-path', 'coverage/lcov.info'], {
  stdio: 'inherit',
  env: { ...process.env, RUSTC_BOOTSTRAP: '1' },
});
if (result.error) {
  process.stderr.write(`coverage: cannot run cargo: ${result.error.message}\n`);
  process.exit(1);
}
process.exit(result.status ?? 1);
