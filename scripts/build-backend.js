import { execSync } from 'child_process';
import fs from 'fs';
import path from 'path';

const extDir = path.join(process.cwd(), 'extensions', 'db-bridge');
const exeName = process.platform === 'win32' ? 'db-bridge.exe' : 'db-bridge';
const src = path.join(extDir, 'target', 'release', exeName);
const dest = path.join(extDir, exeName);

console.log('--- Building backend extension ---');

try {
  execSync('cargo build --release', { stdio: 'inherit', cwd: extDir });
  fs.copyFileSync(src, dest);
  console.log(`Backend built successfully and copied to: ${dest}`);
} catch (error) {
  console.error('Failed to build backend extension:', error);
  process.exit(1);
}
