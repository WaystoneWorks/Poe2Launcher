<script lang="ts">
  import { app } from '../lib/app.svelte';
  import { router } from '../lib/router.svelte';
  import { ipc } from '../lib/ipc';
  import { run } from '../lib/notify.svelte';
  import { defaultScope, scopeOptions } from '../lib/mods';
  import Tabs, { tabId } from '../lib/ui/Tabs.svelte';
  import Dropdown from '../lib/ui/Dropdown.svelte';
  import ModsInstalled from '../lib/ui/ModsInstalled.svelte';
  import ModsBrowse from '../lib/ui/ModsBrowse.svelte';
  import type { ModsView } from '../lib/types';

  // Mods are per server: the page works on one server's set at a time (or on "All servers").
  let scope = $state(defaultScope(app.settings?.selected_profile, app.profiles));
  // The sub-tab is in the URL (`#/library?tab=mods&view=browse`) like the other tabs.
  const view = $derived<'installed' | 'browse'>(router.route.query.get('view') === 'browse' ? 'browse' : 'installed');
  function setView(v: 'installed' | 'browse') {
    router.go('library', { tab: 'mods', view: v === 'browse' ? 'browse' : undefined });
  }
  let data = $state<ModsView | null>(null);

  const options = $derived(scopeOptions(app.profiles));

  async function load() {
    data = (await run(() => ipc.mods(scope))) ?? data;
  }
  $effect(() => {
    void scope;
    void app.status?.install_dir;
    void load();
  });
</script>

<div class="sechead">
  <p class="muted">
    Mods are kept in the launcher's cache (several versions side by side) and switched on <b>per server</b>: a server's required mods are always on for it,
    your own mods only where you enable them.
  </p>
  <div class="scope">
    <span class="muted" aria-hidden="true">Mods for</span>
    <Dropdown label="Mods for" value={scope} align="end" onselect={(v) => (scope = v)} items={options.map((o) => ({ value: o.value, label: o.label }))} />
  </div>
</div>

<Tabs
  tabs={[
    { id: 'installed', label: 'Installed', icon: 'mods', badge: data ? String(new Set(data.entries.map((e) => e.id)).size) : undefined },
    { id: 'browse', label: 'Browse', icon: 'search' },
  ]}
  value={view}
  panel="mods-panel"
  label="Mods"
  onchange={(id) => setView(id as 'installed' | 'browse')}
/>

<div id="mods-panel" role="tabpanel" aria-labelledby={tabId('mods-panel', view)}>
  {#if view === 'browse'}
    <ModsBrowse {data} {scope} reload={load} openInstalled={() => setView('installed')} />
  {:else}
    <ModsInstalled {data} {scope} reload={load} openBrowse={() => setView('browse')} />
  {/if}
</div>

<style>
  .scope { display: inline-flex; align-items: center; gap: 8px; flex: none; white-space: nowrap; }
  .scope :global(.dd-trigger) { min-width: 200px; }
</style>
