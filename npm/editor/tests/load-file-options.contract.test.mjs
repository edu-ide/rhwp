import test from 'node:test';
import assert from 'node:assert/strict';

import { RhwpEditor } from '../index.js';

test('loadFile sends the embed dialog default while preserving explicit choices', async () => {
  const requests = [];
  const transport = {
    request(method, params) {
      requests.push({ method, params });
      return Promise.resolve({ pageCount: 1 });
    },
  };
  const editor = new RhwpEditor({}, transport);
  const data = new Uint8Array([1, 2, 3]);

  await editor.loadFile(data, 'omitted.hwp');
  await editor.loadFile(data, 'false.hwp', { suppressDialogs: false });
  await editor.loadFile(data, 'true.hwp', { suppressDialogs: true });

  assert.deepEqual(
    requests.map(({ method, params }) => ({ method, suppressDialogs: params.suppressDialogs })),
    [
      { method: 'loadFile', suppressDialogs: true },
      { method: 'loadFile', suppressDialogs: false },
      { method: 'loadFile', suppressDialogs: true },
    ],
  );
});


test('loadFile forwards large-document timeout independently of RPC payload', async () => {
  const requests = [];
  const editor = new RhwpEditor({}, { request(method, params, options) {
    requests.push({ method, params, options });
    return Promise.resolve({ pageCount: 300 });
  } });
  const data = new Uint8Array([1, 2, 3]);
  await editor.loadFile(data, 'large.hwp', { timeoutMs: 180000, skipUnsavedGuard: true });
  assert.equal(requests[0].options.timeoutMs, 180000);
  assert.equal(requests[0].params.timeoutMs, undefined);
  assert.equal(requests[0].params.skipUnsavedGuard, true);
  assert.equal(requests[0].params.suppressDialogs, true);
  assert.equal(requests[0].params.data, data);
});
