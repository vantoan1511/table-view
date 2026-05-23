import * as Neutralino from '@neutralinojs/lib';
import { defineStore } from 'pinia';
import { ref } from 'vue';

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

  const MANIFEST_URL =
    'https://raw.githubusercontent.com/vantoan1511/table-view/main/manifest.json';

  const RELEASE_URL = 'https://api.github.com/repos/vantoan1511/table-view/releases/tags';

  const init = async () => {
    // Cleanup update-related files on startup
    if (window.NL_PORT) {
      try {
        const data = await Neutralino.storage.getData('updater_config');
        const config = JSON.parse(data);
        ignoredVersion.value = config.ignoredVersion || null;
      } catch (err) {
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
      const manifest = (await Neutralino.updater.checkForUpdates(MANIFEST_URL)) as UpdateManifest;
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

  const isNewerVersion = (latest: string, current: string) => {
    const l = latest.split('.').map(Number);
    const c = current.split('.').map(Number);
    for (let i = 0; i < 3; i++) {
      if ((l[i] || 0) > (c[i] || 0)) return true;
      if ((l[i] || 0) < (c[i] || 0)) return false;
    }
    return false;
  };

  return {
    isChecking,
    isUpdating,
    updateAvailable,
    updateStatus,
    error,
    ignoredVersion,
    init,
    checkForUpdates,
    installUpdates,
    ignoreUpdate
  };
});
