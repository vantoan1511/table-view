import { DB_BRIDGE_EXTENSION_ID, events, extensions, isAvailable } from './nativeService';

/**
 * BridgeService centralizes communication with the NeutralinoJS Rust backend.
 * It provides a promise-based API for extension dispatches, reducing boilerplate
 * and improving error handling consistency.
 */
export class BridgeService {
  /**
   * Dispatches a command to the Rust extension and waits for a specific result event.
   */
  static async request<T = any>(
    command: string,
    resultEvent: string,
    payload: any = {}
  ): Promise<T> {
    if (!isAvailable()) {
      throw new Error('Neutralino runtime not available');
    }

    const reqId = Date.now().toString() + Math.random().toString(36).slice(2, 5);

    return new Promise((resolve, reject) => {
      const onResult = (data: any) => {
        if (data?.reqId === reqId) {
          events.off(resultEvent, onResult);
          if (data.success) {
            resolve(data);
          } else {
            reject(new Error(data.error || `Request failed: ${command}`));
          }
        }
      };

      events.on(resultEvent, onResult);

      extensions
        .dispatch(DB_BRIDGE_EXTENSION_ID, command, {
          ...payload,
          reqId
        })
        .catch((err) => {
          events.off(resultEvent, onResult);
          reject(err);
        });
    });
  }
}
