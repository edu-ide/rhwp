import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import test from 'node:test';

const tableSource = readFileSync(resolve(import.meta.dirname, '../src/command/commands/table.ts'), 'utf-8');
const inputHandlerSource = readFileSync(resolve(import.meta.dirname, '../src/engine/input-handler.ts'), 'utf-8');
const tableCellPropsDialogSource = readFileSync(resolve(import.meta.dirname, '../src/ui/table-cell-props-dialog.ts'), 'utf-8');
const cellBorderBgDialogSource = readFileSync(resolve(import.meta.dirname, '../src/ui/cell-border-bg-dialog.ts'), 'utf-8');
const formulaDialogSource = readFileSync(resolve(import.meta.dirname, '../src/ui/formula-dialog.ts'), 'utf-8');

test('table row and column snapshot commands include realtime operation metadata', () => {
  assert.match(tableSource, /kind:\s*'insertTableRow'/);
  assert.match(tableSource, /kind:\s*'insertTableColumn'/);
  assert.match(tableSource, /kind:\s*'deleteTableRow'/);
  assert.match(tableSource, /kind:\s*'deleteTableColumn'/);
  assert.match(tableSource, /realtimeOperation/);
});

test('table cell merge and split snapshot commands include realtime operation metadata', () => {
  assert.match(tableSource, /kind:\s*'mergeTableCells'/);
  assert.match(tableSource, /kind:\s*'splitTableCell'/);
  assert.match(tableSource, /kind:\s*'splitTableCellsInRange'/);
  assert.match(tableSource, /splitRows/);
  assert.match(tableSource, /splitCols/);
});

test('table create and delete snapshot commands include realtime operation metadata', () => {
  assert.match(tableSource, /kind:\s*'createTable'/);
  assert.match(tableSource, /kind:\s*'deleteTable'/);
  assert.match(tableSource, /rowCount:\s*rows/);
  assert.match(tableSource, /colCount:\s*cols/);
  assert.match(tableSource, /tableOptions/);
});

test('table property snapshot commands include realtime operation metadata', () => {
  assert.match(tableSource, /kind:\s*'setTableProperties'/);
  assert.match(tableSource, /kind:\s*'resizeTableCells'/);
  assert.match(tableSource, /tableProps/);
  assert.match(tableSource, /cellUpdates/);
});

test('remote table property operations are applied by InputHandler', () => {
  assert.match(inputHandlerSource, /setTableProperties/);
  assert.match(inputHandlerSource, /setCellProperties/);
  assert.match(inputHandlerSource, /resizeTableCells/);
  assert.match(inputHandlerSource, /applyRemoteSetTableProperties/);
  assert.match(inputHandlerSource, /applyRemoteSetCellProperties/);
  assert.match(inputHandlerSource, /applyRemoteResizeTableCells/);
});

test('table property dialogs emit realtime operation drafts for direct wasm writes', () => {
  assert.match(tableSource, /emitRealtimeOperationDraftPublic/);
  assert.match(tableSource, /onRealtimeOperation/);
  assert.match(tableCellPropsDialogSource, /onRealtimeOperation/);
  assert.match(tableCellPropsDialogSource, /kind:\s*'setCellProperties'/);
  assert.match(tableCellPropsDialogSource, /kind:\s*'setTableProperties'/);
  assert.match(cellBorderBgDialogSource, /onRealtimeOperation/);
  assert.match(cellBorderBgDialogSource, /kind:\s*'setCellProperties'/);
});

test('table cell text transform commands emit realtime replacement operations', () => {
  assert.match(tableSource, /emitCellTextReplacementRealtimeOperations/);
  assert.match(tableSource, /kind:\s*'deleteText'/);
  assert.match(tableSource, /kind:\s*'insertText'/);
  assert.match(tableSource, /deletedText:\s*oldText/);
  assert.ok(
    [...tableSource.matchAll(/emitCellTextReplacementRealtimeOperations\(/g)].length >= 4,
    'helper definition plus thousand separator and decimal commands should emit replacement operations',
  );
});

test('formula writes emit realtime replacement operations', () => {
  assert.match(tableSource, /emitFormulaResultRealtimeOperations/);
  assert.match(tableSource, /dialog\.onRealtimeOperation\s*=\s*\(draft\)\s*=>\s*ih\.emitRealtimeOperationDraftPublic\(draft\)/);
  assert.match(formulaDialogSource, /onRealtimeOperation/);
  assert.match(formulaDialogSource, /kind:\s*'deleteText'/);
  assert.match(formulaDialogSource, /kind:\s*'insertText'/);
  assert.match(formulaDialogSource, /deletedText:\s*oldText/);
});

test('snapshot operations emit realtime metadata through InputHandler', () => {
  assert.match(inputHandlerSource, /desc\.meta\?\.realtimeOperation/);
  assert.match(inputHandlerSource, /emitRealtimeOperationDraft/);
});
