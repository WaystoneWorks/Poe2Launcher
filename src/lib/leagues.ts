import type { LeagueInfo } from './types';

export interface LeagueGroup {
  base: LeagueInfo;
  variants: LeagueInfo[];
}

/** Variant tag: `HC`, `SSF`, `HC SSF` or null for a base league. */
export function variantTag(l: Pick<LeagueInfo, 'hardcore' | 'ssf'>): string | null {
  if (l.hardcore && l.ssf) return 'HC SSF';
  if (l.hardcore) return 'HC';
  if (l.ssf) return 'SSF';
  return null;
}

/** Groups the flat list by family, the base (or the first row) leading, in the server's order. */
export function leagueGroups(list: LeagueInfo[]): LeagueGroup[] {
  const order: string[] = [];
  const by = new Map<string, LeagueInfo[]>();
  for (const l of list) {
    const key = l.family || l.id;
    if (!by.has(key)) {
      by.set(key, []);
      order.push(key);
    }
    by.get(key)!.push(l);
  }
  return order.map((k) => {
    const rows = by.get(k)!;
    const base = rows.find((r) => !r.hardcore && !r.ssf) ?? rows[0]!;
    return { base, variants: rows.filter((r) => r !== base) };
  });
}

/** `Ends 2027-02-10` or `Starts 2026-11-10`, local time; null for an open-ended league that has begun. */
export function leagueDates(l: Pick<LeagueInfo, 'starts_at' | 'ends_at'>, nowSecs = Date.now() / 1000): string | null {
  const day = (n: number) => {
    const d = new Date(n * 1000);
    return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`;
  };
  if (l.starts_at != null && l.starts_at > nowSecs) return `Starts ${day(l.starts_at)}`;
  if (l.ends_at != null) return `Ends ${day(l.ends_at)}`;
  return null;
}
