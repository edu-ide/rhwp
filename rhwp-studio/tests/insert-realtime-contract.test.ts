import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import test from 'node:test';

const insertSource = readFileSync(resolve(import.meta.dirname, '../src/command/commands/insert.ts'), 'utf-8');
const picturePropsDialogSource = readFileSync(resolve(import.meta.dirname, '../src/ui/picture-props-dialog.ts'), 'utf-8');

test('picture object commands emit realtime resizeObject drafts for direct property writes', () => {
  assert.match(insertSource, /emitObjectPropertiesRealtimeOperation/);
  assert.match(insertSource, /kind:\s*'resizeObject'/);
  assert.match(insertSource, /objectTargets/);
  assert.match(insertSource, /emitRealtimeOperationDraftPublic/);
  assert.match(insertSource, /applyRotationDelta/);
  assert.match(insertSource, /toggleFlip/);
  assert.match(insertSource, /insert:caption-toggle/);
});

test('picture properties dialog emits realtime resizeObject drafts', () => {
  assert.match(insertSource, /picturePropsDialog\.onRealtimeOperation\s*=\s*\(draft\)\s*=>\s*ih\.emitRealtimeOperationDraftPublic\(draft\)/);
  assert.match(picturePropsDialogSource, /onRealtimeOperation/);
  assert.match(picturePropsDialogSource, /kind:\s*'resizeObject'/);
  assert.match(picturePropsDialogSource, /objectTargets/);
  assert.match(picturePropsDialogSource, /before:/);
  assert.match(picturePropsDialogSource, /after:/);
});
