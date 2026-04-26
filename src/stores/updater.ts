import * as Neutralino from '@neutralinojs/lib'
import { defineStore } from 'pinia'
import { ref } from 'vue'

export interface UpdateManifest {
  applicationId: string
  version: string
  resourcesURL: string
  data: {
    extensionVersion: string
    extensionUrlWin: string
    extensionUrlMac: string
    extensionUrlLinux: string
    releaseNotes?: string
  }
}

export const useUpdaterStore = defineStore('updater', () => {
  const isChecking = ref(false)
  const isUpdating = ref(false)
  const updateAvailable = ref<UpdateManifest | null>(null)
  const updateStatus = ref('')
  const error = ref<string | null>(null)

  const MANIFEST_URL = 'https://raw.githubusercontent.com/vantoan1511/table-view/main/manifest.json'

  const init = async () => {
    // Cleanup old extension binaries from previous updates
    if (window.NL_PORT) {
      try {
        const platform = window.NL_OS.toLowerCase()
        const extPath = platform === 'windows'
          ? 'extensions\\db-bridge\\db-bridge.exe.old'
          : 'extensions/db-bridge/db-bridge.old'

        await Neutralino.filesystem.remove(extPath).catch(() => {
          // Ignore error if file doesn't exist
        })
      } catch (err) {
        console.warn('Cleanup failed:', err)
      }
    }
  }

  const checkForUpdates = async (manual = false) => {
    if (isChecking.value || !window.NL_PORT) return

    isChecking.value = true
    error.value = null

    try {
      // 1. Check manifest
      const manifest = await Neutralino.updater.checkForUpdates(MANIFEST_URL) as UpdateManifest

      const currentAppVersion = window.NL_APPVERSION
      const currentExtVersion = localStorage.getItem('db_extension_version') || '0.0.0'

      const appNeedsUpdate = isNewerVersion(manifest.version, currentAppVersion)
      const extNeedsUpdate = isNewerVersion(manifest.data.extensionVersion, currentExtVersion)

      if (appNeedsUpdate || extNeedsUpdate) {
        updateAvailable.value = manifest
      } else if (manual) {
        // Notify user they are up to date
      }
    } catch (err: any) {
      console.error('Update check failed:', err)
      if (manual) error.value = 'Failed to check for updates: ' + (err.message || err)
    } finally {
      isChecking.value = false
    }
  }

  const installUpdates = async () => {
    if (!updateAvailable.value || isUpdating.value) return

    isUpdating.value = true
    updateStatus.value = 'Initializing update...'

    try {
      const manifest = updateAvailable.value
      const currentAppVersion = window.NL_APPVERSION
      const currentExtVersion = localStorage.getItem('db_extension_version') || '0.0.0'

      // 1. Update Extension first (custom updater)
      if (isNewerVersion(manifest.data.extensionVersion, currentExtVersion)) {
        updateStatus.value = 'Downloading database engine update...'

        let downloadUrl = ''
        const platform = window.NL_OS.toLowerCase()
        if (platform === 'windows') downloadUrl = manifest.data.extensionUrlWin
        else if (platform === 'darwin') downloadUrl = manifest.data.extensionUrlMac
        else downloadUrl = manifest.data.extensionUrlLinux

        if (downloadUrl) {
          await triggerExtensionUpdate(downloadUrl)
          localStorage.setItem('db_extension_version', manifest.data.extensionVersion)
        }
      }

      // 2. Update Frontend (.neu file)
      if (isNewerVersion(manifest.version, currentAppVersion)) {
        updateStatus.value = 'Installing UI updates...'
        await Neutralino.updater.install()
      }

      updateStatus.value = 'Update complete! Restarting application...'

      // Give the user a moment to see the completion message
      setTimeout(async () => {
        await Neutralino.app.restartProcess()
      }, 2000)

    } catch (err: any) {
      console.error('Update failed:', err)
      error.value = 'Update failed: ' + (err.message || err)
      isUpdating.value = false
    }
  }

  const triggerExtensionUpdate = (downloadUrl: string): Promise<void> => {
    return new Promise((resolve, reject) => {
      const reqId = Date.now().toString()

      const onResult = (evt: any) => {
        const payload = evt.detail
        if (payload.reqId !== reqId) return

        Neutralino.events.off('dbBridge.updateExtensionResult', onResult)
        if (payload.success) resolve()
        else reject(new Error(payload.error))
      }

      Neutralino.events.on('dbBridge.updateExtensionResult', onResult)
      Neutralino.extensions.dispatch('com.github.vantoan1511.table-view.db-bridge', 'dbBridge.updateExtension', {
        reqId,
        downloadUrl
      })
    })
  }

  const isNewerVersion = (latest: string, current: string) => {
    const l = latest.split('.').map(Number)
    const c = current.split('.').map(Number)
    for (let i = 0; i < 3; i++) {
      if ((l[i] || 0) > (c[i] || 0)) return true
      if ((l[i] || 0) < (c[i] || 0)) return false
    }
    return false
  }

  return {
    isChecking,
    isUpdating,
    updateAvailable,
    updateStatus,
    error,
    init,
    checkForUpdates,
    installUpdates
  }
})
