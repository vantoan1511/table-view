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
} else {
  console.log('Dist directory not found, skipping cleanup.');
}
