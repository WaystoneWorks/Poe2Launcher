/**
 * Keyboard for a `role="radiogroup"` of `role="radio"` buttons: one Tab stop (the checked
 * option), arrow keys and Home/End move to the next enabled option and select it.
 * Use: `<div class="pills" role="radiogroup" use:radioGroup={value}>`; the parameter is only
 * there so the Tab stop follows the selection.
 */
export function radioGroup(node: HTMLElement, _value?: unknown) {
  const radios = () => [...node.querySelectorAll<HTMLElement>('[role="radio"]')].filter((r) => !(r as HTMLButtonElement).disabled);

  function sync() {
    const all = radios();
    const checked = all.find((r) => r.getAttribute('aria-checked') === 'true') ?? all[0];
    for (const r of node.querySelectorAll<HTMLElement>('[role="radio"]')) r.tabIndex = r === checked ? 0 : -1;
  }

  function key(e: KeyboardEvent) {
    const all = radios();
    const i = all.indexOf(document.activeElement as HTMLElement);
    if (i < 0 || all.length === 0) return;
    const n = all.length;
    const to =
      e.key === 'ArrowRight' || e.key === 'ArrowDown' ? (i + 1) % n
      : e.key === 'ArrowLeft' || e.key === 'ArrowUp' ? (i - 1 + n) % n
      : e.key === 'Home' ? 0
      : e.key === 'End' ? n - 1
      : -1;
    if (to < 0) return;
    e.preventDefault();
    all[to]!.focus();
    all[to]!.click();
  }

  node.addEventListener('keydown', key);
  queueMicrotask(sync);
  return {
    update() {
      queueMicrotask(sync);
    },
    destroy() {
      node.removeEventListener('keydown', key);
    },
  };
}
