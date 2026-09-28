/** Match the entire quotation, never a prefix or the first of several matches. */
export function exactEvidenceRange(text: string, quote: string):
  { status: 'resolved'; start: number; end: number } | { status: 'missing' | 'ambiguous' } {
  const offsets: number[] = [];
  let normalized = '';
  for (let i = 0; i < text.length; i++) {
    if (/\s/.test(text[i])) {
      if (normalized.endsWith(' ')) continue;
      normalized += ' ';
    } else normalized += text[i];
    offsets.push(i);
  }
  const needle = quote.replace(/\s+/g, ' ').trim();
  if (!needle) return { status: 'missing' };
  const start = normalized.indexOf(needle);
  if (start < 0) return { status: 'missing' };
  if (normalized.indexOf(needle, start + 1) >= 0) return { status: 'ambiguous' };
  return { status: 'resolved', start: offsets[start], end: offsets[start + needle.length - 1] + 1 };
}
