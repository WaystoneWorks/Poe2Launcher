import { Channel, invoke } from '@tauri-apps/api/core';
import { relaunch } from '@tauri-apps/plugin-process';
import { checkForUpdate, installUpdate, type DownloadEvent, type UpdateDeps, type UpdateHandle, type UpdateState } from './update';

interface UpdateMeta {
  version: string;
  current_version: string;
  date?: string | null;
  body?: string | null;
}

/** The launcher looks for `latest.json` at the release source (Settings > Advanced), so the check and the install are backend commands. */
async function check(): Promise<UpdateHandle | null> {
  const meta = await invoke<UpdateMeta | null>('update_check');
  if (!meta) return null;
  return {
    version: meta.version,
    currentVersion: meta.current_version,
    date: meta.date ?? undefined,
    body: meta.body ?? undefined,
    async downloadAndInstall(onEvent) {
      const channel = new Channel<DownloadEvent>();
      if (onEvent) channel.onmessage = onEvent;
      await invoke('update_install', { onEvent: channel });
    },
  };
}

const deps: UpdateDeps = { check, relaunch };

class Updater {
  state = $state<UpdateState>({ kind: 'idle' });
  private handle: UpdateHandle | null = null;

  get busy(): boolean {
    return ['checking', 'installing', 'restarting'].includes(this.state.kind);
  }

  /** Looks for a newer launcher release. Never throws; a failed check is the quiet `unavailable` state. */
  async check(): Promise<void> {
    if (this.busy) return;
    this.state = { kind: 'checking' };
    const r = await checkForUpdate(deps);
    this.handle = r.handle;
    this.state = r.state;
  }

  /** Downloads, verifies (signature), installs and restarts. */
  async install(): Promise<void> {
    if (!this.handle || this.busy) return;
    await installUpdate(this.handle, deps, (s) => (this.state = s));
  }
}

export const updater = new Updater();
