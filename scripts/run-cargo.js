const { existsSync } = require('node:fs');
const { join } = require('node:path');
const { spawnSync } = require('node:child_process');

function resolveCargo() {
  const candidates = [];

  if (process.env.CARGO) {
    candidates.push(process.env.CARGO);
  }

  if (process.env.USERPROFILE) {
    candidates.push(join(process.env.USERPROFILE, '.cargo', 'bin', 'cargo.exe'));
    candidates.push(join(process.env.USERPROFILE, '.cargo', 'bin', 'cargo'));
  }

  candidates.push('cargo');

  for (const candidate of candidates) {
    if (candidate === 'cargo' || existsSync(candidate)) {
      return candidate;
    }
  }

  return null;
}

const cargo = resolveCargo();

if (!cargo) {
  console.error('cargo was not found. Install Rust or set the CARGO environment variable.');
  process.exit(1);
}

const result = spawnSync(cargo, process.argv.slice(2), {
  stdio: 'inherit',
  shell: false,
});

if (result.error) {
  console.error(result.error.message);
  process.exit(1);
}

process.exit(result.status ?? 0);
