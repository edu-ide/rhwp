import type { EditCommand, ParaFormatTarget } from './command';
import type { CellPathLike, CellProperties, CharProperties, DocumentPosition, ParaProperties, TableProperties } from '@/core/types';

export type RhwpRealtimeOperationKind =
  | 'insertText'
  | 'deleteText'
  | 'deleteTextSpans'
  | 'applyCharFormat'
  | 'applyParaFormat'
  | 'splitParagraph'
  | 'mergeParagraph'
  | 'mergeNextParagraph'
  | 'splitParagraphInCell'
  | 'mergeParagraphInCell'
  | 'mergeNextParagraphInCell'
  | 'moveTable'
  | 'movePicture'
  | 'moveShape'
  | 'resizeObject'
  | 'deleteObject'
  | 'insertEquation'
  | 'insertField'
  | 'insertFootnote'
  | 'insertEndnote'
  | 'changeShapeZOrder'
  | 'insertTableRow'
  | 'insertTableColumn'
  | 'deleteTableRow'
  | 'deleteTableColumn'
  | 'mergeTableCells'
  | 'splitTableCell'
  | 'splitTableCellsInRange'
  | 'createTable'
  | 'deleteTable'
  | 'setTableProperties'
  | 'setCellProperties'
  | 'resizeTableCells';

export interface RhwpRealtimeDeleteSpan {
  position: DocumentPosition;
  count: number;
  deletedText?: string;
}

export interface RhwpRealtimeObjectTarget {
  sec: number;
  ppi: number;
  ci: number;
  type: string;
  cellPath?: CellPathLike;
  before: Record<string, unknown>;
  after: Record<string, unknown>;
}

export type RhwpRealtimeObjectType = 'image' | 'shape' | 'equation' | 'group' | 'line';
export type RhwpRealtimeZOrderAction = 'front' | 'forward' | 'backward' | 'back';

export interface RhwpRealtimeTableCreateOptions {
  treatAsChar?: boolean;
  colWidths?: number[];
  rowHeights?: number[];
}

export interface RhwpRealtimeTableCellUpdate {
  cellIdx: number;
  widthDelta?: number;
  heightDelta?: number;
  renderWidth?: number;
  renderHeight?: number;
  localResize?: boolean;
}

export interface RhwpRealtimeOperation {
  contract: 'rhwp-realtime-op/v1';
  opId: string;
  originId: string;
  sequence: number;
  kind: RhwpRealtimeOperationKind;
  position: DocumentPosition;
  text?: string;
  count?: number;
  direction?: 'forward' | 'backward';
  deletedText?: string;
  spans?: RhwpRealtimeDeleteSpan[];
  start?: DocumentPosition;
  end?: DocumentPosition;
  props?: Partial<CharProperties> | Partial<ParaProperties>;
  targets?: ParaFormatTarget[];
  cursorBefore?: DocumentPosition;
  mergePointOffset?: number;
  sec?: number;
  ppi?: number;
  ci?: number;
  deltaH?: number;
  deltaV?: number;
  resultPpi?: number;
  resultCi?: number;
  objectType?: RhwpRealtimeObjectType;
  origHorzOffset?: number;
  origVertOffset?: number;
  cellPath?: CellPathLike;
  objectTargets?: RhwpRealtimeObjectTarget[];
  rowIndex?: number;
  colIndex?: number;
  startRow?: number;
  startCol?: number;
  endRow?: number;
  endCol?: number;
  insertAfter?: boolean;
  tableCount?: number;
  splitRows?: number;
  splitCols?: number;
  equalRowHeight?: boolean;
  mergeFirst?: boolean;
  rowCount?: number;
  colCount?: number;
  tableOptions?: RhwpRealtimeTableCreateOptions;
  cellIndex?: number;
  tableProps?: Partial<TableProperties>;
  cellProps?: Partial<CellProperties>;
  cellUpdates?: RhwpRealtimeTableCellUpdate[];
  equationText?: string;
  fontSize?: number;
  color?: number;
  fieldGuide?: string;
  fieldMemo?: string;
  fieldName?: string;
  fieldEditable?: boolean;
  zOrderAction?: RhwpRealtimeZOrderAction;
  timestamp: number;
}

export interface CreateRealtimeOperationOptions {
  originId: string;
  sequence: number;
}

export type RhwpRealtimeOperationDraft =
  Omit<RhwpRealtimeOperation, 'contract' | 'opId' | 'originId' | 'sequence' | 'timestamp'>
  & { timestamp?: number };

type CommandInternals = EditCommand & {
  position?: DocumentPosition;
  text?: string;
  count?: number;
  direction?: 'forward' | 'backward';
  deletedText?: string;
  start?: DocumentPosition;
  end?: DocumentPosition;
  props?: Partial<CharProperties> | Partial<ParaProperties>;
  targets?: any[];
  cursorBefore?: DocumentPosition;
  mergePointOffset?: number;
  sec?: number;
  ppi?: number;
  ci?: number;
  deltaH?: number;
  deltaV?: number;
  resultPpi?: number;
  resultCi?: number;
  origHorzOffset?: number;
  origVertOffset?: number;
  cellPath?: CellPathLike;
};

export function createRealtimeOriginId(): string {
  const cryptoApi = globalThis.crypto;
  if (cryptoApi?.randomUUID) return cryptoApi.randomUUID();
  return `rhwp-${Date.now().toString(36)}-${Math.random().toString(36).slice(2)}`;
}

export function createRealtimeOperationFromCommand(
  command: EditCommand,
  options: CreateRealtimeOperationOptions,
): RhwpRealtimeOperation | null {
  const raw = command as CommandInternals;
  const position = raw.position ?? raw.start ?? raw.cursorBefore ?? commandAnchorPosition(raw);
  if (!position) return null;

  const base = {
    contract: 'rhwp-realtime-op/v1' as const,
    opId: `${options.originId}:${options.sequence}`,
    originId: options.originId,
    sequence: options.sequence,
    position: clonePosition(position),
    timestamp: command.timestamp,
  };

  if (command.type === 'insertText' && typeof raw.text === 'string') {
    return {
      ...base,
      kind: 'insertText',
      text: raw.text,
    };
  }

  if (command.type === 'insertLineBreak' || command.type === 'insertTab') {
    return {
      ...base,
      kind: 'insertText',
      text: command.type === 'insertLineBreak' ? '\n' : '\t',
    };
  }

  if (
    command.type === 'deleteText'
    && typeof raw.count === 'number'
    && (raw.direction === 'forward' || raw.direction === 'backward')
  ) {
    return {
      ...base,
      kind: 'deleteText',
      count: raw.count,
      direction: raw.direction,
      deletedText: raw.deletedText,
    };
  }

  if (command.type === 'splitParagraph' && raw.position && isBodyPosition(raw.position)) {
    return {
      ...base,
      kind: 'splitParagraph',
      position: clonePosition(raw.position),
    };
  }

  if (
    (command.type === 'mergeParagraph' || command.type === 'mergeNextParagraph')
    && raw.position
    && isBodyPosition(raw.position)
  ) {
    return {
      ...base,
      kind: command.type,
      position: clonePosition(raw.position),
      mergePointOffset: raw.mergePointOffset ?? raw.position.charOffset,
    };
  }

  if (command.type === 'splitParagraphInCell' && raw.position && isCellPosition(raw.position)) {
    return {
      ...base,
      kind: 'splitParagraphInCell',
      position: clonePosition(raw.position),
    };
  }

  if (
    (command.type === 'mergeParagraphInCell' || command.type === 'mergeNextParagraphInCell')
    && raw.position
    && isCellPosition(raw.position)
  ) {
    return {
      ...base,
      kind: command.type,
      position: clonePosition(raw.position),
      mergePointOffset: raw.mergePointOffset ?? raw.position.charOffset,
    };
  }

  if (
    command.type === 'applyCharFormat'
    && raw.start
    && raw.end
    && raw.props
  ) {
    return {
      ...base,
      kind: 'applyCharFormat',
      position: clonePosition(raw.start),
      start: clonePosition(raw.start),
      end: clonePosition(raw.end),
      props: cloneProps(raw.props),
    };
  }

  if (
    command.type === 'applyParaFormat'
    && raw.cursorBefore
    && Array.isArray(raw.targets)
    && raw.props
  ) {
    return {
      ...base,
      kind: 'applyParaFormat',
      position: clonePosition(raw.cursorBefore),
      cursorBefore: clonePosition(raw.cursorBefore),
      targets: raw.targets.map(cloneParaFormatTarget),
      props: cloneProps(raw.props),
    };
  }

  if (
    command.type === 'moveTable'
    && isNumber(raw.sec)
    && isNumber(raw.ppi)
    && isNumber(raw.ci)
    && isNumber(raw.deltaH)
    && isNumber(raw.deltaV)
    && isNumber(raw.resultPpi)
    && isNumber(raw.resultCi)
  ) {
    return {
      ...base,
      kind: 'moveTable',
      position: { sectionIndex: raw.sec, paragraphIndex: raw.ppi, charOffset: 0 },
      sec: raw.sec,
      ppi: raw.ppi,
      ci: raw.ci,
      deltaH: raw.deltaH,
      deltaV: raw.deltaV,
      resultPpi: raw.resultPpi,
      resultCi: raw.resultCi,
    };
  }

  if (
    (command.type === 'movePicture' || command.type === 'moveShape')
    && isNumber(raw.sec)
    && isNumber(raw.ppi)
    && isNumber(raw.ci)
    && isNumber(raw.deltaH)
    && isNumber(raw.deltaV)
    && isNumber(raw.origHorzOffset)
    && isNumber(raw.origVertOffset)
  ) {
    return {
      ...base,
      kind: command.type,
      position: { sectionIndex: raw.sec, paragraphIndex: raw.ppi, charOffset: 0 },
      objectType: command.type === 'moveShape' ? 'shape' : 'image',
      sec: raw.sec,
      ppi: raw.ppi,
      ci: raw.ci,
      deltaH: raw.deltaH,
      deltaV: raw.deltaV,
      origHorzOffset: raw.origHorzOffset,
      origVertOffset: raw.origVertOffset,
      cellPath: cloneCellPath(raw.cellPath),
    };
  }

  if (command.type === 'resizeObject' && Array.isArray(raw.targets)) {
    const objectTargets = raw.targets
      .filter(isObjectResizeTargetLike)
      .map(cloneObjectTarget);
    if (objectTargets.length === 0) return null;
    const first = objectTargets[0];
    return {
      ...base,
      kind: 'resizeObject',
      position: { sectionIndex: first.sec, paragraphIndex: first.ppi, charOffset: 0 },
      objectTargets,
    };
  }

  return null;
}

export function createRealtimeOperationFromDraft(
  draft: RhwpRealtimeOperationDraft,
  options: CreateRealtimeOperationOptions,
): RhwpRealtimeOperation {
  return cloneOperation({
    ...draft,
    contract: 'rhwp-realtime-op/v1',
    opId: `${options.originId}:${options.sequence}`,
    originId: options.originId,
    sequence: options.sequence,
    position: clonePosition(draft.position),
    timestamp: draft.timestamp ?? Date.now(),
  });
}

export function isOwnRealtimeOperation(op: RhwpRealtimeOperation, originId: string): boolean {
  return op.originId === originId;
}

export function transformCursorAfterRemoteOperation(
  cursor: DocumentPosition,
  op: RhwpRealtimeOperation,
): DocumentPosition {
  if (op.kind === 'applyCharFormat' || op.kind === 'applyParaFormat') {
    return clonePosition(cursor);
  }
  if (op.kind === 'splitParagraph') {
    return transformPositionAgainstSplit(cursor, op.position);
  }
  if (op.kind === 'mergeParagraph' || op.kind === 'mergeNextParagraph') {
    return transformPositionAgainstMerge(cursor, op);
  }
  if (op.kind === 'splitParagraphInCell') {
    return transformPositionAgainstCellSplit(cursor, op.position);
  }
  if (op.kind === 'mergeParagraphInCell' || op.kind === 'mergeNextParagraphInCell') {
    return transformPositionAgainstCellMerge(cursor, op);
  }
  if (!sameTextContainer(cursor, op.position)) return clonePosition(cursor);
  const offset = cursor.charOffset;
  const opOffset = op.position.charOffset;

  if (op.kind === 'insertText' || isStructuralInsertKind(op.kind)) {
    const insertLength = getInsertedControlLength(op);
    if (insertLength > 0 && opOffset <= offset) {
      return { ...clonePosition(cursor), charOffset: offset + insertLength };
    }
    return clonePosition(cursor);
  }

  if (op.kind === 'deleteTextSpans') {
    return sortedDeleteSpansDescending(op.spans ?? []).reduce(
      (nextCursor, span) => transformCursorAfterDelete(nextCursor, span.position, span.count),
      clonePosition(cursor),
    );
  }

  const count = op.count ?? 0;
  return transformCursorAfterDelete(cursor, op.position, count);
}

export function transformRemoteOperationAgainstLocalHistory(
  remoteOp: RhwpRealtimeOperation,
  localHistory: readonly RhwpRealtimeOperation[],
): RhwpRealtimeOperation {
  let transformed = cloneOperation(remoteOp);
  for (const localOp of localHistory) {
    if (localOp.opId === transformed.opId) continue;
    if (!canOperationTransformAffect(transformed, localOp)) continue;
    transformed = transformRemoteOperationAgainstLocal(transformed, localOp);
  }
  return transformed;
}

function transformRemoteOperationAgainstLocal(
  remoteOp: RhwpRealtimeOperation,
  localOp: RhwpRealtimeOperation,
): RhwpRealtimeOperation {
  if (localOp.kind === 'splitParagraph') {
    return transformRemoteOperationAgainstLocalSplit(remoteOp, localOp.position);
  }

  if (localOp.kind === 'mergeParagraph' || localOp.kind === 'mergeNextParagraph') {
    return transformRemoteOperationAgainstLocalMerge(remoteOp, localOp);
  }

  if (localOp.kind === 'splitParagraphInCell') {
    return transformRemoteOperationAgainstLocalCellSplit(remoteOp, localOp.position);
  }

  if (localOp.kind === 'mergeParagraphInCell' || localOp.kind === 'mergeNextParagraphInCell') {
    return transformRemoteOperationAgainstLocalCellMerge(remoteOp, localOp);
  }

  if (localOp.kind === 'insertText' || isStructuralInsertKind(localOp.kind)) {
    return transformRemoteOperationAgainstLocalInsert(remoteOp, localOp);
  }

  if (localOp.kind === 'deleteTextSpans') {
    return (localOp.spans ?? []).reduce(
      (next, span) => transformRemoteOperationAgainstLocalDeleteSpan(next, span.position, span.count),
      remoteOp,
    );
  }
  const localCount = localOp.count ?? 0;
  if (localCount <= 0) return remoteOp;
  return transformRemoteOperationAgainstLocalDeleteSpan(remoteOp, localOp.position, localCount);
}

function transformRemoteOperationAgainstLocalInsert(
  remoteOp: RhwpRealtimeOperation,
  localOp: RhwpRealtimeOperation,
): RhwpRealtimeOperation {
  const localLength = getInsertedControlLength(localOp);
  if (localLength <= 0) return remoteOp;
  if (!sameTextContainer(remoteOp.position, localOp.position)) return remoteOp;

  if (remoteOp.kind === 'deleteText' || remoteOp.kind === 'deleteTextSpans') {
    const spans = operationToDeleteSpans(remoteOp).flatMap((span) =>
      transformDeleteSpanAgainstInsert(span, localOp.position, localLength),
    );
    return deleteSpansToOperation(remoteOp, spans);
  }

  if (remoteOp.kind === 'applyCharFormat') {
    return transformCharFormatAgainstInsert(remoteOp, localOp.position, localLength);
  }

  const localOffset = localOp.position.charOffset;
  const remoteOffset = remoteOp.position.charOffset;
  if (
    localOffset < remoteOffset
    || (localOffset === remoteOffset && shouldRemoteComeAfterLocal(remoteOp, localOp))
  ) {
    return shiftOperationPosition(remoteOp, localLength);
  }
  return remoteOp;
}

function isStructuralInsertKind(kind: RhwpRealtimeOperationKind): boolean {
  return kind === 'insertEquation'
    || kind === 'insertField'
    || kind === 'insertFootnote'
    || kind === 'insertEndnote';
}

function getInsertedControlLength(op: RhwpRealtimeOperation): number {
  if (op.kind === 'insertText') return op.text?.length ?? 0;
  if (isStructuralInsertKind(op.kind)) return 1;
  return 0;
}

function transformRemoteOperationAgainstLocalSplit(
  remoteOp: RhwpRealtimeOperation,
  splitPosition: DocumentPosition,
): RhwpRealtimeOperation {
  return mapOperationPositions(
    remoteOp,
    (position) => transformPositionAgainstSplit(position, splitPosition),
    (target) => transformParaTargetAgainstSplit(target, splitPosition),
  );
}

function transformRemoteOperationAgainstLocalMerge(
  remoteOp: RhwpRealtimeOperation,
  mergeOp: RhwpRealtimeOperation,
): RhwpRealtimeOperation {
  return mapOperationPositions(
    remoteOp,
    (position) => transformPositionAgainstMerge(position, mergeOp),
    (target) => transformParaTargetAgainstMerge(target, mergeOp),
  );
}

function transformRemoteOperationAgainstLocalCellSplit(
  remoteOp: RhwpRealtimeOperation,
  splitPosition: DocumentPosition,
): RhwpRealtimeOperation {
  return mapOperationPositions(
    remoteOp,
    (position) => transformPositionAgainstCellSplit(position, splitPosition),
    (target) => transformParaTargetAgainstCellSplit(target, splitPosition),
  );
}

function transformRemoteOperationAgainstLocalCellMerge(
  remoteOp: RhwpRealtimeOperation,
  mergeOp: RhwpRealtimeOperation,
): RhwpRealtimeOperation {
  return mapOperationPositions(
    remoteOp,
    (position) => transformPositionAgainstCellMerge(position, mergeOp),
    (target) => transformParaTargetAgainstCellMerge(target, mergeOp),
  );
}

function transformRemoteOperationAgainstLocalDeleteSpan(
  remoteOp: RhwpRealtimeOperation,
  localPosition: DocumentPosition,
  localCount: number,
): RhwpRealtimeOperation {
  if (localCount <= 0 || !sameTextContainer(remoteOp.position, localPosition)) return remoteOp;

  if (remoteOp.kind === 'deleteText' || remoteOp.kind === 'deleteTextSpans') {
    const spans = operationToDeleteSpans(remoteOp).flatMap((span) =>
      transformDeleteSpanAgainstDelete(span, localPosition, localCount),
    );
    return deleteSpansToOperation(remoteOp, spans);
  }

  if (remoteOp.kind === 'applyCharFormat') {
    return transformCharFormatAgainstDelete(remoteOp, localPosition, localCount);
  }

  const localStart = localPosition.charOffset;
  const localEnd = localStart + localCount;
  const remoteOffset = remoteOp.position.charOffset;
  if (remoteOffset >= localEnd) return shiftOperationPosition(remoteOp, -localCount);
  if (remoteOffset >= localStart) {
    return {
      ...remoteOp,
      position: { ...remoteOp.position, charOffset: localStart },
    };
  }
  return remoteOp;
}

function transformCharFormatAgainstInsert(
  op: RhwpRealtimeOperation,
  insertPosition: DocumentPosition,
  insertLength: number,
): RhwpRealtimeOperation {
  if (!op.start || !op.end || insertLength <= 0) return op;
  if (!sameTextContainer(op.start, insertPosition) || !sameTextContainer(op.end, insertPosition)) return op;

  const insertOffset = insertPosition.charOffset;
  let start = clonePosition(op.start);
  let end = clonePosition(op.end);
  if (insertOffset <= start.charOffset) {
    start = shiftPosition(start, insertLength);
    end = shiftPosition(end, insertLength);
  } else if (insertOffset < end.charOffset) {
    end = shiftPosition(end, insertLength);
  }
  return {
    ...op,
    position: clonePosition(start),
    start,
    end,
  };
}

function transformCharFormatAgainstDelete(
  op: RhwpRealtimeOperation,
  deletePosition: DocumentPosition,
  deleteCount: number,
): RhwpRealtimeOperation {
  if (!op.start || !op.end || deleteCount <= 0) return op;
  if (!sameTextContainer(op.start, deletePosition) || !sameTextContainer(op.end, deletePosition)) return op;

  const start = transformPositionAfterDelete(op.start, deletePosition, deleteCount);
  let end = transformPositionAfterDelete(op.end, deletePosition, deleteCount);
  if (end.charOffset < start.charOffset) end = { ...end, charOffset: start.charOffset };
  return {
    ...op,
    position: clonePosition(start),
    start,
    end,
  };
}

function mapOperationPositions(
  op: RhwpRealtimeOperation,
  mapPosition: (position: DocumentPosition) => DocumentPosition,
  mapTarget: (target: ParaFormatTarget) => ParaFormatTarget,
): RhwpRealtimeOperation {
  const mapped = {
    ...op,
    position: mapPosition(op.position),
    start: op.start ? mapPosition(op.start) : undefined,
    end: op.end ? mapPosition(op.end) : undefined,
    cursorBefore: op.cursorBefore ? mapPosition(op.cursorBefore) : undefined,
    spans: op.spans?.map((span) => ({ ...span, position: mapPosition(span.position) })),
    targets: op.targets?.map(mapTarget),
  };
  return mapOperationBodyAnchors(mapped, mapPosition);
}

function mapOperationBodyAnchors(
  op: RhwpRealtimeOperation,
  mapPosition: (position: DocumentPosition) => DocumentPosition,
): RhwpRealtimeOperation {
  let next = op;
  if (isNumber(op.sec) && isNumber(op.ppi)) {
    const mapped = mapPosition({ sectionIndex: op.sec, paragraphIndex: op.ppi, charOffset: 0 });
    next = { ...next, sec: mapped.sectionIndex, ppi: mapped.paragraphIndex };
  }
  if (isNumber(next.sec) && isNumber(op.resultPpi)) {
    const mapped = mapPosition({ sectionIndex: next.sec, paragraphIndex: op.resultPpi, charOffset: 0 });
    next = { ...next, resultPpi: mapped.paragraphIndex };
  }
  if (op.objectTargets) {
    next = {
      ...next,
      objectTargets: op.objectTargets.map((target) => {
        const mapped = mapPosition({ sectionIndex: target.sec, paragraphIndex: target.ppi, charOffset: 0 });
        return { ...cloneObjectTarget(target), sec: mapped.sectionIndex, ppi: mapped.paragraphIndex };
      }),
    };
  }
  return next;
}

function transformPositionAgainstSplit(
  position: DocumentPosition,
  splitPosition: DocumentPosition,
): DocumentPosition {
  if (!isSameBodySection(position, splitPosition)) return clonePosition(position);
  const splitPara = splitPosition.paragraphIndex;
  if (position.paragraphIndex < splitPara) return clonePosition(position);
  if (position.paragraphIndex > splitPara) {
    return { ...clonePosition(position), paragraphIndex: position.paragraphIndex + 1 };
  }
  if (position.charOffset > splitPosition.charOffset) {
    return {
      ...clonePosition(position),
      paragraphIndex: splitPara + 1,
      charOffset: position.charOffset - splitPosition.charOffset,
    };
  }
  return clonePosition(position);
}

function transformPositionAgainstMerge(
  position: DocumentPosition,
  mergeOp: RhwpRealtimeOperation,
): DocumentPosition {
  if (!isSameBodySection(position, mergeOp.position)) return clonePosition(position);
  const removedPara = mergeOp.kind === 'mergeNextParagraph'
    ? mergeOp.position.paragraphIndex + 1
    : mergeOp.position.paragraphIndex;
  const targetPara = mergeOp.kind === 'mergeNextParagraph'
    ? mergeOp.position.paragraphIndex
    : mergeOp.position.paragraphIndex - 1;
  const mergePointOffset = mergeOp.mergePointOffset ?? mergeOp.position.charOffset;

  if (position.paragraphIndex < removedPara) return clonePosition(position);
  if (position.paragraphIndex === removedPara) {
    return {
      ...clonePosition(position),
      paragraphIndex: Math.max(0, targetPara),
      charOffset: mergePointOffset + position.charOffset,
    };
  }
  return {
    ...clonePosition(position),
    paragraphIndex: Math.max(0, position.paragraphIndex - 1),
  };
}

function transformPositionAgainstCellSplit(
  position: DocumentPosition,
  splitPosition: DocumentPosition,
): DocumentPosition {
  if (!isSameCellContainer(position, splitPosition)) return clonePosition(position);
  const positionCellPara = position.cellParaIndex!;
  const splitCellPara = splitPosition.cellParaIndex!;

  if (positionCellPara < splitCellPara) return clonePosition(position);
  if (positionCellPara > splitCellPara) {
    return setCellParaIndex(position, positionCellPara + 1);
  }
  if (position.charOffset > splitPosition.charOffset) {
    return {
      ...setCellParaIndex(position, splitCellPara + 1),
      charOffset: position.charOffset - splitPosition.charOffset,
    };
  }
  return clonePosition(position);
}

function transformPositionAgainstCellMerge(
  position: DocumentPosition,
  mergeOp: RhwpRealtimeOperation,
): DocumentPosition {
  if (!isSameCellContainer(position, mergeOp.position)) return clonePosition(position);
  const mergeCellPara = mergeOp.position.cellParaIndex!;
  const removedCellPara = mergeOp.kind === 'mergeNextParagraphInCell'
    ? mergeCellPara + 1
    : mergeCellPara;
  const targetCellPara = mergeOp.kind === 'mergeNextParagraphInCell'
    ? mergeCellPara
    : mergeCellPara - 1;
  const mergePointOffset = mergeOp.mergePointOffset ?? (
    mergeOp.kind === 'mergeNextParagraphInCell' ? mergeOp.position.charOffset : 0
  );
  const positionCellPara = position.cellParaIndex!;

  if (positionCellPara < removedCellPara) return clonePosition(position);
  if (positionCellPara === removedCellPara) {
    return {
      ...setCellParaIndex(position, Math.max(0, targetCellPara)),
      charOffset: mergePointOffset + position.charOffset,
    };
  }
  return setCellParaIndex(position, Math.max(0, positionCellPara - 1));
}

function transformParaTargetAgainstSplit(
  target: ParaFormatTarget,
  splitPosition: DocumentPosition,
): ParaFormatTarget {
  if (target.kind !== 'body' || target.sec !== splitPosition.sectionIndex) return cloneParaFormatTarget(target);
  if (target.para > splitPosition.paragraphIndex) {
    return { ...target, para: target.para + 1 };
  }
  return cloneParaFormatTarget(target);
}

function transformParaTargetAgainstMerge(
  target: ParaFormatTarget,
  mergeOp: RhwpRealtimeOperation,
): ParaFormatTarget {
  if (target.kind !== 'body' || target.sec !== mergeOp.position.sectionIndex) return cloneParaFormatTarget(target);
  const removedPara = mergeOp.kind === 'mergeNextParagraph'
    ? mergeOp.position.paragraphIndex + 1
    : mergeOp.position.paragraphIndex;
  const targetPara = mergeOp.kind === 'mergeNextParagraph'
    ? mergeOp.position.paragraphIndex
    : mergeOp.position.paragraphIndex - 1;
  if (target.para < removedPara) return cloneParaFormatTarget(target);
  if (target.para === removedPara) return { ...target, para: Math.max(0, targetPara) };
  return { ...target, para: Math.max(0, target.para - 1) };
}

function transformParaTargetAgainstCellSplit(
  target: ParaFormatTarget,
  splitPosition: DocumentPosition,
): ParaFormatTarget {
  if (!isSameCellTarget(target, splitPosition)) return cloneParaFormatTarget(target);
  if (target.kind === 'cell' && target.cellParaIdx > splitPosition.cellParaIndex!) {
    return { ...target, cellParaIdx: target.cellParaIdx + 1 };
  }
  return cloneParaFormatTarget(target);
}

function transformParaTargetAgainstCellMerge(
  target: ParaFormatTarget,
  mergeOp: RhwpRealtimeOperation,
): ParaFormatTarget {
  if (!isSameCellTarget(target, mergeOp.position)) return cloneParaFormatTarget(target);
  const mergeCellPara = mergeOp.position.cellParaIndex!;
  const removedCellPara = mergeOp.kind === 'mergeNextParagraphInCell'
    ? mergeCellPara + 1
    : mergeCellPara;
  const targetCellPara = mergeOp.kind === 'mergeNextParagraphInCell'
    ? mergeCellPara
    : mergeCellPara - 1;
  if (target.kind !== 'cell' || target.cellParaIdx < removedCellPara) return cloneParaFormatTarget(target);
  if (target.cellParaIdx === removedCellPara) {
    return { ...target, cellParaIdx: Math.max(0, targetCellPara) };
  }
  return { ...target, cellParaIdx: Math.max(0, target.cellParaIdx - 1) };
}

function transformDeleteSpanAgainstInsert(
  span: RhwpRealtimeDeleteSpan,
  insertPosition: DocumentPosition,
  insertLength: number,
): RhwpRealtimeDeleteSpan[] {
  const start = span.position.charOffset;
  const end = start + span.count;
  const insertOffset = insertPosition.charOffset;

  if (insertOffset <= start) {
    return [shiftDeleteSpan(span, insertLength)];
  }
  if (insertOffset >= end) {
    return [cloneDeleteSpan(span)];
  }

  return [
    { position: clonePosition(span.position), count: insertOffset - start },
    {
      position: { ...clonePosition(span.position), charOffset: insertOffset + insertLength },
      count: end - insertOffset,
    },
  ].filter((next) => next.count > 0);
}

function transformDeleteSpanAgainstDelete(
  span: RhwpRealtimeDeleteSpan,
  deletePosition: DocumentPosition,
  deleteCount: number,
): RhwpRealtimeDeleteSpan[] {
  const remoteStart = span.position.charOffset;
  const remoteEnd = remoteStart + span.count;
  const localStart = deletePosition.charOffset;
  const localEnd = localStart + deleteCount;

  if (localEnd <= remoteStart) return [shiftDeleteSpan(span, -deleteCount)];
  if (localStart >= remoteEnd) return [cloneDeleteSpan(span)];

  const overlapStart = Math.max(remoteStart, localStart);
  const overlapEnd = Math.min(remoteEnd, localEnd);
  const remainingCount = span.count - Math.max(0, overlapEnd - overlapStart);
  if (remainingCount <= 0) return [];

  return [{
    position: {
      ...clonePosition(span.position),
      charOffset: remoteStart < localStart ? remoteStart : localStart,
    },
    count: remainingCount,
    deletedText: span.deletedText,
  }];
}

function operationToDeleteSpans(op: RhwpRealtimeOperation): RhwpRealtimeDeleteSpan[] {
  if (op.kind === 'deleteTextSpans') {
    return (op.spans ?? []).filter((span) => span.count > 0).map(cloneDeleteSpan);
  }
  const count = op.count ?? 0;
  if (count <= 0) return [];
  return [{ position: clonePosition(op.position), count, deletedText: op.deletedText }];
}

function deleteSpansToOperation(
  base: RhwpRealtimeOperation,
  spans: RhwpRealtimeDeleteSpan[],
): RhwpRealtimeOperation {
  const normalized = spans.filter((span) => span.count > 0).map(cloneDeleteSpan);
  if (normalized.length === 1) {
    const [span] = normalized;
    return {
      ...base,
      kind: 'deleteText',
      position: span.position,
      count: span.count,
      direction: base.direction ?? 'forward',
      deletedText: span.deletedText,
      spans: undefined,
    };
  }
  return {
    ...base,
    kind: 'deleteTextSpans',
    position: normalized[0]?.position ?? clonePosition(base.position),
    count: normalized.reduce((sum, span) => sum + span.count, 0),
    spans: normalized,
  };
}

function sortedDeleteSpansDescending(spans: readonly RhwpRealtimeDeleteSpan[]): RhwpRealtimeDeleteSpan[] {
  return spans
    .filter((span) => span.count > 0)
    .map(cloneDeleteSpan)
    .sort((a, b) => b.position.charOffset - a.position.charOffset);
}

function shiftDeleteSpan(span: RhwpRealtimeDeleteSpan, delta: number): RhwpRealtimeDeleteSpan {
  return {
    ...cloneDeleteSpan(span),
    position: {
      ...clonePosition(span.position),
      charOffset: Math.max(0, span.position.charOffset + delta),
    },
  };
}

function cloneDeleteSpan(span: RhwpRealtimeDeleteSpan): RhwpRealtimeDeleteSpan {
  return {
    ...span,
    position: clonePosition(span.position),
  };
}

function transformCursorAfterDelete(
  cursor: DocumentPosition,
  position: DocumentPosition,
  count: number,
): DocumentPosition {
  if (count <= 0 || !sameTextContainer(cursor, position)) return clonePosition(cursor);
  const offset = cursor.charOffset;
  const opOffset = position.charOffset;
  const deleteEnd = opOffset + count;
  if (offset > deleteEnd) {
    return { ...clonePosition(cursor), charOffset: offset - count };
  }
  if (offset >= opOffset) {
    return { ...clonePosition(cursor), charOffset: opOffset };
  }
  return clonePosition(cursor);
}

function transformPositionAfterDelete(
  position: DocumentPosition,
  deletePosition: DocumentPosition,
  count: number,
): DocumentPosition {
  if (count <= 0 || !sameTextContainer(position, deletePosition)) return clonePosition(position);
  const offset = position.charOffset;
  const deleteStart = deletePosition.charOffset;
  const deleteEnd = deleteStart + count;
  if (offset >= deleteEnd) return { ...clonePosition(position), charOffset: offset - count };
  if (offset >= deleteStart) return { ...clonePosition(position), charOffset: deleteStart };
  return clonePosition(position);
}

function shouldRemoteComeAfterLocal(remoteOp: RhwpRealtimeOperation, localOp: RhwpRealtimeOperation): boolean {
  return compareOperationOrder(localOp, remoteOp) < 0;
}

function compareOperationOrder(a: RhwpRealtimeOperation, b: RhwpRealtimeOperation): number {
  const origin = a.originId.localeCompare(b.originId);
  if (origin !== 0) return origin;
  if (a.sequence !== b.sequence) return a.sequence - b.sequence;
  return a.opId.localeCompare(b.opId);
}

function shiftOperationPosition(op: RhwpRealtimeOperation, delta: number): RhwpRealtimeOperation {
  if (delta === 0) return op;
  return {
    ...op,
    position: {
      ...op.position,
      charOffset: Math.max(0, op.position.charOffset + delta),
    },
  };
}

function shiftPosition(position: DocumentPosition, delta: number): DocumentPosition {
  if (delta === 0) return clonePosition(position);
  return {
    ...clonePosition(position),
    charOffset: Math.max(0, position.charOffset + delta),
  };
}

function sameTextContainer(a: DocumentPosition, b: DocumentPosition): boolean {
  return a.sectionIndex === b.sectionIndex
    && a.paragraphIndex === b.paragraphIndex
    && a.parentParaIndex === b.parentParaIndex
    && a.controlIndex === b.controlIndex
    && a.cellIndex === b.cellIndex
    && a.cellParaIndex === b.cellParaIndex
    && Boolean(a.isTextBox) === Boolean(b.isTextBox)
    && JSON.stringify(a.cellPath ?? []) === JSON.stringify(b.cellPath ?? []);
}

function canOperationTransformAffect(remoteOp: RhwpRealtimeOperation, localOp: RhwpRealtimeOperation): boolean {
  if (localOp.kind === 'splitParagraph' || localOp.kind === 'mergeParagraph' || localOp.kind === 'mergeNextParagraph') {
    return remoteOp.position.sectionIndex === localOp.position.sectionIndex
      && isBodyPosition(remoteOp.position)
      && isBodyPosition(localOp.position);
  }
  if (
    localOp.kind === 'splitParagraphInCell'
    || localOp.kind === 'mergeParagraphInCell'
    || localOp.kind === 'mergeNextParagraphInCell'
  ) {
    return isSameCellContainer(remoteOp.position, localOp.position);
  }
  return sameTextContainer(remoteOp.position, localOp.position);
}

function isBodyPosition(position: DocumentPosition): boolean {
  return position.parentParaIndex === undefined
    && position.controlIndex === undefined
    && position.cellIndex === undefined
    && position.cellParaIndex === undefined
    && !position.isTextBox
    && (position.cellPath?.length ?? 0) === 0;
}

function isSameBodySection(a: DocumentPosition, b: DocumentPosition): boolean {
  return a.sectionIndex === b.sectionIndex && isBodyPosition(a) && isBodyPosition(b);
}

function isCellPosition(position: DocumentPosition): boolean {
  return position.parentParaIndex !== undefined
    && position.controlIndex !== undefined
    && position.cellIndex !== undefined
    && position.cellParaIndex !== undefined
    && !position.isTextBox;
}

function isSameCellContainer(a: DocumentPosition, b: DocumentPosition): boolean {
  if (!isCellPosition(a) || !isCellPosition(b)) return false;
  if (
    a.sectionIndex !== b.sectionIndex
    || a.parentParaIndex !== b.parentParaIndex
    || a.controlIndex !== b.controlIndex
    || a.cellIndex !== b.cellIndex
    || Boolean(a.isTextBox) !== Boolean(b.isTextBox)
  ) {
    return false;
  }
  return JSON.stringify(cellPathWithoutLeafPara(a.cellPath)) === JSON.stringify(cellPathWithoutLeafPara(b.cellPath));
}

function isSameCellTarget(target: ParaFormatTarget, position: DocumentPosition): boolean {
  return target.kind === 'cell'
    && isCellPosition(position)
    && target.sec === position.sectionIndex
    && target.parentPara === position.parentParaIndex
    && target.controlIdx === position.controlIndex
    && target.cellIdx === position.cellIndex;
}

function setCellParaIndex(position: DocumentPosition, cellParaIndex: number): DocumentPosition {
  const next = {
    ...clonePosition(position),
    paragraphIndex: cellParaIndex,
    cellParaIndex,
  };
  if (next.cellPath && next.cellPath.length > 0) {
    next.cellPath = next.cellPath.map((entry, index) => (
      index === next.cellPath!.length - 1 ? { ...entry, cellParaIndex } : { ...entry }
    ));
  }
  return next;
}

function cellPathWithoutLeafPara(cellPath: DocumentPosition['cellPath']): Array<Record<string, number>> {
  if (!cellPath || cellPath.length === 0) return [];
  return cellPath.map((entry, index) => {
    if (index !== cellPath.length - 1) return { ...entry };
    const { cellParaIndex: _cellParaIndex, ...rest } = entry;
    return rest;
  });
}

function commandAnchorPosition(raw: CommandInternals): DocumentPosition | undefined {
  if (isNumber(raw.sec) && isNumber(raw.ppi)) {
    return { sectionIndex: raw.sec, paragraphIndex: raw.ppi, charOffset: 0 };
  }
  const firstTarget = raw.targets?.find(isObjectResizeTargetLike);
  if (firstTarget) {
    return { sectionIndex: firstTarget.sec, paragraphIndex: firstTarget.ppi, charOffset: 0 };
  }
  return undefined;
}

function isNumber(value: unknown): value is number {
  return typeof value === 'number' && Number.isFinite(value);
}

function cloneCellPath<T extends CellPathLike | undefined>(cellPath: T): T {
  if (!cellPath) return undefined as T;
  return cellPath.map((entry) => ({ ...entry })) as T;
}

function isObjectResizeTargetLike(value: unknown): value is RhwpRealtimeObjectTarget {
  if (!value || typeof value !== 'object') return false;
  const target = value as Partial<RhwpRealtimeObjectTarget>;
  return isNumber(target.sec)
    && isNumber(target.ppi)
    && isNumber(target.ci)
    && typeof target.type === 'string'
    && target.before !== undefined
    && target.after !== undefined;
}

function cloneObjectTarget(target: RhwpRealtimeObjectTarget): RhwpRealtimeObjectTarget {
  const cloned: RhwpRealtimeObjectTarget = {
    sec: target.sec,
    ppi: target.ppi,
    ci: target.ci,
    type: target.type,
    before: cloneProps(target.before),
    after: cloneProps(target.after),
  };
  const cellPath = cloneCellPath(target.cellPath);
  if (cellPath) cloned.cellPath = cellPath;
  return cloned;
}

function clonePosition(pos: DocumentPosition): DocumentPosition {
  const cloned: DocumentPosition = { ...pos };
  if (pos.cellPath) cloned.cellPath = pos.cellPath.map((entry) => ({ ...entry }));
  if (pos.cursorRect) cloned.cursorRect = { ...pos.cursorRect };
  return cloned;
}

function cloneProps<T extends object>(props: T): T {
  return JSON.parse(JSON.stringify(props)) as T;
}

function cloneParaFormatTarget(target: ParaFormatTarget): ParaFormatTarget {
  return { ...target };
}

function cloneOperation(op: RhwpRealtimeOperation): RhwpRealtimeOperation {
  return {
    ...op,
    position: clonePosition(op.position),
    start: op.start ? clonePosition(op.start) : undefined,
    end: op.end ? clonePosition(op.end) : undefined,
    cursorBefore: op.cursorBefore ? clonePosition(op.cursorBefore) : undefined,
    props: op.props ? cloneProps(op.props) : undefined,
    targets: op.targets?.map(cloneParaFormatTarget),
    spans: op.spans?.map(cloneDeleteSpan),
    mergePointOffset: op.mergePointOffset,
    cellPath: cloneCellPath(op.cellPath),
    objectTargets: op.objectTargets?.map(cloneObjectTarget),
    tableOptions: op.tableOptions ? cloneProps(op.tableOptions) : undefined,
    tableProps: op.tableProps ? cloneProps(op.tableProps) : undefined,
    cellProps: op.cellProps ? cloneProps(op.cellProps) : undefined,
    cellUpdates: op.cellUpdates?.map((update) => cloneProps(update)),
  };
}
