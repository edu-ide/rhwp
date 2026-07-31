import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import test from 'node:test';

const realtimeSource = readFileSync(resolve(import.meta.dirname, '../src/engine/realtime-operation.ts'), 'utf-8');
const inputHandlerSource = readFileSync(resolve(import.meta.dirname, '../src/engine/input-handler.ts'), 'utf-8');
const keyboardSource = readFileSync(resolve(import.meta.dirname, '../src/engine/input-handler-keyboard.ts'), 'utf-8');
const insertSource = readFileSync(resolve(import.meta.dirname, '../src/command/commands/insert.ts'), 'utf-8');

function assertSource(source: string, pattern: RegExp, message: string): void {
  assert.ok(pattern.test(source), message);
}

test('realtime operation model supports object deletion', () => {
  assertSource(realtimeSource, /\|\s*'deleteObject'/, 'deleteObject kind should exist');
  assertSource(realtimeSource, /RhwpRealtimeObjectType/, 'shared object type should exist');
  assertSource(realtimeSource, /objectType\?:\s*RhwpRealtimeObjectType/, 'operation should carry object type');
});

test('remote deleteObject operations are applied by InputHandler', () => {
  assertSource(inputHandlerSource, /transformedOp\.kind\s*===\s*'deleteObject'/, 'InputHandler should branch on deleteObject');
  assertSource(inputHandlerSource, /applyRemoteDeleteObject/, 'InputHandler should have a remote deleteObject applier');
  assertSource(inputHandlerSource, /deleteCellPictureControlByPath/, 'remote delete should support cell pictures');
  assertSource(inputHandlerSource, /deleteEquationControl/, 'remote delete should support equations');
  assertSource(inputHandlerSource, /deleteShapeControl/, 'remote delete should support shapes');
});

test('local object deletion paths emit deleteObject realtime drafts', () => {
  assertSource(inputHandlerSource, /buildDeleteObjectRealtimeOperation/, 'InputHandler should build deleteObject drafts');
  assertSource(inputHandlerSource, /kind:\s*'deleteObject'/, 'InputHandler draft should use deleteObject');
  assertSource(inputHandlerSource, /realtimeOperation:\s*this\.buildDeleteObjectRealtimeOperation\(ref\)/, 'performDelete should attach deleteObject realtime metadata');
  assertSource(keyboardSource, /buildDeleteObjectRealtimeOperation/, 'keyboard delete/cut should build deleteObject drafts');
  assertSource(keyboardSource, /realtimeOperation:\s*this\.buildDeleteObjectRealtimeOperation\(ref\)/, 'keyboard delete/cut should attach deleteObject realtime metadata');
  assertSource(insertSource, /emitObjectDeleteRealtimeOperation/, 'insert delete command should emit deleteObject');
  assertSource(insertSource, /kind:\s*'deleteObject'/, 'insert delete command draft should use deleteObject');
});
