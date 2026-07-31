import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import test from 'node:test';

const realtimeSource = readFileSync(resolve(import.meta.dirname, '../src/engine/realtime-operation.ts'), 'utf-8');
const inputHandlerSource = readFileSync(resolve(import.meta.dirname, '../src/engine/input-handler.ts'), 'utf-8');
const insertSource = readFileSync(resolve(import.meta.dirname, '../src/command/commands/insert.ts'), 'utf-8');

function assertSource(source: string, pattern: RegExp, message: string): void {
  assert.ok(pattern.test(source), message);
}

test('realtime operation model supports grouping and ungrouping shapes', () => {
  assertSource(realtimeSource, /\|\s*'groupShapes'/, 'groupShapes kind should exist');
  assertSource(realtimeSource, /\|\s*'ungroupShape'/, 'ungroupShape kind should exist');
  assertSource(realtimeSource, /RhwpRealtimeShapeTarget/, 'shape target type should exist');
  assertSource(realtimeSource, /shapeTargets\?:\s*RhwpRealtimeShapeTarget\[\]/, 'operation should carry shape targets');
});

test('remote group and ungroup operations are applied by InputHandler', () => {
  assertSource(inputHandlerSource, /transformedOp\.kind\s*===\s*'groupShapes'/, 'InputHandler should branch on groupShapes');
  assertSource(inputHandlerSource, /transformedOp\.kind\s*===\s*'ungroupShape'/, 'InputHandler should branch on ungroupShape');
  assertSource(inputHandlerSource, /applyRemoteGroupShapes/, 'InputHandler should have group applier');
  assertSource(inputHandlerSource, /applyRemoteUngroupShape/, 'InputHandler should have ungroup applier');
  assertSource(inputHandlerSource, /groupShapes\(/, 'remote group should call wasm.groupShapes');
  assertSource(inputHandlerSource, /ungroupShape\(/, 'remote ungroup should call wasm.ungroupShape');
});

test('local group and ungroup commands emit realtime drafts', () => {
  assertSource(insertSource, /emitGroupShapesRealtimeOperation/, 'group command should emit realtime draft');
  assertSource(insertSource, /emitUngroupShapeRealtimeOperation/, 'ungroup command should emit realtime draft');
  assertSource(insertSource, /kind:\s*'groupShapes'/, 'group draft should use groupShapes');
  assertSource(insertSource, /kind:\s*'ungroupShape'/, 'ungroup draft should use ungroupShape');
  assertSource(insertSource, /shapeTargets/, 'group draft should carry shape targets');
  assertSource(insertSource, /resultPpi/, 'group draft should carry result paragraph');
});
