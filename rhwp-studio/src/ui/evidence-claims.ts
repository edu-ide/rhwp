import { exactEvidenceRange } from './evidence-exact.ts';

export interface EvidenceClaim {
  id: string;
  quote: string;
  page?: number;
  review_status: string;
}

export interface EvidenceClaimLine {
  page: number;
  text: string;
  x: number;
  y: number;
  w: number;
  h: number;
}

/** Never turn a missing, duplicate, or invalid quotation into a guessed position. */
export function resolveEvidenceClaim(lines: EvidenceClaimLine[], claim: EvidenceClaim): EvidenceClaimLine[] {
  const page = claim.page;
  const scoped = page === undefined ? lines : lines.filter(line => line.page === page - 1);
  const match = exactEvidenceRange(scoped.map(line => line.text).join('\n'), claim.quote);
  if (match.status !== 'resolved') return [];
  let offset = 0;
  return scoped.filter(line => {
    const end = offset + line.text.length;
    const overlaps = end > match.start && offset < match.end;
    offset = end + 1;
    return overlaps;
  });
}

export function parseEvidenceClaims(value: unknown): EvidenceClaim[] {
  if (!Array.isArray(value) || value.length > 1000) throw new Error('links must be an array of at most 1000 claims');
  const ids = new Set<string>();
  return value.map(item => {
    if (!item || typeof item !== 'object' || typeof item.id !== 'string' || !item.id
      || typeof item.quote !== 'string' || item.quote.length > 20000
      || typeof item.review_status !== 'string'
      || (item.page !== undefined && (!Number.isSafeInteger(item.page) || item.page < 1))
      || ids.has(item.id)) throw new Error('Invalid evidence claim');
    ids.add(item.id);
    return {id: item.id, quote: item.quote, page: item.page, review_status: item.review_status};
  });
}
