// @vitest-environment jsdom
import { describe, expect, it } from 'vitest';
import { radioGroup } from './radios';

function group(checked: number, disabled: number[] = []) {
  const root = document.createElement('div');
  const picked: number[] = [];
  for (let i = 0; i < 4; i++) {
    const b = document.createElement('button');
    b.setAttribute('role', 'radio');
    b.setAttribute('aria-checked', String(i === checked));
    b.disabled = disabled.includes(i);
    b.onclick = () => picked.push(i);
    root.append(b);
  }
  document.body.append(root);
  return { root, picked, radios: [...root.querySelectorAll('button')] };
}

describe('radioGroup', () => {
  it('has one Tab stop, on the checked option', async () => {
    const { root, radios } = group(2);
    radioGroup(root);
    await Promise.resolve();
    expect(radios.map((r) => r.tabIndex)).toEqual([-1, -1, 0, -1]);
  });

  it('moves and selects with the arrows, skipping disabled options and wrapping', () => {
    const { root, picked, radios } = group(0, [1]);
    radioGroup(root);
    radios[0]!.focus();
    root.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowRight' }));
    expect(picked).toEqual([2]);
    expect(document.activeElement).toBe(radios[2]);
    root.dispatchEvent(new KeyboardEvent('keydown', { key: 'End' }));
    root.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowRight' }));
    expect(picked).toEqual([2, 3, 0]);
  });
});
