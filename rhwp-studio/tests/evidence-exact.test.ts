import test from 'node:test';
import assert from 'node:assert/strict';
import { exactEvidenceRange } from '../src/ui/evidence-exact.ts';

test('complete quote spans rendered lines without prefix matching', () => {
  assert.deepEqual(exactEvidenceRange('앞 문장\n정확한  근거\n문장입니다.\n뒤 문장', '정확한 근거 문장입니다.'), {status: 'resolved', start: 5, end: 19});
  assert.deepEqual(exactEvidenceRange('같은 시작이지만 서로 다른 문장입니다.', '같은 시작이지만 완전히 다른 문장입니다.'), {status: 'missing'});
});
test('duplicate complete quotes remain ambiguous', () => {
  assert.deepEqual(exactEvidenceRange('문장 하나\n문장 하나', '문장 하나'), {status: 'ambiguous'});
  assert.deepEqual(exactEvidenceRange('어떤 문장', '  '), {status: 'missing'});
  assert.deepEqual(exactEvidenceRange('a b', 'ab'), {status: 'missing'});
});
