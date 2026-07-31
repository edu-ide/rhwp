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

test('realtime operation model supports structural insert commands', () => {
  assertSource(realtimeSource, /\|\s*'insertEquation'/, 'insertEquation kind should exist');
  assertSource(realtimeSource, /\|\s*'insertField'/, 'insertField kind should exist');
  assertSource(realtimeSource, /\|\s*'insertFootnote'/, 'insertFootnote kind should exist');
  assertSource(realtimeSource, /\|\s*'insertEndnote'/, 'insertEndnote kind should exist');
  assertSource(realtimeSource, /equationText\?:\s*string/, 'equation payload should carry equationText');
  assertSource(realtimeSource, /fieldGuide\?:\s*string/, 'field payload should carry guide text');
  assertSource(realtimeSource, /fieldEditable\?:\s*boolean/, 'field payload should carry editable flag');
});

test('remote structural insert operations are applied by InputHandler', () => {
  assertSource(inputHandlerSource, /transformedOp\.kind\s*===\s*'insertEquation'/, 'InputHandler should branch on insertEquation');
  assertSource(inputHandlerSource, /applyRemoteInsertEquation/, 'InputHandler should have equation applier');
  assertSource(inputHandlerSource, /applyRemoteInsertField/, 'InputHandler should have field applier');
  assertSource(inputHandlerSource, /applyRemoteInsertFootnote/, 'InputHandler should have footnote applier');
  assertSource(inputHandlerSource, /applyRemoteInsertEndnote/, 'InputHandler should have endnote applier');
  assertSource(inputHandlerSource, /insertEquation\(/, 'remote equation should call wasm.insertEquation');
  assertSource(inputHandlerSource, /insertClickHereField\(/, 'remote field should call wasm.insertClickHereField');
  assertSource(inputHandlerSource, /insertFootnote\(/, 'remote footnote should call wasm.insertFootnote');
  assertSource(inputHandlerSource, /insertEndnote\(/, 'remote endnote should call wasm.insertEndnote');
});

test('local structural insert commands emit realtime drafts', () => {
  assertSource(insertSource, /emitInsertEquationRealtimeOperation/, 'equation insert should emit realtime draft');
  assertSource(insertSource, /emitInsertFieldRealtimeOperation/, 'field insert should emit realtime draft');
  assertSource(insertSource, /emitInsertNoteRealtimeOperation/, 'note insert should emit realtime draft');
  assertSource(insertSource, /kind:\s*'insertEquation'/, 'equation draft should use insertEquation');
  assertSource(insertSource, /kind:\s*'insertField'/, 'field draft should use insertField');
  assertSource(insertSource, /kind:\s*noteKind/, 'note draft should use footnote/endnote kind');
  assertSource(insertSource, /emitRealtimeOperationDraftPublic/, 'local insert commands should publish through InputHandler');
});
