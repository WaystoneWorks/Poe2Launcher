<script lang="ts">
  import { tick } from 'svelte';
  import Icon, { type IconName } from './Icon.svelte';

  export interface Tab { id: string; label: string; icon?: IconName; badge?: string }
  /**
   * `panel` is the id of the element that shows the selected tab's content: it gets
   * `role="tabpanel"` and `aria-labelledby={tabId(panel, value)}` from the page.
   */
  let { tabs, value, onchange, label = 'Sections', panel, sub = false }: { tabs: Tab[]; value: string; onchange: (id: string) => void; label?: string; panel?: string; sub?: boolean } = $props();
  let list: HTMLElement | undefined = $state();

  async function key(e: KeyboardEvent, i: number) {
    const n = tabs.length;
    const to = e.key === 'ArrowRight' ? (i + 1) % n : e.key === 'ArrowLeft' ? (i - 1 + n) % n : e.key === 'Home' ? 0 : e.key === 'End' ? n - 1 : -1;
    if (to < 0) return;
    e.preventDefault();
    const next = tabs[to];
    if (!next) return;
    onchange(next.id);
    await tick();
    list?.querySelectorAll<HTMLElement>('[role="tab"]')[to]?.focus();
  }
</script>

<script lang="ts" module>
  /** The id of a tab button, for the panel's `aria-labelledby`. */
  export function tabId(panel: string, id: string): string {
    return `${panel}-tab-${id}`;
  }
</script>

<div class="seg tabs" class:sub role="tablist" aria-label={label} bind:this={list}>
  {#each tabs as t, i (t.id)}
    <button
      type="button"
      role="tab"
      id={panel ? tabId(panel, t.id) : undefined}
      aria-controls={panel && value === t.id ? panel : undefined}
      class:on={value === t.id}
      aria-selected={value === t.id}
      tabindex={value === t.id ? 0 : -1}
      onclick={() => onchange(t.id)}
      onkeydown={(e) => key(e, i)}
    >
      {#if t.icon}<Icon name={t.icon} size={15} />{/if}
      {t.label}
      {#if t.badge}<span class="tab-badge">{t.badge}</span>{/if}
    </button>
  {/each}
</div>

<style>
  .tabs button { display: inline-flex; align-items: center; gap: 7px; }
  .tab-badge { font-size: 11px; padding: 0 6px; border-radius: 99px; background: var(--surface-3); color: var(--muted); font-variant-numeric: tabular-nums; }
  .on .tab-badge { background: color-mix(in srgb, var(--accent) 20%, transparent); color: var(--accent); }
</style>
