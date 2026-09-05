import { app, events, filesystem, os, storage, updater } from '@/services/nativeService';
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
  const downloadProgress = ref(0);
  const updateAvailable = ref<UpdateManifest | null>(null);
  const updateStatus = ref('');
  const error = ref<string | null>(null);
  const ignoredVersion = ref<string | null>(null);
  const installedVersion = ref<string | null>(null);
  const showUpdateDialog = ref(false);

  const STABLE_MANIFEST_URL =
    'https://raw.githubusercontent.com/vantoan1511/table-view/main/manifest.json';
  const PREVIEW_MANIFEST_URL =
    'https://raw.githubusercontent.com/vantoan1511/table-view/main/manifest-preview.json';

  const RELEASE_URL = 'https://api.github.com/repos/vantoan1511/table-view/releases/tags';

  const getCurrentAppVersion = () => {
    const baseVersion = window.NL_APPVERSION || '0.0.0';
    if (!installedVersion.value) return baseVersion;
    if (
      installedVersion.value === baseVersion ||
      isNewerVersion(installedVersion.value, baseVersion, true)
    ) {
      return installedVersion.value;
    }
    return baseVersion;
  };

  const init = async () => {
    // Cleanup update-related files on startup
    if (window.NL_PORT) {
      try {
        const data = await storage.getData('updater_config');
        const config = JSON.parse(data);
        ignoredVersion.value = config.ignoredVersion || null;
        installedVersion.value = config.installedVersion || null;
      } catch {
        // Not found or invalid
      }

      try {
        const platform = (window.NL_OS || '').toLowerCase();
        if (platform === 'windows') {
          await filesystem.remove?.('updater.bat')?.catch(() => {});
          await filesystem.remove?.('resources.neu.new')?.catch(() => {});
          await filesystem.remove?.('bin\\db-bridge.exe.new')?.catch(() => {});
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
      const manifest = (await updater.checkForUpdates(manifestUrl)) as UpdateManifest;
      const currentAppVersion = getCurrentAppVersion();
      const appNeedsUpdate = isNewerVersion(
        manifest.version,
        currentAppVersion,
        preferencesStore.settings.optInPreview
      );

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
      } else if (manual) {
        const toastStore = useToastStore();
        toastStore.addToast({
          title: 'No Updates Available',
          message: `You are on the latest version (${currentAppVersion}).`,
          severity: 'info',
          ttl: 3000
        });
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
    downloadProgress.value = 0;
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
      try {
        await (filesystem as any).createDirectory('bin');
      } catch {
        // Directory already exists — not an error
      }
      const extPath = 'bin\\db-bridge.exe.new';
      await downloadFileNative(manifest.data.extensionUrl, extPath);

      // 3. Determine running executable name dynamically on Windows
      let exeName = 'table-view.exe'; // Standard production fallback
      if (window.NL_PID) {
        try {
          const pid = window.NL_PID;
          const cmd = `powershell -Command "(Get-Process -Id ${pid}).Path"`;
          const res = await os.execCommand(cmd);
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

      // 4. Persist installed version metadata
      installedVersion.value = manifest.version;
      try {
        await storage.setData(
          'updater_config',
          JSON.stringify({
            ignoredVersion: ignoredVersion.value,
            installedVersion: manifest.version
          })
        );
      } catch (err) {
        console.warn('Failed to persist installed version in storage:', err);
      }

      // 5. Create Swapper Batch Script
      updateStatus.value = 'Staging installation...';
      const batContent = createSwapperBat(exeName);
      await filesystem.writeFile('updater.bat', batContent);

      updateStatus.value = 'Update staged! Restarting in 3 seconds...';

      // 6. Trigger swapper and exit
      setTimeout(async () => {
        await os.execCommand('cmd /c start /min updater.bat');
        await app.exit();
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
        await storage.setData(
          'updater_config',
          JSON.stringify({
            ignoredVersion: version,
            installedVersion: installedVersion.value
          })
        );
      } catch (err) {
        console.error('Failed to save updater config:', err);
      }
    }
    updateAvailable.value = null; // Close dialog
    showUpdateDialog.value = false;
  };

  const encodeScriptToUtf16Base64 = (script: string): string => {
    const buffer = new Uint16Array(script.length);
    for (let i = 0; i < script.length; i++) {
      buffer[i] = script.charCodeAt(i);
    }
    const uint8 = new Uint8Array(buffer.buffer);
    let binary = '';
    for (let i = 0; i < uint8.byteLength; i++) {
      const b = uint8[i];
      if (b !== undefined) {
        binary += String.fromCharCode(b);
      }
    }
    return btoa(binary);
  };

  const downloadFileNative = async (url: string, dest: string) => {
    downloadProgress.value = 0;
    const psScript = `
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
$req = [System.Net.HttpWebRequest]::Create('${url}')
$req.Method = 'GET'
$res = $req.GetResponse()
$total = $res.ContentLength
$stream = $res.GetResponseStream()
$dest = '${dest}'
$dir = Split-Path -Path $dest
if ($dir -and !(Test-Path -Path $dir)) { New-Item -ItemType Directory -Path $dir -Force | Out-Null }
$file = [System.IO.File]::Create($dest)
$buffer = New-Object byte[] 8192
$read = 0
$downloaded = 0
$lastReport = -1
while (($read = $stream.Read($buffer, 0, $buffer.Length)) -gt 0) {
    $file.Write($buffer, 0, $read)
    $downloaded += $read
    if ($total -gt 0) {
        $pct = [math]::Floor(($downloaded / $total) * 100)
        if ($pct -ne $lastReport) {
            Write-Host "PROGRESS:$pct"
            $lastReport = $pct
        }
    }
}
$file.Close()
$res.Close()
`;

    const encodedCommand = encodeScriptToUtf16Base64(psScript);
    const cmd = `powershell -WindowStyle Hidden -EncodedCommand ${encodedCommand}`;

    return new Promise<void>(async (resolve, reject) => {
      let processOutput = '';
      let isResolved = false;
      let procId: number | null = null;

      interface SpawnedProcessData {
        id: number;
        action: 'stdOut' | 'stdErr' | 'exit';
        data: unknown;
      }

      const handler = (evt: any) => {
        const detail: SpawnedProcessData = evt?.detail !== undefined ? evt.detail : evt;
        if (procId !== null && detail?.id === procId) {
          if (detail.action === 'stdOut') {
            const data = String(detail.data ?? '');
            processOutput += data;
            const matches = data.match(/PROGRESS:(\d+)/g);
            if (matches && matches.length > 0) {
              const lastMatch = matches[matches.length - 1];
              if (lastMatch) {
                const pct = parseInt(lastMatch.replace('PROGRESS:', ''), 10);
                if (!isNaN(pct)) {
                  downloadProgress.value = pct;
                }
              }
            }
          } else if (detail.action === 'stdErr') {
            processOutput += String(detail.data ?? '');
          } else if (detail.action === 'exit') {
            events.off('spawnedProcess', handler);
            if (!isResolved) {
              isResolved = true;
              if (detail.data === 0) {
                downloadProgress.value = 100;
                resolve();
              } else {
                reject(
                  new Error(
                    `Download failed with exit code ${detail.data}: ${processOutput.trim()}`
                  )
                );
              }
            }
          }
        }
      };

      try {
        events.on('spawnedProcess', handler);
        const proc = await os.spawnProcess(cmd);
        procId = proc.id;
      } catch (err) {
        events.off('spawnedProcess', handler);
        reject(err);
      }
    });
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
if not exist "bin" mkdir "bin"
if exist "bin\\db-bridge.exe.new" (
    move /y "bin\\db-bridge.exe.new" "bin\\db-bridge.exe" > nul
) else if exist "extensions\\db-bridge\\db-bridge.exe.new" (
    move /y "extensions\\db-bridge\\db-bridge.exe.new" "bin\\db-bridge.exe" > nul
)
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
    const clean = (v || '').trim().replace(/^v/i, '');
    const withoutBuild = clean.split('+')[0] || '';
    const dashIndex = withoutBuild.indexOf('-');
    let versionCore = withoutBuild;
    let prereleaseStr = '';

    if (dashIndex !== -1) {
      versionCore = withoutBuild.slice(0, dashIndex);
      prereleaseStr = withoutBuild.slice(dashIndex + 1);
    }

    const numParts = (versionCore || '').split('.').map((n) => parseInt(n, 10) || 0);
    const major = numParts[0] || 0;
    const minor = numParts[1] || 0;
    const patch = numParts[2] || 0;

    const isPrerelease = prereleaseStr.length > 0;
    const prereleaseIdentifiers: (string | number)[] = isPrerelease
      ? prereleaseStr.split('.').map((id) => (/^\d+$/.test(id) ? parseInt(id, 10) : id))
      : [];

    return {
      major,
      minor,
      patch,
      isPrerelease,
      prereleaseIdentifiers
    };
  };

  const isNewerVersion = (latest: string, current: string, optInPreview = false) => {
    const l = parseVersion(latest);
    const c = parseVersion(current);

    if (l.major !== c.major) return l.major > c.major;
    if (l.minor !== c.minor) return l.minor > c.minor;
    if (l.patch !== c.patch) return l.patch > c.patch;

    if (!l.isPrerelease && c.isPrerelease) return true;
    if (l.isPrerelease && !c.isPrerelease) return optInPreview;

    if (l.isPrerelease && c.isPrerelease) {
      const len = Math.max(l.prereleaseIdentifiers.length, c.prereleaseIdentifiers.length);
      for (let i = 0; i < len; i++) {
        const idL = l.prereleaseIdentifiers[i];
        const idC = c.prereleaseIdentifiers[i];

        if (idL === undefined) return false;
        if (idC === undefined) return true;
        if (idL === idC) continue;

        const typeL = typeof idL;
        const typeC = typeof idC;

        if (typeL === 'number' && typeC === 'number') {
          return (idL as number) > (idC as number);
        }
        if (typeL === 'number' && typeC === 'string') {
          return false;
        }
        if (typeL === 'string' && typeC === 'number') {
          return true;
        }
        if (typeL === 'string' && typeC === 'string') {
          return (idL as string).localeCompare(idC as string) > 0;
        }
      }
    }

    return false;
  };

  return {
    isChecking,
    isUpdating,
    downloadProgress,
    updateAvailable,
    showUpdateDialog,
    updateStatus,
    error,
    ignoredVersion,
    installedVersion,
    getCurrentAppVersion,
    init,
    checkForUpdates,
    installUpdates,
    ignoreUpdate
  };
});
