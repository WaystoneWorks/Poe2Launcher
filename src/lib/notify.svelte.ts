import { errorText } from './ipc';

export interface Toast {
  id: number;
  kind: 'ok' | 'warn' | 'error';
  text: string;
}

/** How long a toast stays, in ms. */
export function toastMs(kind: Toast['kind']): number {
  return kind === 'ok' ? 4000 : 9000;
}

class Toasts {
  items = $state<Toast[]>([]);
  /** The text of the polite live region (ok toasts); cleared first so a repeated message is read again. */
  spoken = $state('');
  private next = 1;
  /** Per toast: its timer and the time left when paused. */
  private timers = new Map<number, { timer: ReturnType<typeof setTimeout> | null; left: number; since: number }>();

  show(kind: Toast['kind'], text: string): void {
    const id = this.next++;
    this.items = [...this.items, { id, kind, text }];
    this.arm(id, toastMs(kind));
    if (kind !== 'error') {
      this.spoken = '';
      setTimeout(() => (this.spoken = text), 60);
    }
  }

  private arm(id: number, left: number): void {
    this.timers.set(id, { timer: setTimeout(() => this.dismiss(id), left), left, since: Date.now() });
  }

  /** Hover or focus keeps a toast open; the timer goes on from where it stopped. */
  pause(id: number): void {
    const t = this.timers.get(id);
    if (!t || t.timer === null) return;
    clearTimeout(t.timer);
    this.timers.set(id, { timer: null, left: Math.max(1000, t.left - (Date.now() - t.since)), since: 0 });
  }

  resume(id: number): void {
    const t = this.timers.get(id);
    if (t && t.timer === null) this.arm(id, t.left);
  }

  dismiss(id: number): void {
    const t = this.timers.get(id);
    if (t?.timer) clearTimeout(t.timer);
    this.timers.delete(id);
    this.items = this.items.filter((t) => t.id !== id);
  }
}
export const toasts = new Toasts();

/** Runs an action and reports the outcome in a toast. Returns the result, or undefined on failure. */
export async function run<T>(fn: () => Promise<T>, ok?: string | ((r: T) => string)): Promise<T | undefined> {
  try {
    const r = await fn();
    const text = typeof ok === 'function' ? ok(r) : ok;
    if (text) toasts.show('ok', text);
    return r;
  } catch (e) {
    toasts.show('error', errorText(e));
    return undefined;
  }
}
