import { invoke } from '@tauri-apps/api/core';
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
import { VisualizerConfig } from '../types/visualizer';

const CHANNEL_NAME = 'audiowave_preview_sync';

class DetachedPreviewService {
  private channel: BroadcastChannel | null = null;

  constructor() {
    if (typeof BroadcastChannel !== 'undefined') {
      this.channel = new BroadcastChannel(CHANNEL_NAME);
    }
  }

  public async openDetachedPreview(): Promise<void> {
    try {
      await invoke('open_detached_preview_window');
    } catch (err) {
      console.warn('[DetachedPreviewService] Native invoke fallback:', err);
      try {
        const existing = await WebviewWindow.getByLabel('detached-preview');
        if (existing) {
          await existing.setFocus();
          return;
        }
        const webview = new WebviewWindow('detached-preview', {
          url: 'index.html?detached=true',
          title: 'AudioWave Studio - Live Preview',
          width: 1280,
          height: 720,
          resizable: true,
        });
        webview.once('tauri://error', () => {
          window.open(window.location.origin + '?detached=true', 'audiowave-preview', 'width=1280,height=720');
        });
      } catch {
        window.open(window.location.origin + '?detached=true', 'audiowave-preview', 'width=1280,height=720');
      }
    }
  }

  public broadcastConfig(config: VisualizerConfig): void {
    if (this.channel) {
      this.channel.postMessage({ type: 'CONFIG_UPDATE', config });
    }
  }

  public broadcastAudioFrame(freqData: Uint8Array, timeData: Uint8Array): void {
    if (this.channel) {
      this.channel.postMessage({
        type: 'AUDIO_FRAME',
        freqData: Array.from(freqData),
        timeData: Array.from(timeData),
      });
    }
  }

  public listen(
    onConfig: (config: VisualizerConfig) => void,
    onAudioFrame?: (freqData: Uint8Array, timeData: Uint8Array) => void
  ): () => void {
    if (!this.channel) return () => {};

    const handler = (event: MessageEvent) => {
      const data = event.data;
      if (!data) return;
      if (data.type === 'CONFIG_UPDATE' && data.config) {
        onConfig(data.config);
      } else if (data.type === 'AUDIO_FRAME' && onAudioFrame && data.freqData && data.timeData) {
        onAudioFrame(new Uint8Array(data.freqData), new Uint8Array(data.timeData));
      }
    };

    this.channel.addEventListener('message', handler);
    return () => {
      this.channel?.removeEventListener('message', handler);
    };
  }
}

export const detachedPreviewService = new DetachedPreviewService();
