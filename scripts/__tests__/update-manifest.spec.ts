import fs from 'fs';
import path from 'path';
import { describe, it, expect } from 'vitest';
import { resolveLegacyManifestUpdate } from '../update-manifest.js';

describe('update-manifest', () => {
  const baseManifest = {
    applicationId: 'com.github.vantoan1511.tableview',
    version: '0.7.1',
    resourcesURL:
      'https://github.com/vantoan1511/table-view/releases/download/v0.7.1/resources.neu',
    data: {
      extensionUrl:
        'https://github.com/vantoan1511/table-view/releases/download/v0.7.1/db-bridge.exe',
      releaseNotes: 'notes'
    }
  };

  it('updates version and pins tag download URLs for valid releases <= 0.7.9', () => {
    const res072 = resolveLegacyManifestUpdate(baseManifest, '0.7.2');
    expect(res072.updated).toBe(true);
    expect(res072.manifest.version).toBe('0.7.2');
    expect(res072.manifest.resourcesURL).toBe(
      'https://github.com/vantoan1511/table-view/releases/download/v0.7.2/resources.neu'
    );
    expect(res072.manifest.data.extensionUrl).toBe(
      'https://github.com/vantoan1511/table-view/releases/download/v0.7.2/db-bridge.exe'
    );
    expect(res072.manifest.data.releaseNotes).toBe('notes');

    const res079 = resolveLegacyManifestUpdate(baseManifest, 'v0.7.9');
    expect(res079.updated).toBe(true);
    expect(res079.manifest.version).toBe('0.7.9');
    expect(res079.manifest.resourcesURL).toBe(
      'https://github.com/vantoan1511/table-view/releases/download/v0.7.9/resources.neu'
    );
    expect(res079.manifest.data.extensionUrl).toBe(
      'https://github.com/vantoan1511/table-view/releases/download/v0.7.9/db-bridge.exe'
    );
    expect(res079.manifest.data.releaseNotes).toBe('notes');
  });

  it('skips manifest update when target version exceeds 0.7.9', () => {
    for (const version of ['0.8.0', 'v0.8.0', '0.7.10', '1.0.0']) {
      const res = resolveLegacyManifestUpdate(baseManifest, version);
      expect(res.updated).toBe(false);
      expect(res.manifest).toEqual(baseManifest);
    }
  });

  it('skips manifest update for prerelease versions', () => {
    for (const version of ['0.7.9-preview.1', '0.8.0-preview.1', '0.7.2-rc.1']) {
      const res = resolveLegacyManifestUpdate(baseManifest, version);
      expect(res.updated).toBe(false);
      expect(res.manifest).toEqual(baseManifest);
    }
  });

  it('throws on invalid version string', () => {
    for (const version of ['', 'invalid', '0.7']) {
      expect(() => resolveLegacyManifestUpdate(baseManifest, version)).toThrow();
    }
  });

  it('manifest.json pins versioned tag URLs and does not use /releases/latest/download/', () => {
    const manifestPath = path.resolve(process.cwd(), 'manifest.json');
    const manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf8'));

    expect(manifest.version).toBe('0.7.1');
    expect(manifest.resourcesURL).toContain(`/releases/download/v${manifest.version}/`);
    expect(manifest.data.extensionUrl).toContain(`/releases/download/v${manifest.version}/`);
    expect(manifest.resourcesURL).not.toContain('/releases/latest/');
    expect(manifest.data.extensionUrl).not.toContain('/releases/latest/');
  });
});
