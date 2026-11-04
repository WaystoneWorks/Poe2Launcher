<script lang="ts">
  import { leagueDates, leagueGroups, variantTag } from '../leagues';
  import Chip from './Chip.svelte';
  import Spinner from './Spinner.svelte';
  import type { LeagueInfo } from '../types';

  // The active leagues of a server with their banners. `load` reads them when the list is shown.
  let { load }: { load: () => Promise<LeagueInfo[]> } = $props();
  let list = $state<LeagueInfo[] | null>(null);
  let error = $state<string | null>(null);
  $effect(() => {
    list = null;
    error = null;
    load().then((l) => (list = l), (e) => (error = e instanceof Error ? e.message : String(e)));
  });
  const groups = $derived(leagueGroups(list ?? []));
</script>

{#if error}
  <p class="hint err">{error}</p>
{:else if list === null}
  <p class="hint"><Spinner size={12} /> Loading leagues...</p>
{:else if groups.length === 0}
  <p class="hint">No league yet: players can log in but cannot create characters.</p>
{:else}
  <ul class="leagues">
    {#each groups as g (g.base.id)}
      <li>
        {#if g.base.banner}<img src={g.base.banner} alt="" />{/if}
        <div class="info">
          <b>{g.base.display_name}</b>
          {#if g.base.is_default}<Chip tone="accent">default</Chip>{/if}
          {#each g.variants as v (v.id)}<Chip>{variantTag(v) ?? v.display_name}</Chip>{/each}
          {#if leagueDates(g.base)}<span class="muted">{leagueDates(g.base)}</span>{/if}
        </div>
      </li>
    {/each}
  </ul>
{/if}

<style>
  .leagues { list-style: none; margin: 0; padding: 0; display: grid; gap: 8px; }
  .leagues li { display: flex; align-items: center; gap: 12px; }
  .leagues img { width: 96px; max-height: 48px; object-fit: contain; border-radius: var(--radius-sm); background: var(--bg-2); flex: none; }
  .info { display: flex; flex-wrap: wrap; align-items: center; gap: 6px 8px; font-size: var(--fs-sm); }
</style>
