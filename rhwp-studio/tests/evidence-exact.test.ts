import test from 'node:test';
import assert from 'node:assert/strict';
import { exactEvidenceRange } from '../src/ui/evidence-exact.ts';
import { parseEvidenceClaims, resolveEvidenceClaim } from '../src/ui/evidence-claims.ts';

test('complete quote spans rendered lines without prefix matching', () => {
  assert.deepEqual(exactEvidenceRange('앞 문장\n정확한  근거\n문장입니다.\n뒤 문장', '정확한 근거 문장입니다.'), {status: 'resolved', start: 5, end: 19});
  assert.deepEqual(exactEvidenceRange('같은 시작이지만 서로 다른 문장입니다.', '같은 시작이지만 완전히 다른 문장입니다.'), {status: 'missing'});
});
test('duplicate complete quotes remain ambiguous', () => {
  assert.deepEqual(exactEvidenceRange('문장 하나\n문장 하나', '문장 하나'), {status: 'ambiguous'});
  assert.deepEqual(exactEvidenceRange('어떤 문장', '  '), {status: 'missing'});
  assert.deepEqual(exactEvidenceRange('a b', 'ab'), {status: 'missing'});
});

test('claim annotations require the entire quotation and re-resolve changed render trees', () => {
  const lines = ['도입', '정확한 근거', '문장입니다.', '마무리'].map((text, index) => ({
    page: 0, text, x: 10, y: index * 20, w: 100, h: 18,
  }));
  const claim = { id: 'claim-1', quote: '정확한 근거 문장입니다.', review_status: 'unreviewed' };
  assert.deepEqual(resolveEvidenceClaim(lines, claim), lines.slice(1, 3));
  assert.deepEqual(resolveEvidenceClaim(lines.map(line => ({ ...line, text: line.text.replace('문장입니다.', '다른 문장입니다.') })), claim), []);
  assert.deepEqual(resolveEvidenceClaim([...lines, ...lines], claim), []);
  assert.deepEqual(resolveEvidenceClaim([...lines, ...lines.map(line => ({ ...line, page: 1 }))], { ...claim, page: 2 }), lines.slice(1, 3).map(line => ({ ...line, page: 1 })));
  assert.deepEqual(resolveEvidenceClaim(lines, { ...claim, page: 2 }), []);
});

test('claim RPC rejects duplicate identifiers and invalid page hints without silently changing numbering', () => {
  const claims = [{ id: 'missing', quote: '', review_status: 'unreviewed' }, { id: 'valid', quote: '근거', page: 2, review_status: 'reviewed' }];
  assert.deepEqual(parseEvidenceClaims(claims).map(claim => claim.id), ['missing', 'valid']);
  for (const value of [null, [claims[0], claims[0]], [{ ...claims[1], page: 0 }], [{ ...claims[1], page: 1.5 }], [{ ...claims[1], quote: 10 }]]) {
    assert.throws(() => parseEvidenceClaims(value));
  }
});
