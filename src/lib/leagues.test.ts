import { describe, expect, it } from 'vitest';
import { leagueDates, leagueGroups, variantTag } from './leagues';
import type { LeagueInfo } from './types';

const lg = (id: string, family: string, hardcore = false, ssf = false): LeagueInfo => ({
  id, display_name: id, family, hardcore, ssf, permanent: false, is_default: false, starts_at: null, ends_at: null, banner: null,
});

describe('leagues', () => {
  it('tags variants', () => {
    expect(variantTag(lg('a', 'a'))).toBeNull();
    expect(variantTag(lg('a', 'a', true))).toBe('HC');
    expect(variantTag(lg('a', 'a', false, true))).toBe('SSF');
    expect(variantTag(lg('a', 'a', true, true))).toBe('HC SSF');
  });
  it('groups by family with the base leading', () => {
    const g = leagueGroups([lg('HC Dawn', 'Dawn', true), lg('Dawn', 'Dawn'), lg('Standard', 'Standard')]);
    expect(g.map((x) => x.base.id)).toEqual(['Dawn', 'Standard']);
    expect(g[0]!.variants.map((v) => v.id)).toEqual(['HC Dawn']);
  });
  it('falls back to a variant when the base is hidden', () => {
    expect(leagueGroups([lg('HC Dawn', 'Dawn', true)])[0]!.base.id).toBe('HC Dawn');
  });
  it('words the dates', () => {
    const now = new Date(2026, 10, 4).getTime() / 1000;
    const at = (y: number, m: number, d: number) => new Date(y, m - 1, d, 12).getTime() / 1000;
    expect(leagueDates({ starts_at: null, ends_at: null }, now)).toBeNull();
    expect(leagueDates({ starts_at: at(2026, 11, 10), ends_at: at(2027, 2, 10) }, now)).toBe('Starts 2026-11-10');
    expect(leagueDates({ starts_at: at(2026, 10, 1), ends_at: at(2027, 2, 10) }, now)).toBe('Ends 2027-02-10');
  });
});
