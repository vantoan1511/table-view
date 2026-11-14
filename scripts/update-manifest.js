import fs from 'fs';
import { pathToFileURL } from 'url';

export const MAX_LEGACY_MANIFEST_VERSION = '0.7.9';
const RELEASE_DOWNLOAD_BASE_URL = 'https://github.com/vantoan1511/table-view/releases/download';

const SEMVER_REGEX = /^(\d+)\.(\d+)\.(\d+)(?:-([0-9A-Za-z.-]+))?(?:\+[0-9A-Za-z.-]+)?$/;

export function parseStableSemver(rawVersion) {
  if (typeof rawVersion !== 'string') {
    throw new Error(`Invalid version string: "${String(rawVersion)}"`);
  }
  const clean = rawVersion.trim().replace(/^v/i, '');
  const match = SEMVER_REGEX.exec(clean);
  if (!match) {
    throw new Error(`Invalid version string: "${rawVersion}"`);
  }

  const major = parseInt(match[1], 10);
  const minor = parseInt(match[2], 10);
  const patch = parseInt(match[3], 10);
  const isPrerelease = Boolean(match[4]);

  return {
    major,
    minor,
    patch,
    isPrerelease,
    normalized: `${major}.${minor}.${patch}`
  };
}

const CAP_SEMVER = parseStableSemver(MAX_LEGACY_MANIFEST_VERSION);

function exceedsCap(parsed) {
  if (parsed.major !== CAP_SEMVER.major) return parsed.major > CAP_SEMVER.major;
  if (parsed.minor !== CAP_SEMVER.minor) return parsed.minor > CAP_SEMVER.minor;
  return parsed.patch > CAP_SEMVER.patch;
}

export function resolveLegacyManifestUpdate(manifest, targetVersion) {
  const parsed = parseStableSemver(targetVersion);

  if (parsed.isPrerelease) {
    return {
      updated: false,
      reason: `Skipped prerelease version ${targetVersion}`,
      manifest
    };
  }

  if (exceedsCap(parsed)) {
    return {
      updated: false,
      reason: `Skipped version ${parsed.normalized} (exceeds legacy cap ${MAX_LEGACY_MANIFEST_VERSION})`,
      manifest
    };
  }

  const tag = `v${parsed.normalized}`;
  return {
    updated: true,
    reason: `Updated manifest to ${parsed.normalized}`,
    manifest: {
      ...manifest,
      version: parsed.normalized,
      resourcesURL: `${RELEASE_DOWNLOAD_BASE_URL}/${tag}/resources.neu`,
      data: {
        ...manifest.data,
        extensionUrl: `${RELEASE_DOWNLOAD_BASE_URL}/${tag}/db-bridge.exe`
      }
    }
  };
}

function runCli() {
  const args = process.argv.slice(2);
  if (args[0] === '--check') {
    const version = args[1];
    if (!version) {
      console.error('Usage: node scripts/update-manifest.js --check <version>');
      process.exit(1);
    }
    const parsed = parseStableSemver(version);
    const shouldUpdate = !parsed.isPrerelease && !exceedsCap(parsed);
    console.log(shouldUpdate ? 'true' : 'false');
    return;
  }

  const version = args[0];
  const manifestPath = args[1] || 'manifest.json';
  if (!version) {
    console.error('Usage: node scripts/update-manifest.js <version> [manifestPath]');
    process.exit(1);
  }

  const raw = fs.readFileSync(manifestPath, 'utf8');
  const manifest = JSON.parse(raw);
  const result = resolveLegacyManifestUpdate(manifest, version);

  if (result.updated) {
    fs.writeFileSync(manifestPath, JSON.stringify(result.manifest, null, 2) + '\n', 'utf8');
  }
  console.log(result.reason);
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  runCli();
}
