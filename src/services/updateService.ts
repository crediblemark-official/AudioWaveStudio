export interface UpdateCheckResult {
  hasUpdate: boolean;
  currentVersion: string;
  latestVersion: string;
  releaseNotes?: string;
  downloadUrl?: string;
  error?: string;
}

export const CURRENT_VERSION = '1.9.0';
export const UPDATE_ENDPOINT_GITHUB = 'https://api.github.com/repos/crediblemark-official/AudioWaveStudio/releases/latest';
export const UPDATE_ENDPOINT_CREDIBLEMARK = 'https://crediblemark.com/api/audiowave/update.json';

function compareVersions(v1: string, v2: string): number {
  const p1 = v1.replace(/^v/, '').split('.').map(Number);
  const p2 = v2.replace(/^v/, '').split('.').map(Number);
  const len = Math.max(p1.length, p2.length);

  for (let i = 0; i < len; i++) {
    const num1 = p1[i] || 0;
    const num2 = p2[i] || 0;
    if (num1 > num2) return 1;
    if (num1 < num2) return -1;
  }
  return 0;
}

export async function checkForUpdates(): Promise<UpdateCheckResult> {
  try {
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), 6000);

    // Primary attempt: GitHub Releases API
    let res = await fetch(UPDATE_ENDPOINT_GITHUB, {
      signal: controller.signal,
      headers: { Accept: 'application/vnd.github.v3+json' },
    }).catch(() => null);

    let latestVersion = CURRENT_VERSION;
    let downloadUrl = 'https://crediblemark.com';
    let releaseNotes = '';

    if (res && res.ok) {
      const data = await res.json();
      latestVersion = (data.tag_name || data.name || CURRENT_VERSION).replace(/^v/, '');
      downloadUrl = data.html_url || 'https://crediblemark.com';
      releaseNotes = data.body || '';
    } else {
      // Fallback attempt: CredibleMark API endpoint
      const fallbackRes = await fetch(UPDATE_ENDPOINT_CREDIBLEMARK, {
        signal: controller.signal,
      }).catch(() => null);

      if (fallbackRes && fallbackRes.ok) {
        const data = await fallbackRes.json();
        latestVersion = (data.version || CURRENT_VERSION).replace(/^v/, '');
        downloadUrl = data.downloadUrl || 'https://crediblemark.com';
        releaseNotes = data.notes || '';
      }
    }

    clearTimeout(timeoutId);

    const hasUpdate = compareVersions(latestVersion, CURRENT_VERSION) > 0;

    return {
      hasUpdate,
      currentVersion: CURRENT_VERSION,
      latestVersion,
      releaseNotes,
      downloadUrl,
    };
  } catch (err: unknown) {
    const message = err instanceof Error ? err.message : String(err);
    return {
      hasUpdate: false,
      currentVersion: CURRENT_VERSION,
      latestVersion: CURRENT_VERSION,
      error: message.includes('aborted')
        ? 'Waktu koneksi habis saat memeriksa pembaruan.'
        : 'Tidak dapat terhubung ke server pembaruan.',
    };
  }
}
