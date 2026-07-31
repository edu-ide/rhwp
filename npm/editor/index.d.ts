/**
 * @rhwp/editor — HWP 에디터 웹 컴포넌트
 */

export interface EditorOptions {
  /** rhwp-studio URL (기본: https://edwardkim.github.io/rhwp/) */
  studioUrl?: string;
  /** iframe 너비 (기본: '100%') */
  width?: string;
  /** iframe 높이 (기본: '100%') */
  height?: string;
}

export interface LoadResult {
  pageCount: number;
}

export interface HwpVerifyResult {
  /** 직렬화된 HWP 바이트 수 */
  bytesLen: number;
  /** 직렬화 직전 페이지 수 */
  pageCountBefore: number;
  /** 자기 재로드 후 페이지 수 (recovered === true 일 때 의미 있음) */
  pageCountAfter: number;
  /** 자기 재로드 성공 여부 */
  recovered: boolean;
}

export interface RhwpHistoryResult {
  ok: boolean;
  canUndo: boolean;
  canRedo: boolean;
}

export interface RhwpDispatchCommandResult {
  ok: boolean;
  commandId: string;
  known?: boolean;
  canUndo?: boolean;
  canRedo?: boolean;
}

export interface RhwpCommandPaletteResult {
  ok: boolean;
}

export interface RhwpDocumentPosition {
  sectionIndex: number;
  paragraphIndex: number;
  charOffset: number;
  parentParaIndex?: number;
  controlIndex?: number;
  cellIndex?: number;
  cellParaIndex?: number;
  isTextBox?: boolean;
  cellPath?: Array<Record<string, unknown>>;
}

export interface RhwpRealtimeDeleteSpan {
  position: RhwpDocumentPosition;
  count: number;
  deletedText?: string;
}

export interface RhwpRealtimeObjectTarget {
  sec: number;
  ppi: number;
  ci: number;
  type: string;
  cellPath?: Array<Record<string, unknown>>;
  before: Record<string, unknown>;
  after: Record<string, unknown>;
}

export interface RhwpRealtimeOperation {
  contract: 'rhwp-realtime-op/v1';
  opId: string;
  originId: string;
  sequence: number;
  kind:
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
  position: RhwpDocumentPosition;
  text?: string;
  count?: number;
  direction?: 'forward' | 'backward';
  deletedText?: string;
  spans?: RhwpRealtimeDeleteSpan[];
  start?: RhwpDocumentPosition;
  end?: RhwpDocumentPosition;
  props?: Record<string, unknown>;
  targets?: Array<
    | { kind: 'body'; sec: number; para: number }
    | { kind: 'cell'; sec: number; parentPara: number; controlIdx: number; cellIdx: number; cellParaIdx: number }
  >;
  cursorBefore?: RhwpDocumentPosition;
  mergePointOffset?: number;
  sec?: number;
  ppi?: number;
  ci?: number;
  deltaH?: number;
  deltaV?: number;
  resultPpi?: number;
  resultCi?: number;
  objectType?: 'image' | 'shape';
  origHorzOffset?: number;
  origVertOffset?: number;
  cellPath?: Array<Record<string, unknown>>;
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
  tableOptions?: {
    treatAsChar?: boolean;
    colWidths?: number[];
    rowHeights?: number[];
  };
  cellIndex?: number;
  tableProps?: Record<string, unknown>;
  cellProps?: Record<string, unknown>;
  cellUpdates?: Array<{
    cellIdx: number;
    widthDelta?: number;
    heightDelta?: number;
    renderWidth?: number;
    renderHeight?: number;
    localResize?: boolean;
  }>;
  timestamp: number;
}

export interface RhwpApplyOperationResult {
  ok: boolean;
  ignored?: boolean;
  error?: string;
  cursorPosition?: RhwpDocumentPosition;
  operation?: RhwpRealtimeOperation;
}

export interface RhwpCharPropertiesResult {
  sectionIndex: number;
  paragraphIndex: number;
  charOffset: number;
  properties: Record<string, unknown>;
}

export interface RhwpParagraphInfoResult {
  sectionIndex: number;
  paragraphIndex: number;
  paragraphCount: number;
  paragraphLength: number;
}

export declare class RhwpEditor {
  /** HWP 파일을 로드합니다 */
  loadFile(data: ArrayBuffer | Uint8Array, fileName?: string): Promise<LoadResult>;
  /** 현재 문서의 페이지 수를 반환합니다 */
  pageCount(): Promise<number>;
  /** 실행 취소 가능한 변경이 있는지 반환합니다 */
  canUndo(): Promise<boolean>;
  /** 다시 실행 가능한 변경이 있는지 반환합니다 */
  canRedo(): Promise<boolean>;
  /** iframe 내부 에디터에서 실행 취소를 수행합니다 */
  undo(): Promise<RhwpHistoryResult>;
  /** iframe 내부 에디터에서 다시 실행을 수행합니다 */
  redo(): Promise<RhwpHistoryResult>;
  /** iframe 내부 rhwp-studio 커맨드를 실행합니다 */
  dispatchCommand(commandId: string, params?: Record<string, unknown>): Promise<RhwpDispatchCommandResult>;
  /** iframe 내부 커맨드 팔레트를 엽니다 */
  openCommandPalette(): Promise<RhwpCommandPaletteResult>;
  /** 특정 페이지를 SVG 문자열로 렌더링합니다 */
  getPageSvg(page?: number): Promise<string>;
  /** 현재 문서를 HWP 바이너리로 내보냅니다 */
  exportHwp(): Promise<Uint8Array>;
  /** 현재 문서를 HWPX(ZIP+XML) 바이너리로 내보냅니다 */
  exportHwpx(): Promise<Uint8Array>;
  /** HWP 직렬화 + 자기 재로드 검증 메타데이터 (#178) */
  exportHwpVerify(): Promise<HwpVerifyResult>;
  /** 본문 위치의 글자 속성을 읽습니다 */
  getCharProperties(options?: Partial<RhwpDocumentPosition>): Promise<RhwpCharPropertiesResult>;
  /** 본문 문단 수와 특정 문단 길이를 읽습니다 */
  getParagraphInfo(options?: Partial<RhwpDocumentPosition>): Promise<RhwpParagraphInfoResult>;
  /** RHWP realtime operation을 iframe 내부 문서에 적용합니다 */
  applyOperation(operation: RhwpRealtimeOperation): Promise<RhwpApplyOperationResult>;
  /** iframe 내부 이벤트를 구독합니다 */
  on(eventName: 'operation', handler: (operation: RhwpRealtimeOperation) => void): () => void;
  /** iframe 엘리먼트를 반환합니다 */
  readonly element: HTMLIFrameElement;
  /** 에디터를 제거합니다 */
  destroy(): void;
}

/**
 * HWP 에디터를 생성하여 지정된 컨테이너에 마운트합니다.
 *
 * @example
 * ```javascript
 * import { createEditor } from '@rhwp/editor';
 *
 * const editor = await createEditor('#container');
 * const resp = await fetch('document.hwp');
 * await editor.loadFile(await resp.arrayBuffer());
 * ```
 */
export declare function createEditor(
  container: string | HTMLElement,
  options?: EditorOptions,
): Promise<RhwpEditor>;
