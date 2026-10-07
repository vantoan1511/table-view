import fs from 'fs';
import path from 'path';

const distDir = path.join(process.cwd(), 'dist', 'table-view');

const allowedFiles = ['bin', 'resources.neu', 'table-view-win_x64.exe', 'icon.ico'];

if (fs.existsSync(distDir)) {
  console.log('Cleaning up unnecessary binaries from dist...');
  const files = fs.readdirSync(distDir);
  for (const file of files) {
    if (!allowedFiles.includes(file)) {
      const filePath = path.join(distDir, file);
      if (fs.statSync(filePath).isDirectory()) {
        fs.rmSync(filePath, { recursive: true, force: true });
      } else {
        fs.unlinkSync(filePath);
      }
      console.log(`Removed from dist: ${file}`);
    }
  }

  const binDir = path.join(distDir, 'bin');
  if (fs.existsSync(binDir)) {
    const binFiles = fs.readdirSync(binDir);
    for (const file of binFiles) {
      if (file.startsWith('neutralino-')) {
        const filePath = path.join(binDir, file);
        fs.unlinkSync(filePath);
        console.log(`Removed from bin: ${file}`);
      }
    }
  }

  console.log('Dist directory cleanup complete.');

  // Validate mandatory artifacts exist
  const isWin = process.platform === 'win32';
  const bridgeBinary = isWin ? path.join('bin', 'db-bridge.exe') : path.join('bin', 'db-bridge');
  const requiredFiles = ['resources.neu', bridgeBinary];

  for (const requiredFile of requiredFiles) {
    const requiredPath = path.join(distDir, requiredFile);
    if (!fs.existsSync(requiredPath)) {
      console.error(`Fatal: Required distribution artifact is missing: ${requiredPath}`);
      process.exit(1);
    }
  }
  console.log('Distribution artifacts validated successfully.');
} else {
  console.error('Fatal: Dist directory not found at ' + distDir);
  process.exit(1);
}
