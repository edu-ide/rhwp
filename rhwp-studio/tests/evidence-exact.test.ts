import test from 'node:test';
import assert from 'node:assert/strict';
import { exactEvidenceRange } from '../src/ui/evidence-exact.ts';
import { claimAnnotationStyle, parseEvidenceClaims, resolveEvidenceClaim } from '../src/ui/evidence-claims.ts';

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
  const claim = { id: 'claim-1', quote: '정확한 근거 문장입니다.', review_status: 'unreviewed', evidence_state: 'linked' as const };
  assert.deepEqual(resolveEvidenceClaim(lines, claim), lines.slice(1, 3));
  assert.deepEqual(resolveEvidenceClaim(lines.map(line => ({ ...line, text: line.text.replace('문장입니다.', '다른 문장입니다.') })), claim), []);
  assert.deepEqual(resolveEvidenceClaim([...lines, ...lines], claim), []);
  assert.deepEqual(resolveEvidenceClaim([...lines, ...lines.map(line => ({ ...line, page: 1 }))], { ...claim, page: 2 }), lines.slice(1, 3).map(line => ({ ...line, page: 1 })));
  assert.deepEqual(resolveEvidenceClaim(lines, { ...claim, page: 2 }), []);
});

test('claim RPC rejects duplicate identifiers and invalid page hints without silently changing numbering', () => {
  const claims = [{ id: 'missing', quote: '', review_status: 'unreviewed' }, { id: 'valid', quote: '근거', page: 2, review_status: 'reviewed', evidence_state: 'missing' }];
  assert.deepEqual(parseEvidenceClaims(claims).map(claim => [claim.id, claim.evidence_state]), [['missing', 'linked'], ['valid', 'missing']]);
  for (const value of [null, [claims[0], claims[0]], [{ ...claims[1], page: 0 }], [{ ...claims[1], page: 1.5 }], [{ ...claims[1], quote: 10 }],
    [{ ...claims[1], evidence_state: 'verified' }]]) {
    assert.throws(() => parseEvidenceClaims(value));
  }
});

test('a sentence wrapped inside a word still resolves, since rendered lines carry no spaces at the join', () => {
  const lines = ['앞 문단', '자체 시험에서 오류 3건을 확인했', '습니다. 다음 문장'].map((text, index) => ({
    page: 0, text, x: 10, y: index * 20, w: 100, h: 18,
  }));
  const claim = { id: 'wrapped', quote: '자체 시험에서 오류 3건을 확인했습니다.', review_status: 'unreviewed', evidence_state: 'missing' as const };
  assert.deepEqual(resolveEvidenceClaim(lines, claim), lines.slice(1, 3));
  assert.deepEqual(resolveEvidenceClaim(lines, { ...claim, quote: '오류 4건을 확인했습니다.' }), []);
});

test('claims without evidence are painted apart from linked and reviewed claims', () => {
  const claim = { id: 'c', quote: '문장', review_status: 'unreviewed', evidence_state: 'linked' as const };
  const linked = claimAnnotationStyle(claim), reviewed = claimAnnotationStyle({ ...claim, review_status: 'reviewed' });
  const missing = claimAnnotationStyle({ ...claim, evidence_state: 'missing' });
  assert.equal(missing.label, '근거 필요');
  assert.equal(missing.line, 'dashed');
  assert.equal(new Set([linked.color, reviewed.color, missing.color]).size, 3);
  assert.equal(linked.line, 'dotted');
});

test('a judgment backed by reasoning is painted in its own colour, dashed when a premise does not hold', () => {
  const claim = { id: 'j', quote: '그래서 필요합니다', review_status: 'unreviewed', evidence_state: 'reasoned' as const };
  const reasoned = claimAnnotationStyle(claim), weak = claimAnnotationStyle({ ...claim, premise_problem: true });
  const others = ['linked', 'missing'].map(state => claimAnnotationStyle({ ...claim, evidence_state: state as 'linked' | 'missing' }));
  assert.equal(reasoned.label, '논리 근거');
  assert.deepEqual([reasoned.line, reasoned.lineWidth, weak.line, weak.lineWidth], ['double', 3, 'dashed', 2]);
  assert.ok(others.every(style => style.color !== reasoned.color));
  assert.deepEqual(parseEvidenceClaims([{ ...claim, premise_problem: true }]).map(item => [item.evidence_state, item.premise_problem]), [['reasoned', true]]);
  assert.throws(() => parseEvidenceClaims([{ ...claim, premise_problem: 'yes' }]));
});

test('semantic view styles stay apart from evidence and preserve filtered annotation numbers', () => {
  const claim = {id: 's', quote: '의미 있는 문장', review_status: 'reviewed', evidence_state: 'annotation' as const};
  const parsed = parseEvidenceClaims([{...claim, annotation_number: 19, annotation_layer: 'intent', annotation_label: '의도 · 문제 제기'}])[0];
  assert.equal(parsed.annotation_number, 19);
  assert.equal(claimAnnotationStyle(parsed).label, '의도 · 문제 제기');
  const intent = claimAnnotationStyle(parsed), concept = claimAnnotationStyle({...claim, annotation_layer: 'concept'});
  const editorial = claimAnnotationStyle({...claim, annotation_layer: 'editorial', editorial_kind: 'unnecessary'});
  const good = claimAnnotationStyle({...claim, annotation_layer: 'editorial', editorial_kind: 'good'});
  assert.equal(new Set([intent.color, concept.color, editorial.color, good.color]).size, 4);
  assert.equal(editorial.line, 'dashed');
  assert.equal(claimAnnotationStyle({...parsed, evidence_state: 'linked'}).color, intent.color);
  assert.equal(claimAnnotationStyle(claim).label, '의미 표시');
  for (const invalid of [{annotation_number: 0}, {annotation_number: 1.2}, {annotation_layer: 'verified'},
    {annotation_label: 'x'.repeat(241)}, {editorial_kind: 'delete-automatically'}]) {
    assert.throws(() => parseEvidenceClaims([{...claim, ...invalid}]));
  }
});
