<script lang="ts">
  import { fly, fade } from 'svelte/transition';
  import { flip } from 'svelte/animate';
  import { cubicOut } from 'svelte/easing';
  import { toastMs, toasts } from '../notify.svelte';
  import { ms } from '../motion';
  import Icon from './Icon.svelte';
</script>

<!-- Errors are alerts (announced at once); the rest go through a persistent polite live region. -->
<div class="sr-only" aria-live="polite" aria-atomic="true">{toasts.spoken}</div>
<div class="toasts">
  {#each toasts.items as t (t.id)}
    <div
      class="toast {t.kind}"
      role={t.kind === 'error' ? 'alert' : undefined}
      onmouseenter={() => toasts.pause(t.id)}
      onmouseleave={() => toasts.resume(t.id)}
      onfocusin={() => toasts.pause(t.id)}
      onfocusout={() => toasts.resume(t.id)}
      in:fly={{ x: 40, duration: ms(260), easing: cubicOut }}
      out:fade={{ duration: ms(160) }}
      animate:flip={{ duration: ms(200) }}
    >
      <span class="toast-icon"><Icon name={t.kind === 'ok' ? 'check' : 'alert'} size={16} /></span>
      <span class="toast-text">{t.text}</span>
      <button type="button" class="btn icon ghost sm" aria-label="Dismiss" onclick={() => toasts.dismiss(t.id)}><Icon name="close" size={14} /></button>
      <span class="toast-timer" style:animation-duration="{toastMs(t.kind)}ms"></span>
    </div>
  {/each}
</div>
