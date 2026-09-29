export interface EvidenceClaim {
  id: string;
  quote: string;
  page?: number;
  review_status: string;
  /** missing: the claim has no source yet and must stay visibly apart from linked claims. */
  evidence_state: 'linked' | 'missing';
}

export interface EvidenceClaimLine {
  page: number;
  text: string;
  x: number;
  y: number;
  w: number;
  h: number;
}

export interface EvidenceClaimStyle {
  label: string;
  color: string;
  background: string;
  line: 'dotted' | 'dashed';
  badgeBackground: string;
}

const WHITESPACE = /\s/;

/**
 * Never turn a missing, duplicate, or invalid quotation into a guessed position.
 * Whitespace is ignored: a wrapped Korean line breaks inside a word, so the rendered
 * lines of one sentence carry no reliable spaces at their joins.
 */
export function resolveEvidenceClaim(lines: EvidenceClaimLine[], claim: EvidenceClaim): EvidenceClaimLine[] {
  const page = claim.page;
  const scoped = page === undefined ? lines : lines.filter(line => line.page === page - 1);
  const needle = claim.quote.replace(/\s+/g, '');
  if (!needle) return [];
  let joined = '';
  const owners: number[] = [];
  scoped.forEach((line, index) => {
    for (const character of line.text) {
      if (WHITESPACE.exec(character)) continue;
      joined += character;
      // A character outside the BMP is two code units; keep one owner per unit.
      for (let unit = 0; unit < character.length; unit++) owners.push(index);
    }
  });
  const start = joined.indexOf(needle);
  if (start < 0 || joined.indexOf(needle, start + 1) >= 0) return [];
  return scoped.slice(owners[start], owners[start + needle.length - 1] + 1);
}

/** One palette with the Office panel and the Word view: linked, reviewed, and still needing evidence. */
export function claimAnnotationStyle(claim: EvidenceClaim): EvidenceClaimStyle {
  if (claim.evidence_state === 'missing') {
    return { label: '근거 필요', color: '#be123c', background: 'rgba(244,63,94,.16)', line: 'dashed', badgeBackground: '#fff1f2' };
  }
  if (claim.review_status === 'reviewed') {
    return { label: '근거', color: '#0f766e', background: 'rgba(20,184,166,.13)', line: 'dotted', badgeBackground: '#fff' };
  }
  return { label: '근거', color: '#a16207', background: 'rgba(250,204,21,.20)', line: 'dotted', badgeBackground: '#fff' };
}

export function parseEvidenceClaims(value: unknown): EvidenceClaim[] {
  if (!Array.isArray(value) || value.length > 1000) throw new Error('links must be an array of at most 1000 claims');
  const ids = new Set<string>();
  return value.map(item => {
    if (!item || typeof item !== 'object' || typeof item.id !== 'string' || !item.id
      || typeof item.quote !== 'string' || item.quote.length > 20000
      || typeof item.review_status !== 'string'
      || (item.page !== undefined && (!Number.isSafeInteger(item.page) || item.page < 1))
      || (item.evidence_state !== undefined && item.evidence_state !== 'linked' && item.evidence_state !== 'missing')
      || ids.has(item.id)) throw new Error('Invalid evidence claim');
    ids.add(item.id);
    // An Office host from before 근거 필요 sends only linked claims.
    return {id: item.id, quote: item.quote, page: item.page, review_status: item.review_status,
      evidence_state: item.evidence_state ?? 'linked'};
  });
}
