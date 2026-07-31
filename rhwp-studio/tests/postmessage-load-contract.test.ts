import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import test from 'node:test';

const mainSource = readFileSync(resolve(import.meta.dirname, '../src/main.ts'), 'utf-8');
const editorSource = readFileSync(resolve(import.meta.dirname, '../../npm/editor/index.js'), 'utf-8');
const keyboardSource = readFileSync(resolve(import.meta.dirname, '../src/engine/input-handler-keyboard.ts'), 'utf-8');
const wasmBridgeSource = readFileSync(resolve(import.meta.dirname, '../src/core/wasm-bridge.ts'), 'utf-8');

test('postMessage document loads run without interactive prompts', () => {
  assert.match(mainSource, /interface LoadBytesOptions/);
  assert.match(mainSource, /interactivePrompts\?: boolean/);
  assert.match(mainSource, /initializeDocument\(docInfo,[\s\S]*interactivePrompts/);
  assert.match(mainSource, /loadBytes\(bytes,[\s\S]*interactivePrompts: false/);
});

test('URL parameter document loads run without interactive prompts', () => {
  const start = mainSource.indexOf('async function loadFromUrlParam');
  assert.notEqual(start, -1, 'loadFromUrlParam is present');
  const end = mainSource.indexOf('function showFileUrlAccessGuidance', start);
  assert.notEqual(end, -1, 'loadFromUrlParam block end is present');
  const block = mainSource.slice(start, end);
  const nonInteractiveLoads = block.match(/loadBytes\(data, fileName, null, performance\.now\(\), \{\s*interactivePrompts: false,\s*\}\)/g) ?? [];
  assert.equal(nonInteractiveLoads.length, 2, 'both URL fetch branches must suppress interactive prompts');
});

test('postMessage API exposes readonly text range inspection', () => {
  assert.match(mainSource, /case 'getTextRange'/);
  assert.match(mainSource, /wasm\.getTextRange/);
  assert.match(editorSource, /async getTextRange/);
  assert.match(editorSource, /this\._request\('getTextRange'/);
});

test('postMessage API exposes readonly char property inspection for formatting proof', () => {
  assert.match(mainSource, /case 'getCharProperties'/);
  assert.match(mainSource, /wasm\.getCharPropertiesAt/);
  assert.match(editorSource, /async getCharProperties/);
  assert.match(editorSource, /this\._request\('getCharProperties'/);
});

test('postMessage API exposes readonly paragraph info for structural proof', () => {
  assert.match(mainSource, /case 'getParagraphInfo'/);
  assert.match(mainSource, /wasm\.getParagraphCount/);
  assert.match(mainSource, /wasm\.getParagraphLength/);
  assert.match(editorSource, /async getParagraphInfo/);
  assert.match(editorSource, /this\._request\('getParagraphInfo'/);
});

test('postMessage API exposes undo and redo for embedded hosts', () => {
  assert.match(mainSource, /case 'undo'/);
  assert.match(mainSource, /inputHandler\?\.performUndo\(\)/);
  assert.match(mainSource, /case 'redo'/);
  assert.match(mainSource, /inputHandler\?\.performRedo\(\)/);
  assert.match(mainSource, /case 'canUndo'/);
  assert.match(mainSource, /inputHandler\?\.canUndo\(\)/);
  assert.match(mainSource, /case 'canRedo'/);
  assert.match(mainSource, /inputHandler\?\.canRedo\(\)/);
  assert.match(editorSource, /async undo\(\)/);
  assert.match(editorSource, /this\._request\('undo'/);
  assert.match(editorSource, /async redo\(\)/);
  assert.match(editorSource, /this\._request\('redo'/);
  assert.match(editorSource, /async canUndo\(\)/);
  assert.match(editorSource, /this\._request\('canUndo'/);
  assert.match(editorSource, /async canRedo\(\)/);
  assert.match(editorSource, /this\._request\('canRedo'/);
});

test('postMessage API exposes command dispatch for embedded host shortcuts', () => {
  assert.match(mainSource, /case 'dispatchCommand'/);
  assert.match(mainSource, /dispatcher\.dispatch\(commandId, commandParams\)/);
  assert.match(editorSource, /const DEFAULT_SHORTCUTS = \[/);
  assert.match(editorSource, /function matchShortcut/);
  assert.match(editorSource, /async dispatchCommand\(commandId, params = \{\}\)/);
  assert.match(editorSource, /this\._request\('dispatchCommand', \{ commandId, params \}/);
  assert.match(editorSource, /const cmdId = matchShortcut\(event\)/);
  assert.match(editorSource, /this\.dispatchCommand\(cmdId\)/);
});

test('keyboard undo and redo bypass command dispatcher state gates', () => {
  const start = keyboardSource.indexOf('export function handleCtrlKey');
  assert.notEqual(start, -1, 'handleCtrlKey is present');
  const end = keyboardSource.indexOf('// ─── 코드 단축키 1번째 키', start);
  assert.notEqual(end, -1, 'handleCtrlKey direct shortcut block end is present');
  const block = keyboardSource.slice(start, end);
  assert.match(block, /cmdId === 'edit:undo'/);
  assert.match(block, /this\.performUndo\?\.\(\)/);
  assert.match(block, /cmdId === 'edit:redo'/);
  assert.match(block, /this\.performRedo\?\.\(\)/);
});

test('npm editor forwards host-level shortcuts to the iframe API', () => {
  assert.match(editorSource, /_installHostShortcutForwarding\(\)/);
  assert.match(editorSource, /\[\{ key: 'z', ctrl: true \}, 'edit:undo'\]/);
  assert.match(editorSource, /\[\{ key: 'y', ctrl: true \}, 'edit:redo'\]/);
  assert.match(editorSource, /\[\{ key: 'f', ctrl: true \}, 'edit:find'\]/);
  assert.match(editorSource, /const cmdId = matchShortcut\(event\)/);
  assert.match(editorSource, /this\.undo\(\)/);
  assert.match(editorSource, /this\.redo\(\)/);
  assert.match(editorSource, /this\.dispatchCommand\(cmdId\)/);
  assert.match(editorSource, /removeEventListener\('keydown', this\._keydownHandler/);
});

test('WasmBridge tolerates older WASM view option APIs during command context checks', () => {
  assert.match(wasmBridgeSource, /getOptionalDocMethod/);
  assert.match(wasmBridgeSource, /getOptionalDocMethod\('getShowParagraphMarks'\)/);
  assert.match(wasmBridgeSource, /getOptionalDocMethod\('getShowControlCodes'\)/);
  assert.match(wasmBridgeSource, /getOptionalDocMethod\('setShowParagraphMarks'\)/);
  assert.match(wasmBridgeSource, /getOptionalDocMethod\('setShowControlCodes'\)/);
  assert.match(wasmBridgeSource, /return get \? Boolean\(get\(\)\) : false/);
});

test('postMessage API exposes realtime operation bridge for embedding hosts', () => {
  assert.match(mainSource, /eventBus\.on\('realtime-operation'/);
  assert.match(mainSource, /case 'applyOperation'/);
  assert.match(mainSource, /inputHandler\.applyRealtimeOperation/);
  assert.match(editorSource, /rhwp-event/);
  assert.match(editorSource, /async applyOperation\(operation\)/);
  assert.match(editorSource, /this\._request\('applyOperation'/);
  assert.match(editorSource, /on\(eventName, handler\)/);
  assert.match(editorSource, /removeEventListener\('message', this\._messageHandler\)/);
});

test('npm editor ready handshake uses short probes instead of 10s RPC stalls', () => {
  assert.match(editorSource, /const READY_PROBE_TIMEOUT_MS = 750/);
  assert.match(editorSource, /const DEFAULT_REQUEST_TIMEOUT_MS = 10000/);
  assert.match(editorSource, /_request\(method, params = \{\}, options = \{\}\)/);
  assert.match(editorSource, /options\.timeoutMs \?\? DEFAULT_REQUEST_TIMEOUT_MS/);
  assert.match(editorSource, /this\._request\('ready', \{\}, \{ timeoutMs: READY_PROBE_TIMEOUT_MS \}\)/);
});

test('npm editor loadFile can receive an extended timeout for large documents', () => {
  assert.match(editorSource, /async loadFile\(data, fileName = 'document\.hwp', options = \{\}\)/);
  assert.match(editorSource, /this\._request\('loadFile', \{ data: bytes, fileName \}, options\)/);
});

test('npm editor installs iframe load listener before navigation starts', () => {
  const listenerIndex = editorSource.indexOf("iframe.addEventListener('load'");
  const srcIndex = editorSource.indexOf('iframe.src = studioUrl');
  const appendIndex = editorSource.indexOf('el.appendChild(iframe)');

  assert.ok(listenerIndex >= 0, 'load listener is present');
  assert.ok(srcIndex >= 0, 'iframe src assignment is present');
  assert.ok(appendIndex >= 0, 'iframe append is present');
  assert.ok(listenerIndex < srcIndex, 'load listener must be installed before iframe.src');
  assert.ok(listenerIndex < appendIndex, 'load listener must be installed before appendChild');
});
