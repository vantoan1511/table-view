import * as Neutralino from '@neutralinojs/lib'

/**
 * BridgeService centralizes communication with the NeutralinoJS Rust backend.
 * It provides a promise-based API for extension dispatches, reducing boilerplate
 * and improving error handling consistency.
 */
export class BridgeService {
  private static EXTENSION_ID = 'com.github.vantoan1511.table-view.db-bridge'

  /**
   * Dispatches a command to the Rust extension and waits for a specific result event.
   */
  static async request<T = any>(command: string, resultEvent: string, payload: any = {}): Promise<T> {
    if (!window.NL_PORT) {
      throw new Error('Neutralino runtime not available')
    }

    const reqId = Date.now().toString() + Math.random().toString(36).slice(2, 5)
    
    return new Promise((resolve, reject) => {
      const onResult = (evt: any) => {
        const data = evt.detail
        if (data.reqId === reqId) {
          Neutralino.events.off(resultEvent, onResult)
          if (data.success) {
            resolve(data)
          } else {
            reject(new Error(data.error || `Request failed: ${command}`))
          }
        }
      }

      Neutralino.events.on(resultEvent, onResult)
      
      Neutralino.extensions.dispatch(this.EXTENSION_ID, command, {
        ...payload,
        reqId
      }).catch(err => {
        Neutralino.events.off(resultEvent, onResult)
        reject(err)
      })
    })
  }
}
