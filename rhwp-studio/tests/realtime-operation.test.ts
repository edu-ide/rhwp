import test from 'node:test';
import assert from 'node:assert/strict';

import {
  createRealtimeOperationFromCommand,
  createRealtimeOperationFromDraft,
  transformRemoteOperationAgainstLocalHistory,
  transformCursorAfterRemoteOperation,
  type RhwpRealtimeOperation,
} from '../src/engine/realtime-operation.ts';
import type { DocumentPosition } from '../src/core/types.ts';

const pos = (charOffset: number): DocumentPosition => ({
  sectionIndex: 0,
  paragraphIndex: 2,
  charOffset,
});

const cellPos = (cellParaIndex: number, charOffset: number): DocumentPosition => ({
  sectionIndex: 0,
  paragraphIndex: cellParaIndex,
  parentParaIndex: 2,
  controlIndex: 0,
  cellIndex: 1,
  cellParaIndex,
  charOffset,
});

test('insert text commands serialize to rhwp realtime operations', () => {
  const op = createRealtimeOperationFromCommand(
    {
      type: 'insertText',
      position: pos(4),
      text: '안녕하세요',
      timestamp: 1234,
      execute() { return pos(9); },
      undo() { return pos(4); },
      mergeWith() { return null; },
    },
    { originId: 'client-a', sequence: 7 },
  );

  assert.equal(op?.contract, 'rhwp-realtime-op/v1');
  assert.equal(op?.kind, 'insertText');
  assert.equal(op?.originId, 'client-a');
  assert.equal(op?.sequence, 7);
  assert.deepEqual(op?.position, pos(4));
  assert.equal(op?.text, '안녕하세요');
  assert.equal(op?.timestamp, 1234);
});

test('delete text commands serialize to rhwp realtime operations', () => {
  const op = createRealtimeOperationFromCommand(
    {
      type: 'deleteText',
      position: pos(5),
      count: 2,
      direction: 'backward',
      deletedText: '요세',
      timestamp: 5678,
      execute() { return pos(5); },
      undo() { return pos(7); },
      mergeWith() { return null; },
    },
    { originId: 'client-a', sequence: 8 },
  );

  assert.equal(op?.kind, 'deleteText');
  assert.equal(op?.count, 2);
  assert.equal(op?.direction, 'backward');
  assert.equal(op?.deletedText, '요세');
  assert.equal(op?.timestamp, 5678);
});

test('line break and tab commands serialize as insert text realtime operations', () => {
  const lineBreak = createRealtimeOperationFromCommand(
    {
      type: 'insertLineBreak',
      position: pos(2),
      timestamp: 1111,
      execute() { return pos(3); },
      undo() { return pos(2); },
      mergeWith() { return null; },
    },
    { originId: 'client-a', sequence: 9 },
  );
  const tab = createRealtimeOperationFromCommand(
    {
      type: 'insertTab',
      position: pos(3),
      timestamp: 2222,
      execute() { return pos(4); },
      undo() { return pos(3); },
      mergeWith() { return null; },
    },
    { originId: 'client-a', sequence: 10 },
  );

  assert.equal(lineBreak?.kind, 'insertText');
  assert.equal(lineBreak?.text, '\n');
  assert.equal(tab?.kind, 'insertText');
  assert.equal(tab?.text, '\t');
});

test('char format commands serialize selected ranges to rhwp realtime operations', () => {
  const op = createRealtimeOperationFromCommand(
    {
      type: 'applyCharFormat',
      start: pos(2),
      end: pos(7),
      props: { bold: true, fontSize: 1400 },
      timestamp: 3333,
      execute() { return pos(2); },
      undo() { return pos(2); },
      mergeWith() { return null; },
    },
    { originId: 'client-a', sequence: 11 },
  );

  assert.equal(op?.kind, 'applyCharFormat');
  assert.deepEqual(op?.position, pos(2));
  assert.deepEqual(op?.start, pos(2));
  assert.deepEqual(op?.end, pos(7));
  assert.deepEqual(op?.props, { bold: true, fontSize: 1400 });
});

test('para format commands serialize targets to rhwp realtime operations', () => {
  const targets = [
    { kind: 'body' as const, sec: 0, para: 2 },
    { kind: 'body' as const, sec: 0, para: 3 },
  ];
  const op = createRealtimeOperationFromCommand(
    {
      type: 'applyParaFormat',
      targets,
      props: { alignment: 'center' },
      cursorBefore: pos(4),
      timestamp: 4444,
      execute() { return pos(4); },
      undo() { return pos(4); },
      mergeWith() { return null; },
    },
    { originId: 'client-a', sequence: 12 },
  );

  assert.equal(op?.kind, 'applyParaFormat');
  assert.deepEqual(op?.position, pos(4));
  assert.deepEqual(op?.targets, targets);
  assert.deepEqual(op?.props, { alignment: 'center' });
});

test('body paragraph split and merge commands serialize to structural realtime operations', () => {
  const split = createRealtimeOperationFromCommand(
    {
      type: 'splitParagraph',
      position: pos(3),
      timestamp: 5555,
      execute() { return { sectionIndex: 0, paragraphIndex: 3, charOffset: 0 }; },
      undo() { return pos(3); },
      mergeWith() { return null; },
    },
    { originId: 'client-a', sequence: 13 },
  );
  const merge = createRealtimeOperationFromCommand(
    {
      type: 'mergeParagraph',
      position: { sectionIndex: 0, paragraphIndex: 3, charOffset: 0 },
      timestamp: 6666,
      execute() { return pos(8); },
      undo() { return { sectionIndex: 0, paragraphIndex: 3, charOffset: 0 }; },
      mergeWith() { return null; },
    },
    { originId: 'client-a', sequence: 14 },
  );

  assert.equal(split?.kind, 'splitParagraph');
  assert.deepEqual(split?.position, pos(3));
  assert.equal(merge?.kind, 'mergeParagraph');
  assert.deepEqual(merge?.position, { sectionIndex: 0, paragraphIndex: 3, charOffset: 0 });
});

test('cell paragraph split and merge commands serialize to structural realtime operations', () => {
  const split = createRealtimeOperationFromCommand(
    {
      type: 'splitParagraphInCell',
      position: cellPos(1, 3),
      timestamp: 7777,
      execute() { return cellPos(2, 0); },
      undo() { return cellPos(1, 3); },
      mergeWith() { return null; },
    },
    { originId: 'client-a', sequence: 15 },
  );
  const merge = createRealtimeOperationFromCommand(
    {
      type: 'mergeParagraphInCell',
      position: cellPos(2, 0),
      mergePointOffset: 6,
      timestamp: 8888,
      execute() { return cellPos(1, 6); },
      undo() { return cellPos(2, 0); },
      mergeWith() { return null; },
    },
    { originId: 'client-a', sequence: 16 },
  );
  const mergeNext = createRealtimeOperationFromCommand(
    {
      type: 'mergeNextParagraphInCell',
      position: cellPos(1, 6),
      timestamp: 9999,
      execute() { return cellPos(1, 6); },
      undo() { return cellPos(1, 6); },
      mergeWith() { return null; },
    },
    { originId: 'client-a', sequence: 17 },
  );

  assert.equal(split?.kind, 'splitParagraphInCell');
  assert.deepEqual(split?.position, cellPos(1, 3));
  assert.equal(merge?.kind, 'mergeParagraphInCell');
  assert.equal(merge?.mergePointOffset, 6);
  assert.deepEqual(merge?.position, cellPos(2, 0));
  assert.equal(mergeNext?.kind, 'mergeNextParagraphInCell');
  assert.equal(mergeNext?.mergePointOffset, 6);
  assert.deepEqual(mergeNext?.position, cellPos(1, 6));
});

test('table and object movement commands serialize to realtime operations', () => {
  const table = createRealtimeOperationFromCommand(
    {
      type: 'moveTable',
      sec: 0,
      ppi: 4,
      ci: 1,
      deltaH: 120,
      deltaV: -80,
      resultPpi: 5,
      resultCi: 2,
      timestamp: 1010,
      execute() { return { sectionIndex: 0, paragraphIndex: 5, charOffset: 0 }; },
      undo() { return { sectionIndex: 0, paragraphIndex: 4, charOffset: 0 }; },
      mergeWith() { return null; },
    },
    { originId: 'client-a', sequence: 18 },
  );
  const picture = createRealtimeOperationFromCommand(
    {
      type: 'movePicture',
      sec: 0,
      ppi: 6,
      ci: 3,
      deltaH: 40,
      deltaV: 60,
      origHorzOffset: 1000,
      origVertOffset: 2000,
      timestamp: 2020,
      execute() { return { sectionIndex: 0, paragraphIndex: 6, charOffset: 0 }; },
      undo() { return { sectionIndex: 0, paragraphIndex: 6, charOffset: 0 }; },
      mergeWith() { return null; },
    },
    { originId: 'client-a', sequence: 19 },
  );
  const shape = createRealtimeOperationFromCommand(
    {
      type: 'moveShape',
      sec: 0,
      ppi: 7,
      ci: 4,
      deltaH: -20,
      deltaV: 30,
      origHorzOffset: 3000,
      origVertOffset: 4000,
      cellPath: [{ controlIndex: 0, cellIndex: 1, cellParaIndex: 0 }],
      timestamp: 3030,
      execute() { return { sectionIndex: 0, paragraphIndex: 7, charOffset: 0 }; },
      undo() { return { sectionIndex: 0, paragraphIndex: 7, charOffset: 0 }; },
      mergeWith() { return null; },
    },
    { originId: 'client-a', sequence: 20 },
  );

  assert.equal(table?.kind, 'moveTable');
  assert.deepEqual(table?.position, { sectionIndex: 0, paragraphIndex: 4, charOffset: 0 });
  assert.equal(table?.sec, 0);
  assert.equal(table?.ppi, 4);
  assert.equal(table?.ci, 1);
  assert.equal(table?.deltaH, 120);
  assert.equal(table?.deltaV, -80);
  assert.equal(table?.resultPpi, 5);
  assert.equal(table?.resultCi, 2);

  assert.equal(picture?.kind, 'movePicture');
  assert.deepEqual(picture?.position, { sectionIndex: 0, paragraphIndex: 6, charOffset: 0 });
  assert.equal(picture?.objectType, 'image');
  assert.equal(picture?.ppi, 6);
  assert.equal(picture?.ci, 3);
  assert.equal(picture?.deltaH, 40);
  assert.equal(picture?.deltaV, 60);
  assert.equal(picture?.origHorzOffset, 1000);
  assert.equal(picture?.origVertOffset, 2000);

  assert.equal(shape?.kind, 'moveShape');
  assert.equal(shape?.objectType, 'shape');
  assert.deepEqual(shape?.cellPath, [{ controlIndex: 0, cellIndex: 1, cellParaIndex: 0 }]);
});

test('object resize commands serialize object targets to realtime operations', () => {
  const objectTargets = [
    {
      sec: 0,
      ppi: 6,
      ci: 3,
      type: 'image',
      before: { width: 1200, height: 800, horzOffset: 100 },
      after: { width: 1500, height: 900, horzOffset: 180 },
    },
    {
      sec: 0,
      ppi: 7,
      ci: 4,
      type: 'shape',
      cellPath: [{ controlIndex: 0, cellIndex: 1, cellParaIndex: 0 }],
      before: { width: 500 },
      after: { width: 700 },
    },
  ];
  const op = createRealtimeOperationFromCommand(
    {
      type: 'resizeObject',
      targets: objectTargets,
      timestamp: 4040,
      execute() { return { sectionIndex: 0, paragraphIndex: 6, charOffset: 0 }; },
      undo() { return { sectionIndex: 0, paragraphIndex: 6, charOffset: 0 }; },
      mergeWith() { return null; },
    },
    { originId: 'client-a', sequence: 21 },
  );

  assert.equal(op?.kind, 'resizeObject');
  assert.deepEqual(op?.position, { sectionIndex: 0, paragraphIndex: 6, charOffset: 0 });
  assert.deepEqual(op?.objectTargets, objectTargets);
});

test('table row and column snapshots can create realtime operations from drafts', () => {
  const insertRow = createRealtimeOperationFromDraft(
    {
      kind: 'insertTableRow',
      position: { sectionIndex: 0, paragraphIndex: 4, charOffset: 0 },
      sec: 0,
      ppi: 4,
      ci: 1,
      rowIndex: 2,
      insertAfter: true,
      tableCount: 3,
      timestamp: 5050,
    },
    { originId: 'client-a', sequence: 22 },
  );
  const deleteColumn = createRealtimeOperationFromDraft(
    {
      kind: 'deleteTableColumn',
      position: { sectionIndex: 0, paragraphIndex: 4, charOffset: 0 },
      sec: 0,
      ppi: 4,
      ci: 1,
      colIndex: 5,
      tableCount: 1,
      timestamp: 6060,
    },
    { originId: 'client-a', sequence: 23 },
  );

  assert.equal(insertRow.kind, 'insertTableRow');
  assert.equal(insertRow.contract, 'rhwp-realtime-op/v1');
  assert.equal(insertRow.opId, 'client-a:22');
  assert.equal(insertRow.originId, 'client-a');
  assert.equal(insertRow.sequence, 22);
  assert.equal(insertRow.rowIndex, 2);
  assert.equal(insertRow.insertAfter, true);
  assert.equal(insertRow.tableCount, 3);
  assert.equal(insertRow.timestamp, 5050);

  assert.equal(deleteColumn.kind, 'deleteTableColumn');
  assert.equal(deleteColumn.opId, 'client-a:23');
  assert.equal(deleteColumn.colIndex, 5);
  assert.equal(deleteColumn.tableCount, 1);
});

test('table cell merge and split snapshots can create realtime operations from drafts', () => {
  const merge = createRealtimeOperationFromDraft(
    {
      kind: 'mergeTableCells',
      position: { sectionIndex: 0, paragraphIndex: 4, charOffset: 0 },
      sec: 0,
      ppi: 4,
      ci: 1,
      startRow: 1,
      startCol: 2,
      endRow: 3,
      endCol: 4,
      timestamp: 7070,
    },
    { originId: 'client-a', sequence: 24 },
  );
  const split = createRealtimeOperationFromDraft(
    {
      kind: 'splitTableCell',
      position: { sectionIndex: 0, paragraphIndex: 4, charOffset: 0 },
      sec: 0,
      ppi: 4,
      ci: 1,
      rowIndex: 3,
      colIndex: 5,
      splitRows: 2,
      splitCols: 3,
      equalRowHeight: true,
      mergeFirst: false,
      timestamp: 8080,
    },
    { originId: 'client-a', sequence: 25 },
  );
  const splitRange = createRealtimeOperationFromDraft(
    {
      kind: 'splitTableCellsInRange',
      position: { sectionIndex: 0, paragraphIndex: 4, charOffset: 0 },
      sec: 0,
      ppi: 4,
      ci: 1,
      startRow: 1,
      startCol: 2,
      endRow: 3,
      endCol: 4,
      splitRows: 2,
      splitCols: 2,
      equalRowHeight: false,
      timestamp: 9090,
    },
    { originId: 'client-a', sequence: 26 },
  );

  assert.equal(merge.kind, 'mergeTableCells');
  assert.equal(merge.opId, 'client-a:24');
  assert.equal(merge.startRow, 1);
  assert.equal(merge.endCol, 4);

  assert.equal(split.kind, 'splitTableCell');
  assert.equal(split.opId, 'client-a:25');
  assert.equal(split.rowIndex, 3);
  assert.equal(split.colIndex, 5);
  assert.equal(split.splitRows, 2);
  assert.equal(split.splitCols, 3);
  assert.equal(split.equalRowHeight, true);
  assert.equal(split.mergeFirst, false);

  assert.equal(splitRange.kind, 'splitTableCellsInRange');
  assert.equal(splitRange.opId, 'client-a:26');
  assert.equal(splitRange.startRow, 1);
  assert.equal(splitRange.endCol, 4);
  assert.equal(splitRange.splitRows, 2);
  assert.equal(splitRange.splitCols, 2);
  assert.equal(splitRange.equalRowHeight, false);
});

test('table create and delete snapshots can create realtime operations from drafts', () => {
  const create = createRealtimeOperationFromDraft(
    {
      kind: 'createTable',
      position: { sectionIndex: 0, paragraphIndex: 4, charOffset: 7 },
      rowCount: 3,
      colCount: 4,
      tableOptions: {
        treatAsChar: true,
        colWidths: [1200, 1300, 1400, 1500],
        rowHeights: [600, 700, 800],
      },
      timestamp: 10010,
    },
    { originId: 'client-a', sequence: 27 },
  );
  const remove = createRealtimeOperationFromDraft(
    {
      kind: 'deleteTable',
      position: { sectionIndex: 0, paragraphIndex: 5, charOffset: 0 },
      sec: 0,
      ppi: 5,
      ci: 2,
      timestamp: 10020,
    },
    { originId: 'client-a', sequence: 28 },
  );

  assert.equal(create.kind, 'createTable');
  assert.equal(create.opId, 'client-a:27');
  assert.deepEqual(create.position, { sectionIndex: 0, paragraphIndex: 4, charOffset: 7 });
  assert.equal(create.rowCount, 3);
  assert.equal(create.colCount, 4);
  assert.deepEqual(create.tableOptions, {
    treatAsChar: true,
    colWidths: [1200, 1300, 1400, 1500],
    rowHeights: [600, 700, 800],
  });

  assert.equal(remove.kind, 'deleteTable');
  assert.equal(remove.opId, 'client-a:28');
  assert.deepEqual(remove.position, { sectionIndex: 0, paragraphIndex: 5, charOffset: 0 });
  assert.equal(remove.ppi, 5);
  assert.equal(remove.ci, 2);
});

test('table and cell property snapshots can create realtime operations from drafts', () => {
  const tableDraft: any = {
    kind: 'setTableProperties',
    position: { sectionIndex: 0, paragraphIndex: 5, charOffset: 0 },
    sec: 0,
    ppi: 5,
    ci: 2,
    tableProps: {
      hasCaption: true,
      border: { color: '#123456' },
    },
    timestamp: 11010,
  };
  const cellDraft: any = {
    kind: 'setCellProperties',
    position: { sectionIndex: 0, paragraphIndex: 5, charOffset: 0 },
    sec: 0,
    ppi: 5,
    ci: 2,
    cellIndex: 4,
    cellProps: {
      width: 2400,
      fill: { color: '#abcdef' },
    },
    timestamp: 11020,
  };
  const resizeDraft: any = {
    kind: 'resizeTableCells',
    position: { sectionIndex: 0, paragraphIndex: 5, charOffset: 0 },
    sec: 0,
    ppi: 5,
    ci: 2,
    cellUpdates: [
      { cellIdx: 4, widthDelta: 120, renderWidth: 2400, localResize: true },
    ],
    timestamp: 11030,
  };

  const table = createRealtimeOperationFromDraft(tableDraft, { originId: 'client-a', sequence: 29 }) as any;
  const cell = createRealtimeOperationFromDraft(cellDraft, { originId: 'client-a', sequence: 30 }) as any;
  const resize = createRealtimeOperationFromDraft(resizeDraft, { originId: 'client-a', sequence: 31 }) as any;
  tableDraft.tableProps.border.color = '#000000';
  cellDraft.cellProps.fill.color = '#000000';
  resizeDraft.cellUpdates[0].widthDelta = 999;

  assert.equal(table.kind, 'setTableProperties');
  assert.equal(table.opId, 'client-a:29');
  assert.equal(table.tableProps.border.color, '#123456');
  assert.equal(cell.kind, 'setCellProperties');
  assert.equal(cell.cellIndex, 4);
  assert.equal(cell.cellProps.fill.color, '#abcdef');
  assert.equal(resize.kind, 'resizeTableCells');
  assert.deepEqual(resize.cellUpdates, [
    { cellIdx: 4, widthDelta: 120, renderWidth: 2400, localResize: true },
  ]);
});

test('remote insert/delete operations transform local cursor in the same paragraph', () => {
  const insertOp: RhwpRealtimeOperation = {
    contract: 'rhwp-realtime-op/v1',
    opId: 'client-b:1',
    originId: 'client-b',
    sequence: 1,
    kind: 'insertText',
    position: pos(3),
    text: 'abc',
    timestamp: 1,
  };
  assert.deepEqual(transformCursorAfterRemoteOperation(pos(5), insertOp), pos(8));

  const deleteOp: RhwpRealtimeOperation = {
    contract: 'rhwp-realtime-op/v1',
    opId: 'client-b:2',
    originId: 'client-b',
    sequence: 2,
    kind: 'deleteText',
    position: pos(2),
    count: 4,
    direction: 'forward',
    timestamp: 2,
  };
  assert.deepEqual(transformCursorAfterRemoteOperation(pos(8), deleteOp), pos(4));
  assert.deepEqual(transformCursorAfterRemoteOperation(pos(4), deleteOp), pos(2));
});

test('concurrent inserts at the same offset converge using origin order', () => {
  const localA: RhwpRealtimeOperation = {
    contract: 'rhwp-realtime-op/v1',
    opId: 'client-a:1',
    originId: 'client-a',
    sequence: 1,
    kind: 'insertText',
    position: pos(0),
    text: 'A',
    timestamp: 1,
  };
  const localB: RhwpRealtimeOperation = {
    contract: 'rhwp-realtime-op/v1',
    opId: 'client-b:1',
    originId: 'client-b',
    sequence: 1,
    kind: 'insertText',
    position: pos(0),
    text: 'B',
    timestamp: 1,
  };

  const remoteBOnA = transformRemoteOperationAgainstLocalHistory(localB, [localA]);
  const remoteAOnB = transformRemoteOperationAgainstLocalHistory(localA, [localB]);

  assert.deepEqual(remoteBOnA.position, pos(1));
  assert.deepEqual(remoteAOnB.position, pos(0));
});

test('remote delete spanning a concurrent local insert preserves the inserted text with delete spans', () => {
  const localInsert: RhwpRealtimeOperation = {
    contract: 'rhwp-realtime-op/v1',
    opId: 'client-a:1',
    originId: 'client-a',
    sequence: 1,
    kind: 'insertText',
    position: pos(2),
    text: 'X',
    timestamp: 1,
  };
  const remoteDelete: RhwpRealtimeOperation = {
    contract: 'rhwp-realtime-op/v1',
    opId: 'client-b:1',
    originId: 'client-b',
    sequence: 1,
    kind: 'deleteText',
    position: pos(1),
    count: 2,
    direction: 'forward',
    timestamp: 2,
  };

  const transformed = transformRemoteOperationAgainstLocalHistory(remoteDelete, [localInsert]);

  assert.equal(transformed.kind, 'deleteTextSpans');
  assert.deepEqual(transformed.spans, [
    { position: pos(1), count: 1 },
    { position: pos(3), count: 1 },
  ]);
  assert.deepEqual(transformCursorAfterRemoteOperation(pos(5), transformed), pos(3));
});

test('char format ranges move across concurrent text inserts', () => {
  const localInsert: RhwpRealtimeOperation = {
    contract: 'rhwp-realtime-op/v1',
    opId: 'client-a:1',
    originId: 'client-a',
    sequence: 1,
    kind: 'insertText',
    position: pos(2),
    text: 'XX',
    timestamp: 1,
  };
  const remoteFormat: RhwpRealtimeOperation = {
    contract: 'rhwp-realtime-op/v1',
    opId: 'client-b:1',
    originId: 'client-b',
    sequence: 1,
    kind: 'applyCharFormat',
    position: pos(4),
    start: pos(4),
    end: pos(8),
    props: { italic: true },
    timestamp: 2,
  };

  const transformed = transformRemoteOperationAgainstLocalHistory(remoteFormat, [localInsert]);

  assert.equal(transformed.kind, 'applyCharFormat');
  assert.deepEqual(transformed.position, pos(6));
  assert.deepEqual(transformed.start, pos(6));
  assert.deepEqual(transformed.end, pos(10));
});

test('remote operations after a concurrent local split move to the next paragraph', () => {
  const localSplit: RhwpRealtimeOperation = {
    contract: 'rhwp-realtime-op/v1',
    opId: 'client-a:split',
    originId: 'client-a',
    sequence: 1,
    kind: 'splitParagraph',
    position: pos(3),
    timestamp: 1,
  };
  const remoteInsertAfterSplitPoint: RhwpRealtimeOperation = {
    contract: 'rhwp-realtime-op/v1',
    opId: 'client-b:insert',
    originId: 'client-b',
    sequence: 1,
    kind: 'insertText',
    position: pos(6),
    text: 'Z',
    timestamp: 2,
  };
  const remoteLaterParagraphFormat: RhwpRealtimeOperation = {
    contract: 'rhwp-realtime-op/v1',
    opId: 'client-b:format',
    originId: 'client-b',
    sequence: 2,
    kind: 'applyParaFormat',
    position: { sectionIndex: 0, paragraphIndex: 4, charOffset: 0 },
    cursorBefore: { sectionIndex: 0, paragraphIndex: 4, charOffset: 0 },
    targets: [{ kind: 'body', sec: 0, para: 4 }],
    props: { alignment: 'center' },
    timestamp: 3,
  };

  const movedInsert = transformRemoteOperationAgainstLocalHistory(remoteInsertAfterSplitPoint, [localSplit]);
  const movedFormat = transformRemoteOperationAgainstLocalHistory(remoteLaterParagraphFormat, [localSplit]);

  assert.deepEqual(movedInsert.position, { sectionIndex: 0, paragraphIndex: 3, charOffset: 3 });
  assert.deepEqual(movedFormat.position, { sectionIndex: 0, paragraphIndex: 5, charOffset: 0 });
  assert.deepEqual(movedFormat.cursorBefore, { sectionIndex: 0, paragraphIndex: 5, charOffset: 0 });
  assert.deepEqual(movedFormat.targets, [{ kind: 'body', sec: 0, para: 5 }]);
});

test('remote later-paragraph operations move back after a concurrent local merge', () => {
  const localMerge: RhwpRealtimeOperation = {
    contract: 'rhwp-realtime-op/v1',
    opId: 'client-a:merge',
    originId: 'client-a',
    sequence: 1,
    kind: 'mergeParagraph',
    position: { sectionIndex: 0, paragraphIndex: 3, charOffset: 0 },
    mergePointOffset: 5,
    timestamp: 1,
  };
  const remoteLaterInsert: RhwpRealtimeOperation = {
    contract: 'rhwp-realtime-op/v1',
    opId: 'client-b:insert',
    originId: 'client-b',
    sequence: 1,
    kind: 'insertText',
    position: { sectionIndex: 0, paragraphIndex: 4, charOffset: 2 },
    text: 'Z',
    timestamp: 2,
  };

  const transformed = transformRemoteOperationAgainstLocalHistory(remoteLaterInsert, [localMerge]);

  assert.deepEqual(transformed.position, { sectionIndex: 0, paragraphIndex: 3, charOffset: 2 });
});

test('remote table and object moves rebase their anchors across concurrent local body structure edits', () => {
  const localSplit: RhwpRealtimeOperation = {
    contract: 'rhwp-realtime-op/v1',
    opId: 'client-a:split',
    originId: 'client-a',
    sequence: 1,
    kind: 'splitParagraph',
    position: { sectionIndex: 0, paragraphIndex: 2, charOffset: 0 },
    timestamp: 1,
  };
  const remoteTableMove: RhwpRealtimeOperation = {
    contract: 'rhwp-realtime-op/v1',
    opId: 'client-b:table',
    originId: 'client-b',
    sequence: 1,
    kind: 'moveTable',
    position: { sectionIndex: 0, paragraphIndex: 4, charOffset: 0 },
    sec: 0,
    ppi: 4,
    ci: 1,
    deltaH: 120,
    deltaV: -80,
    resultPpi: 5,
    resultCi: 2,
    timestamp: 2,
  };
  const movedTable = transformRemoteOperationAgainstLocalHistory(remoteTableMove, [localSplit]);

  assert.deepEqual(movedTable.position, { sectionIndex: 0, paragraphIndex: 5, charOffset: 0 });
  assert.equal(movedTable.ppi, 5);
  assert.equal(movedTable.resultPpi, 6);

  const localMerge: RhwpRealtimeOperation = {
    contract: 'rhwp-realtime-op/v1',
    opId: 'client-a:merge',
    originId: 'client-a',
    sequence: 2,
    kind: 'mergeParagraph',
    position: { sectionIndex: 0, paragraphIndex: 3, charOffset: 0 },
    mergePointOffset: 4,
    timestamp: 3,
  };
  const remotePictureMove: RhwpRealtimeOperation = {
    contract: 'rhwp-realtime-op/v1',
    opId: 'client-b:picture',
    originId: 'client-b',
    sequence: 2,
    kind: 'movePicture',
    objectType: 'image',
    position: { sectionIndex: 0, paragraphIndex: 5, charOffset: 0 },
    sec: 0,
    ppi: 5,
    ci: 3,
    deltaH: 40,
    deltaV: 60,
    origHorzOffset: 1000,
    origVertOffset: 2000,
    timestamp: 4,
  };
  const movedPicture = transformRemoteOperationAgainstLocalHistory(remotePictureMove, [localMerge]);

  assert.deepEqual(movedPicture.position, { sectionIndex: 0, paragraphIndex: 4, charOffset: 0 });
  assert.equal(movedPicture.ppi, 4);
});

test('remote object resize targets rebase across concurrent local body structure edits', () => {
  const localSplit: RhwpRealtimeOperation = {
    contract: 'rhwp-realtime-op/v1',
    opId: 'client-a:split',
    originId: 'client-a',
    sequence: 1,
    kind: 'splitParagraph',
    position: { sectionIndex: 0, paragraphIndex: 2, charOffset: 0 },
    timestamp: 1,
  };
  const remoteResize: RhwpRealtimeOperation = {
    contract: 'rhwp-realtime-op/v1',
    opId: 'client-b:resize',
    originId: 'client-b',
    sequence: 1,
    kind: 'resizeObject',
    position: { sectionIndex: 0, paragraphIndex: 5, charOffset: 0 },
    objectTargets: [{
      sec: 0,
      ppi: 5,
      ci: 2,
      type: 'image',
      before: { width: 1000 },
      after: { width: 1200 },
    }],
    timestamp: 2,
  };

  const movedResize = transformRemoteOperationAgainstLocalHistory(remoteResize, [localSplit]);

  assert.deepEqual(movedResize.position, { sectionIndex: 0, paragraphIndex: 6, charOffset: 0 });
  assert.deepEqual(movedResize.objectTargets, [{
    sec: 0,
    ppi: 6,
    ci: 2,
    type: 'image',
    before: { width: 1000 },
    after: { width: 1200 },
  }]);
});

test('remote object delete anchors rebase across concurrent local body structure edits', () => {
  const localSplit: RhwpRealtimeOperation = {
    contract: 'rhwp-realtime-op/v1',
    opId: 'client-a:split',
    originId: 'client-a',
    sequence: 1,
    kind: 'splitParagraph',
    position: { sectionIndex: 0, paragraphIndex: 2, charOffset: 0 },
    timestamp: 1,
  };
  const remoteDelete = createRealtimeOperationFromDraft({
    kind: 'deleteObject',
    position: { sectionIndex: 0, paragraphIndex: 5, charOffset: 0 },
    sec: 0,
    ppi: 5,
    ci: 2,
    objectType: 'image',
    cellPath: [{ controlIdx: 1, cellIdx: 3, cellParaIdx: 0 }],
  }, { originId: 'client-b', sequence: 1 });

  const movedDelete = transformRemoteOperationAgainstLocalHistory(remoteDelete, [localSplit]);

  assert.deepEqual(movedDelete.position, { sectionIndex: 0, paragraphIndex: 6, charOffset: 0 });
  assert.equal(movedDelete.kind, 'deleteObject');
  assert.equal(movedDelete.ppi, 6);
  assert.equal(movedDelete.objectType, 'image');
  assert.deepEqual(movedDelete.cellPath, [{ controlIdx: 1, cellIdx: 3, cellParaIdx: 0 }]);
});

test('remote structural inserts move local cursor like a one-character control', () => {
  const remoteEquation = createRealtimeOperationFromDraft({
    kind: 'insertEquation',
    position: pos(3),
    equationText: '',
    fontSize: 1000,
    color: 0,
  }, { originId: 'client-b', sequence: 1 });

  const movedCursor = transformCursorAfterRemoteOperation(pos(5), remoteEquation);

  assert.equal(remoteEquation.kind, 'insertEquation');
  assert.equal(movedCursor.charOffset, 6);
});

test('remote group shape targets rebase across concurrent local body structure edits', () => {
  const localSplit: RhwpRealtimeOperation = {
    contract: 'rhwp-realtime-op/v1',
    opId: 'client-a:split',
    originId: 'client-a',
    sequence: 1,
    kind: 'splitParagraph',
    position: { sectionIndex: 0, paragraphIndex: 2, charOffset: 0 },
    timestamp: 1,
  };
  const remoteGroup = createRealtimeOperationFromDraft({
    kind: 'groupShapes',
    position: { sectionIndex: 0, paragraphIndex: 5, charOffset: 0 },
    sec: 0,
    shapeTargets: [
      { ppi: 5, ci: 1 },
      { ppi: 6, ci: 2 },
    ],
    resultPpi: 5,
    resultCi: 3,
  }, { originId: 'client-b', sequence: 1 });

  const movedGroup = transformRemoteOperationAgainstLocalHistory(remoteGroup, [localSplit]);

  assert.equal(movedGroup.kind, 'groupShapes');
  assert.equal(movedGroup.ppi, undefined);
  assert.deepEqual(movedGroup.position, { sectionIndex: 0, paragraphIndex: 6, charOffset: 0 });
  assert.deepEqual(movedGroup.shapeTargets, [
    { ppi: 6, ci: 1 },
    { ppi: 7, ci: 2 },
  ]);
  assert.equal(movedGroup.resultPpi, 6);
  assert.equal(movedGroup.resultCi, 3);
});

test('remote table row and column operations rebase their table anchors across body structure edits', () => {
  const localSplit: RhwpRealtimeOperation = {
    contract: 'rhwp-realtime-op/v1',
    opId: 'client-a:split',
    originId: 'client-a',
    sequence: 1,
    kind: 'splitParagraph',
    position: { sectionIndex: 0, paragraphIndex: 2, charOffset: 0 },
    timestamp: 1,
  };
  const remoteInsertRow: RhwpRealtimeOperation = {
    contract: 'rhwp-realtime-op/v1',
    opId: 'client-b:insert-row',
    originId: 'client-b',
    sequence: 1,
    kind: 'insertTableRow',
    position: { sectionIndex: 0, paragraphIndex: 5, charOffset: 0 },
    sec: 0,
    ppi: 5,
    ci: 2,
    rowIndex: 3,
    insertAfter: false,
    tableCount: 2,
    timestamp: 2,
  };

  const moved = transformRemoteOperationAgainstLocalHistory(remoteInsertRow, [localSplit]);

  assert.deepEqual(moved.position, { sectionIndex: 0, paragraphIndex: 6, charOffset: 0 });
  assert.equal(moved.ppi, 6);
  assert.equal(moved.rowIndex, 3);
  assert.equal(moved.tableCount, 2);
});

test('remote table cell merge and split operations rebase their table anchors across body structure edits', () => {
  const localSplit: RhwpRealtimeOperation = {
    contract: 'rhwp-realtime-op/v1',
    opId: 'client-a:split',
    originId: 'client-a',
    sequence: 1,
    kind: 'splitParagraph',
    position: { sectionIndex: 0, paragraphIndex: 2, charOffset: 0 },
    timestamp: 1,
  };
  const remoteMerge: RhwpRealtimeOperation = {
    contract: 'rhwp-realtime-op/v1',
    opId: 'client-b:merge-cells',
    originId: 'client-b',
    sequence: 1,
    kind: 'mergeTableCells',
    position: { sectionIndex: 0, paragraphIndex: 5, charOffset: 0 },
    sec: 0,
    ppi: 5,
    ci: 2,
    startRow: 1,
    startCol: 2,
    endRow: 3,
    endCol: 4,
    timestamp: 2,
  };

  const moved = transformRemoteOperationAgainstLocalHistory(remoteMerge, [localSplit]);

  assert.deepEqual(moved.position, { sectionIndex: 0, paragraphIndex: 6, charOffset: 0 });
  assert.equal(moved.ppi, 6);
  assert.equal(moved.startRow, 1);
  assert.equal(moved.endCol, 4);
});

test('remote table create and delete operations rebase across body structure edits', () => {
  const localSplit: RhwpRealtimeOperation = {
    contract: 'rhwp-realtime-op/v1',
    opId: 'client-a:split',
    originId: 'client-a',
    sequence: 1,
    kind: 'splitParagraph',
    position: { sectionIndex: 0, paragraphIndex: 2, charOffset: 0 },
    timestamp: 1,
  };
  const remoteCreate: RhwpRealtimeOperation = {
    contract: 'rhwp-realtime-op/v1',
    opId: 'client-b:create-table',
    originId: 'client-b',
    sequence: 1,
    kind: 'createTable',
    position: { sectionIndex: 0, paragraphIndex: 5, charOffset: 4 },
    rowCount: 2,
    colCount: 3,
    timestamp: 2,
  };
  const remoteDelete: RhwpRealtimeOperation = {
    contract: 'rhwp-realtime-op/v1',
    opId: 'client-b:delete-table',
    originId: 'client-b',
    sequence: 2,
    kind: 'deleteTable',
    position: { sectionIndex: 0, paragraphIndex: 6, charOffset: 0 },
    sec: 0,
    ppi: 6,
    ci: 1,
    timestamp: 3,
  };

  const movedCreate = transformRemoteOperationAgainstLocalHistory(remoteCreate, [localSplit]);
  const movedDelete = transformRemoteOperationAgainstLocalHistory(remoteDelete, [localSplit]);

  assert.deepEqual(movedCreate.position, { sectionIndex: 0, paragraphIndex: 6, charOffset: 4 });
  assert.equal(movedCreate.rowCount, 2);
  assert.equal(movedCreate.colCount, 3);
  assert.deepEqual(movedDelete.position, { sectionIndex: 0, paragraphIndex: 7, charOffset: 0 });
  assert.equal(movedDelete.ppi, 7);
  assert.equal(movedDelete.ci, 1);
});

test('remote table property operations rebase across body structure edits', () => {
  const localSplit: RhwpRealtimeOperation = {
    contract: 'rhwp-realtime-op/v1',
    opId: 'client-a:split',
    originId: 'client-a',
    sequence: 1,
    kind: 'splitParagraph',
    position: { sectionIndex: 0, paragraphIndex: 2, charOffset: 0 },
    timestamp: 1,
  };
  const remoteCellProps = {
    contract: 'rhwp-realtime-op/v1',
    opId: 'client-b:set-cell',
    originId: 'client-b',
    sequence: 1,
    kind: 'setCellProperties',
    position: { sectionIndex: 0, paragraphIndex: 6, charOffset: 0 },
    sec: 0,
    ppi: 6,
    ci: 1,
    cellIndex: 3,
    cellProps: { width: 2400 },
    timestamp: 2,
  } as any;
  const remoteResize = {
    contract: 'rhwp-realtime-op/v1',
    opId: 'client-b:resize-cells',
    originId: 'client-b',
    sequence: 2,
    kind: 'resizeTableCells',
    position: { sectionIndex: 0, paragraphIndex: 7, charOffset: 0 },
    sec: 0,
    ppi: 7,
    ci: 1,
    cellUpdates: [{ cellIdx: 3, widthDelta: 120 }],
    timestamp: 3,
  } as any;

  const movedCellProps = transformRemoteOperationAgainstLocalHistory(remoteCellProps, [localSplit]) as any;
  const movedResize = transformRemoteOperationAgainstLocalHistory(remoteResize, [localSplit]) as any;

  assert.deepEqual(movedCellProps.position, { sectionIndex: 0, paragraphIndex: 7, charOffset: 0 });
  assert.equal(movedCellProps.ppi, 7);
  assert.equal(movedCellProps.cellIndex, 3);
  assert.deepEqual(movedCellProps.cellProps, { width: 2400 });
  assert.deepEqual(movedResize.position, { sectionIndex: 0, paragraphIndex: 8, charOffset: 0 });
  assert.equal(movedResize.ppi, 8);
  assert.deepEqual(movedResize.cellUpdates, [{ cellIdx: 3, widthDelta: 120 }]);
});

test('remote operations after a concurrent local cell split move inside the same cell', () => {
  const localSplit: RhwpRealtimeOperation = {
    contract: 'rhwp-realtime-op/v1',
    opId: 'client-a:cell-split',
    originId: 'client-a',
    sequence: 1,
    kind: 'splitParagraphInCell',
    position: cellPos(1, 3),
    timestamp: 1,
  };
  const remoteInsertAfterSplitPoint: RhwpRealtimeOperation = {
    contract: 'rhwp-realtime-op/v1',
    opId: 'client-b:insert',
    originId: 'client-b',
    sequence: 1,
    kind: 'insertText',
    position: cellPos(1, 6),
    text: 'Z',
    timestamp: 2,
  };
  const remoteLaterCellParaFormat: RhwpRealtimeOperation = {
    contract: 'rhwp-realtime-op/v1',
    opId: 'client-b:format',
    originId: 'client-b',
    sequence: 2,
    kind: 'applyParaFormat',
    position: cellPos(3, 0),
    cursorBefore: cellPos(3, 0),
    targets: [{ kind: 'cell', sec: 0, parentPara: 2, controlIdx: 0, cellIdx: 1, cellParaIdx: 3 }],
    props: { alignment: 'center' },
    timestamp: 3,
  };

  const movedInsert = transformRemoteOperationAgainstLocalHistory(remoteInsertAfterSplitPoint, [localSplit]);
  const movedFormat = transformRemoteOperationAgainstLocalHistory(remoteLaterCellParaFormat, [localSplit]);

  assert.deepEqual(movedInsert.position, cellPos(2, 3));
  assert.deepEqual(movedFormat.position, cellPos(4, 0));
  assert.deepEqual(movedFormat.cursorBefore, cellPos(4, 0));
  assert.deepEqual(movedFormat.targets, [
    { kind: 'cell', sec: 0, parentPara: 2, controlIdx: 0, cellIdx: 1, cellParaIdx: 4 },
  ]);
});

test('remote operations in removed cell paragraphs move back after a concurrent local cell merge', () => {
  const localMerge: RhwpRealtimeOperation = {
    contract: 'rhwp-realtime-op/v1',
    opId: 'client-a:cell-merge',
    originId: 'client-a',
    sequence: 1,
    kind: 'mergeParagraphInCell',
    position: cellPos(2, 0),
    mergePointOffset: 6,
    timestamp: 1,
  };
  const remoteInsertInRemovedPara: RhwpRealtimeOperation = {
    contract: 'rhwp-realtime-op/v1',
    opId: 'client-b:insert',
    originId: 'client-b',
    sequence: 1,
    kind: 'insertText',
    position: cellPos(2, 2),
    text: 'Z',
    timestamp: 2,
  };
  const remoteLaterCellParaFormat: RhwpRealtimeOperation = {
    contract: 'rhwp-realtime-op/v1',
    opId: 'client-b:format',
    originId: 'client-b',
    sequence: 2,
    kind: 'applyParaFormat',
    position: cellPos(3, 0),
    cursorBefore: cellPos(3, 0),
    targets: [{ kind: 'cell', sec: 0, parentPara: 2, controlIdx: 0, cellIdx: 1, cellParaIdx: 3 }],
    props: { alignment: 'right' },
    timestamp: 3,
  };

  const movedInsert = transformRemoteOperationAgainstLocalHistory(remoteInsertInRemovedPara, [localMerge]);
  const movedFormat = transformRemoteOperationAgainstLocalHistory(remoteLaterCellParaFormat, [localMerge]);

  assert.deepEqual(movedInsert.position, cellPos(1, 8));
  assert.deepEqual(movedFormat.position, cellPos(2, 0));
  assert.deepEqual(movedFormat.cursorBefore, cellPos(2, 0));
  assert.deepEqual(movedFormat.targets, [
    { kind: 'cell', sec: 0, parentPara: 2, controlIdx: 0, cellIdx: 1, cellParaIdx: 2 },
  ]);
});

test('overlapping concurrent deletes shrink the remote delete range instead of over-deleting', () => {
  const localDelete: RhwpRealtimeOperation = {
    contract: 'rhwp-realtime-op/v1',
    opId: 'client-a:1',
    originId: 'client-a',
    sequence: 1,
    kind: 'deleteText',
    position: pos(2),
    count: 2,
    direction: 'forward',
    timestamp: 1,
  };
  const remoteDelete: RhwpRealtimeOperation = {
    contract: 'rhwp-realtime-op/v1',
    opId: 'client-b:1',
    originId: 'client-b',
    sequence: 1,
    kind: 'deleteText',
    position: pos(1),
    count: 4,
    direction: 'forward',
    timestamp: 2,
  };

  const transformed = transformRemoteOperationAgainstLocalHistory(remoteDelete, [localDelete]);

  assert.equal(transformed.kind, 'deleteText');
  assert.deepEqual(transformed.position, pos(1));
  assert.equal(transformed.count, 2);
});
