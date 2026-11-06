<script lang="ts">
  import { app } from '../lib/app.svelte';
  import { router } from '../lib/router.svelte';
  import { versionLabel } from '../lib/gameversion';
  import PageHead from '../lib/ui/PageHead.svelte';
  import Tabs, { tabId } from '../lib/ui/Tabs.svelte';
  import Install from './Install.svelte';
  import Mods from './Mods.svelte';
  import Logs from './Logs.svelte';

  // The official mode has no mods; the others get all three.
  const tabs = $derived([
    { id: 'game', label: 'Game', icon: 'install' as const },
    ...(app.official ? [] : [{ id: 'mods', label: 'Mods', icon: 'mods' as const }]),
    { id: 'logs', label: 'Logs', icon: 'file' as const },
  ]);
  // Old links `#/mods` and `#/logs` open their tab here.
  const legacy = $derived(router.route.segments[0] === 'mods' || router.route.segments[0] === 'logs' ? router.route.segments[0] : null);
  const want = $derived(router.route.query.get('tab') ?? legacy ?? 'game');
  const tab = $derived(tabs.some((t) => t.id === want) ? want : 'game');
</script>

<PageHead icon="library" title="Library" sub={`Game files, mods and logs for ${versionLabel(app.version)}`} />

<Tabs {tabs} value={tab} label="Library" panel="library-panel" onchange={(id) => router.go('library', { tab: id === 'game' ? undefined : id })} />

<div id="library-panel" role="tabpanel" aria-labelledby={tabId('library-panel', tab)}>
  {#if tab === 'mods'}
    <Mods />
  {:else if tab === 'logs'}
    <Logs />
  {:else}
    <Install />
  {/if}
</div>
