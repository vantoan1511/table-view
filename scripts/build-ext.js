import { execSync } from 'child_process';
import { copyFileSync, existsSync, mkdirSync } from 'fs';
import { platform } from 'os';
import { join } from 'path';

const enableNativeUi =
  process.env.ENABLE_NATIVE_UI === 'true' || process.env.ENABLE_NATIVE_UI === '1';

console.log('Building TableView Binaries (db-bridge)...');

// Building TableView Workspace at repository root (default-members = ["extensions/db-bridge"])
execSync('cargo build --release --locked', {
  stdio: 'inherit'
});

const binaries = ['db-bridge'];

if (enableNativeUi) {
  console.log('Building experimental native desktop window (table-view-desktop)...');
  execSync('cargo build --release --locked -p table-view-desktop --features native-ui', {
    stdio: 'inherit'
  });
  binaries.push('table-view-desktop');
}

// Create the bin directory
const binDir = 'bin';
if (!existsSync(binDir)) {
  mkdirSync(binDir);
}

// Copy the binaries
const ext = platform() === 'win32' ? '.exe' : '';

for (const binName of binaries) {
  const binaryName = `${binName}${ext}`;
  const workspaceSrc = join('target', 'release', binaryName);
  const legacySrc = join('extensions', 'db-bridge', 'target', 'release', binaryName);
  const src = existsSync(workspaceSrc) ? workspaceSrc : legacySrc;
  const dest = join(binDir, binaryName);

  console.log(`Copying binary from ${src} to ${dest}...`);
  copyFileSync(src, dest);
}

console.log('TableView Binaries built successfully!');
