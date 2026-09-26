import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import test from 'node:test';

const realtimeSource = readFileSync(resolve(import.meta.dirname, '../src/engine/realtime-operation.ts'), 'utf-8');
const inputHandlerSource = readFileSync(resolve(import.meta.dirname, '../src/engine/input-handler.ts'), 'utf-8');
const insertSource = readFileSync(resolve(import.meta.dirname, '../src/command/commands/insert.ts'), 'utf-8');
const mouseSource = readFileSync(resolve(import.meta.dirname, '../src/engine/input-handler-mouse.ts'), 'utf-8');

function assertSource(source: string, pattern: RegExp, message: string): void {
  assert.ok(pattern.test(source), message);
}

test('realtime operation model supports shape z-order changes', () => {
  assertSource(realtimeSource, /\|\s*'changeShapeZOrder'/, 'changeShapeZOrder kind should exist');
  assertSource(realtimeSource, /RhwpRealtimeZOrderAction/, 'z-order action type should exist');
  assertSource(realtimeSource, /zOrderAction\?:\s*RhwpRealtimeZOrderAction/, 'operation should carry zOrderAction');
});

test('remote z-order operations are applied by InputHandler', () => {
  assertSource(inputHandlerSource, /transformedOp\.kind\s*===\s*'changeShapeZOrder'/, 'InputHandler should branch on changeShapeZOrder');
  assertSource(inputHandlerSource, /applyRemoteChangeShapeZOrder/, 'InputHandler should have z-order applier');
  assertSource(inputHandlerSource, /changeShapeZOrder\(/, 'remote z-order should call wasm.changeShapeZOrder');
});

test('local z-order paths emit realtime drafts', () => {
  assertSource(insertSource, /realtimeOperation:\s*\{/, 'insert z-order commands publish through the operation router');
  assertSource(insertSource, /kind:\s*'changeShapeZOrder'/, 'z-order draft should use changeShapeZOrder');
  assertSource(insertSource, /zOrderAction:\s*operation/, 'z-order draft should carry action');
  assertSource(mouseSource, /realtimeOperation:\s*\{/, 'mouse bring-to-front draft must run through the operation router');
  assertSource(inputHandlerSource, /!desc\.command\.isNoOp\?\.\(\)/, 'a no-op command must not publish a collaboration operation');
  assertSource(mouseSource, /kind:\s*'changeShapeZOrder'/, 'mouse path should use changeShapeZOrder');
});
