<script lang="ts">
  import { onMount, type Snippet } from 'svelte';
  import { fade, fly } from 'svelte/transition';
  import { DUR, ease, ms } from '../motion';
  import Icon from './Icon.svelte';

  // A panel that slides in from the right over the page (Create a server, your server's settings).
  let { title, onclose, width = 720, children, head }: { title: string; onclose: () => void; width?: number; children: Snippet; head?: Snippet } = $props();
  let panel: HTMLElement | undefined = $state();

  // Focus moves into the panel on open and back to what opened it on close.
  onMount(() => {
    const opener = document.activeElement as HTMLElement | null;
    panel?.focus();
    return () => {
      if (opener?.isConnected) opener.focus();
    };
  });

  /** A modal dialog opened from inside the drawer (a confirm) owns Escape and the focus. */
  function dialogOpen(): boolean {
    return document.querySelector('dialog[open]') !== null;
  }

  function key(e: KeyboardEvent) {
    if (e.defaultPrevented || dialogOpen()) return;
    if (e.key === 'Tab' && panel) {
      // Keep Tab inside the panel (aria-modal).
      const items = [...panel.querySelectorAll<HTMLElement>('a[href], button:not(:disabled), input:not(:disabled), select:not(:disabled), textarea:not(:disabled), summary, [tabindex]:not([tabindex="-1"])')].filter((el) => el.offsetParent !== null);
      if (items.length === 0) return;
      const first = items[0]!;
      const last = items[items.length - 1]!;
      const active = document.activeElement;
      if (e.shiftKey && (active === first || active === panel)) {
        e.preventDefault();
        last.focus();
      } else if (!e.shiftKey && active === last) {
        e.preventDefault();
        first.focus();
      } else if (!panel.contains(active)) {
        e.preventDefault();
        first.focus();
      }
      return;
    }
    if (e.key === 'Escape') {
      e.preventDefault();
      onclose();
    }
  }
</script>

<svelte:window onkeydown={key} />

<div class="drawer-scrim" role="presentation" onclick={onclose} transition:fade={{ duration: ms(DUR.base), easing: ease }}></div>
<div
  class="drawer"
  role="dialog"
  aria-modal="true"
  aria-label={title}
  tabindex="-1"
  bind:this={panel}
  style:width="min({width}px, calc(100% - 120px))"
  transition:fly={{ x: 40, duration: ms(DUR.slow), easing: ease, opacity: 0 }}
>
  <div class="drawer-head">
    <h2>{title}</h2>
    {#if head}{@render head()}{/if}
    <span class="grow"></span>
    <button type="button" class="btn icon sm ghost" aria-label="Close" onclick={onclose}><Icon name="close" size={16} /></button>
  </div>
  <div class="drawer-body">
    {@render children()}
  </div>
</div>

<style>
  .drawer-scrim { position: fixed; inset: var(--titlebar-h, 0px) 0 0 0; z-index: 70; background: rgb(5 2 2 / 0.6); }
  .drawer {
    position: fixed; top: var(--titlebar-h, 0px); right: 0; bottom: 0; z-index: 71; display: flex; flex-direction: column;
    background: var(--surface); border-left: 1px solid var(--gold); box-shadow: var(--shadow); outline: none;
  }
  .drawer-head { display: flex; align-items: center; gap: var(--sp-3); padding: var(--sp-4) var(--sp-5); border-bottom: 1px solid var(--line-soft); }
  .drawer-head h2 { margin: 0; font: 600 var(--fs-lg) / 1.2 var(--font-head); letter-spacing: var(--tracking-head); text-transform: none; color: var(--gold-2); }
  .drawer-body { flex: 1; min-height: 0; overflow-y: auto; padding: var(--sp-4) var(--sp-5) var(--sp-5); }
</style>
