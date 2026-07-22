import * as Neutralino from '@neutralinojs/lib';
import { defineStore } from 'pinia';
import { ref } from 'vue';

import { usePreferencesStore } from './preferences';
import { useToastStore } from './toast';

export interface UpdateManifest {
  applicationId: string;
  version: string;
  resourcesURL: string;
  data: {
    extensionUrl: string;
    releaseNotes?: string;
  };
}

export const useUpdaterStore = defineStore('updater', () => {
  const isChecking = ref(false);
  const isUpdating = ref(false);
  const updateAvailable = ref<UpdateManifest | null>(null);
  const updateStatus = ref('');
  const error = ref<string | null>(null);
  const ignoredVersion = ref<string | null>(null);
  const showUpdateDialog = ref(false);

  const STABLE_MANIFEST_URL =
    'https://raw.githubusercontent.com/vantoan1511/table-view/main/manifest.json';
  const PREVIEW_MANIFEST_URL =
    'https://raw.githubusercontent.com/vantoan1511/table-view/main/manifest-preview.json';

  const RELEASE_URL = 'https://api.github.com/repos/vantoan1511/table-view/releases/tags';

  const init = async () => {
    // Cleanup update-related files on startup
    if (window.NL_PORT) {
      try {
        const data = await Neutralino.storage.getData('updater_config');
        const config = JSON.parse(data);
        ignoredVersion.value = config.ignoredVersion || null;
      } catch {
        // Not found or invalid
      }

      try {
        const platform = window.NL_OS.toLowerCase();
        if (platform === 'windows') {
          await Neutralino.filesystem.remove('updater.bat').catch(() => {});
          await Neutralino.filesystem.remove('resources.neu.new').catch(() => {});
          await Neutralino.filesystem
            .remove('extensions\\db-bridge\\db-bridge.exe.new')
            .catch(() => {});
        }
      } catch (err) {
        console.warn('Cleanup failed:', err);
      }
    }
  };

  const checkForUpdates = async (manual = false) => {
    if (isChecking.value || !window.NL_PORT) return;

    isChecking.value = true;
    error.value = null;

    try {
      const preferencesStore = usePreferencesStore();
      const manifestUrl = preferencesStore.settings.optInPreview
        ? PREVIEW_MANIFEST_URL
        : STABLE_MANIFEST_URL;
      const manifest = (await Neutralino.updater.checkForUpdates(manifestUrl)) as UpdateManifest;
      const currentAppVersion = window.NL_APPVERSION;
      const appNeedsUpdate = isNewerVersion(manifest.version, currentAppVersion);

      if (appNeedsUpdate) {
        try {
          const tag = `v${manifest.version}`;
          const response = await fetch(`${RELEASE_URL}/${tag}`);
          if (response.ok) {
            const release = await response.json();
            if (release && typeof release.body === 'string') {
              manifest.data = manifest.data || {};
              manifest.data.releaseNotes = release.body;
            }
          }
        } catch (fetchErr) {
          console.warn('Failed to fetch release notes from GitHub API:', fetchErr);
        }

        if (manual || manifest.version !== ignoredVersion.value) {
          updateAvailable.value = manifest;
          if (manual) {
            showUpdateDialog.value = true;
          } else {
            const toastStore = useToastStore();
            const toastId = toastStore.addToast({
              title: 'Update Available',
              message: `Version ${manifest.version} is ready to install.`,
              severity: 'info',
              ttl: 0,
              actions: [
                {
                  label: 'Dismiss',
                  onClick: () => {
                    toastStore.removeToast(toastId);
                  }
                },
                {
                  label: 'Show Details',
                  primary: true,
                  onClick: () => {
                    showUpdateDialog.value = true;
                    toastStore.removeToast(toastId);
                  }
                }
              ]
            });
          }
        }
      }
    } catch (err: any) {
      console.error('Update check failed:', err);
      if (manual) error.value = 'Failed to check for updates: ' + (err.message || err);
    } finally {
      isChecking.value = false;
    }
  };

  const installUpdates = async () => {
    if (!updateAvailable.value || isUpdating.value) return;

    isUpdating.value = true;
    updateStatus.value = 'Preparing update...';

    try {
      const manifest = updateAvailable.value;
      const platform = window.NL_OS.toLowerCase();

      if (platform !== 'windows') {
        throw new Error('Native atomic update is currently only optimized for Windows.');
      }

      // 1. Download Resources (.neu)
      updateStatus.value = 'Downloading application resources...';
      const resPath = 'resources.neu.new';
      await downloadFileNative(manifest.resourcesURL, resPath);

      // 2. Download Extension (.exe)
      updateStatus.value = 'Downloading database engine...';
      const extPath = 'extensions\\db-bridge\\db-bridge.exe.new';
      await downloadFileNative(manifest.data.extensionUrl, extPath);

      // 3. Determine running executable name dynamically on Windows
      let exeName = 'table-view.exe'; // Standard production fallback
      if (window.NL_PID) {
        try {
          const pid = window.NL_PID;
          const cmd = `powershell -Command "(Get-Process -Id ${pid}).Path"`;
          const res = await Neutralino.os.execCommand(cmd);
          if (res.exitCode === 0 && res.stdOut.trim()) {
            const fullPath = res.stdOut.trim();
            const parts = fullPath.split(/[\\/]/);
            const filename = parts[parts.length - 1];
            if (filename && filename.endsWith('.exe')) {
              exeName = filename;
            }
          }
        } catch (e) {
          console.warn(
            'Failed to dynamically determine running executable name, falling back to table-view.exe',
            e
          );
        }
      }

      // 4. Create Swapper Batch Script
      updateStatus.value = 'Staging installation...';
      const batContent = createSwapperBat(exeName);
      await Neutralino.filesystem.writeFile('updater.bat', batContent);

      updateStatus.value = 'Update staged! Restarting in 3 seconds...';

      // 5. Trigger swapper and exit
      setTimeout(async () => {
        await Neutralino.os.execCommand('cmd /c start /min updater.bat');
        await Neutralino.app.exit();
      }, 3000);
    } catch (err: any) {
      console.error('Update failed:', err);
      error.value = 'Update failed: ' + (err.message || err);
      isUpdating.value = false;
    }
  };

  const ignoreUpdate = async (version: string) => {
    ignoredVersion.value = version;
    if (window.NL_PORT) {
      try {
        await Neutralino.storage.setData(
          'updater_config',
          JSON.stringify({ ignoredVersion: version })
        );
      } catch (err) {
        console.error('Failed to save updater config:', err);
      }
    }
    updateAvailable.value = null; // Close dialog
    showUpdateDialog.value = false;
  };

  const downloadFileNative = async (url: string, dest: string) => {
    // Use PowerShell to bypass CORS and handle redirects reliably
    const cmd = `powershell -Command "Invoke-WebRequest -Uri '${url}' -OutFile '${dest}'"`;
    const res = await Neutralino.os.execCommand(cmd);
    if (res.exitCode !== 0) {
      throw new Error(`Download failed (PowerShell): ${res.stdErr}`);
    }
  };

  const createSwapperBat = (exeName: string) => {
    return `@echo off
echo Finalizing update... Please wait.
timeout /t 2 /nobreak > nul

echo Stopping running processes...
taskkill /f /im db-bridge.exe >nul 2>&1
taskkill /f /im ${exeName} >nul 2>&1
timeout /t 2 /nobreak > nul

:retry_ext
move /y "extensions\\db-bridge\\db-bridge.exe.new" "extensions\\db-bridge\\db-bridge.exe" > nul
if errorlevel 1 (
    echo Waiting for engine to release lock...
    taskkill /f /im db-bridge.exe >nul 2>&1
    timeout /t 2 /nobreak > nul
    goto retry_ext
)

:retry_res
move /y "resources.neu.new" "resources.neu" > nul
if errorlevel 1 (
    echo Waiting for app to release lock...
    timeout /t 2 /nobreak > nul
    goto retry_res
)

echo Update complete! Restarting...
start "" "${exeName}"
del "%~f0" & exit
`;
  };

  const parseVersion = (v: string) => {
    const clean = v.replace(/^v/, '');
    const [versionPart, prereleasePart] = clean.split('-');
    const numParts = (versionPart || '').split('.').map((n) => parseInt(n, 10) || 0);
    let buildNum = 0;
    if (prereleasePart) {
      const match = prereleasePart.match(/\d+/);
      if (match) buildNum = parseInt(match[0], 10);
    }
    return {
      major: numParts[0] || 0,
      minor: numParts[1] || 0,
      patch: numParts[2] || 0,
      isPrerelease: !!prereleasePart,
      buildNum
    };
  };

  const isNewerVersion = (latest: string, current: string) => {
    const l = parseVersion(latest);
    const c = parseVersion(current);

    if (l.major !== c.major) return l.major > c.major;
    if (l.minor !== c.minor) return l.minor > c.minor;
    if (l.patch !== c.patch) return l.patch > c.patch;

    if (l.isPrerelease && c.isPrerelease) {
      return l.buildNum > c.buildNum;
    }

    if (!l.isPrerelease && c.isPrerelease) return true;
    if (l.isPrerelease && !c.isPrerelease) return false;

    return false;
  };

  return {
    isChecking,
    isUpdating,
    updateAvailable,
    showUpdateDialog,
    updateStatus,
    error,
    ignoredVersion,
    init,
    checkForUpdates,
    installUpdates,
    ignoreUpdate
  };
});
