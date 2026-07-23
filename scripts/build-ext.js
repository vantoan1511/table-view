import { execSync } from 'child_process';
import { copyFileSync, existsSync, mkdirSync } from 'fs';
import { platform } from 'os';
import { join } from 'path';

console.log('Building TableView Binaries (db-bridge)...');

// Building TableView Workspace in extensions/db-bridge directory
execSync('cargo build --release', { cwd: join('extensions', 'db-bridge'), stdio: 'inherit' });

// Create the bin directory
const binDir = 'bin';
if (!existsSync(binDir)) {
  mkdirSync(binDir);
}

// Copy the binaries
const ext = platform() === 'win32' ? '.exe' : '';

const binaries = ['db-bridge'];

for (const binName of binaries) {
  const binaryName = `${binName}${ext}`;
  const src = join('extensions', 'db-bridge', 'target', 'release', binaryName);
  const dest = join(binDir, binaryName);

  console.log(`Copying binary from ${src} to ${dest}...`);
  copyFileSync(src, dest);
}

console.log('TableView Binaries built successfully!');
