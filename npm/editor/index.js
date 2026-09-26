/**
 * @rhwp/editor — HWP 에디터를 iframe으로 임베드
 *
 * 사용법:
 *   import { createEditor } from '@rhwp/editor';
 *   const editor = await createEditor('#container');
 *   await editor.loadFile(buffer, 'document.hwp');
 *
 * 본 제품은 한글과컴퓨터의 한글 문서 파일(.hwp) 공개 문서를 참고하여 개발하였습니다.
 */

const READY_PROBE_TIMEOUT_MS = 750;
const READY_RETRY_DELAY_MS = 250;
const READY_ATTEMPTS = 80;
import { EditorTransport } from './transport.js';
import {
  assertCapability,
  validateApplyTextCommand,
  validateBodyParagraphTarget,
  validateDocumentState,
  validateDocumentChangedEvent,
  validateFocusTargetResult,
  validateRevertTextCommand,
  validateSelectionContext,
  validateTextCommandReceipt,
} from './document-agent-contract.js';

const DEFAULT_STUDIO_URL = 'https://edwardkim.github.io/rhwp/';

const DEFAULT_SHORTCUTS = [
  [{ key: 'z', ctrl: true }, 'edit:undo'],
  [{ key: 'z', ctrl: true, shift: true }, 'edit:redo'],
  [{ key: 'y', ctrl: true }, 'edit:redo'],
  [{ key: 'a', ctrl: true }, 'edit:select-all'],
  [{ key: 'e', ctrl: true }, 'edit:delete'],
  [{ key: 'ㄷ', ctrl: true }, 'edit:delete'],
  [{ key: 'c', code: 'KeyC', alt: true }, 'edit:format-copy'],
  [{ key: 'ㅊ', alt: true }, 'edit:format-copy'],
  [{ key: 'n', alt: true }, 'file:new-doc'],
  [{ key: 'ㅜ', alt: true }, 'file:new-doc'],
  [{ key: 'o', ctrl: true }, 'file:open'],
  [{ key: 'ㅐ', ctrl: true }, 'file:open'],
  [{ key: 's', ctrl: true }, 'file:save'],
  [{ key: 's', ctrl: true, shift: true }, 'file:save-as'],
  [{ key: 'ㄴ', ctrl: true, shift: true }, 'file:save-as'],
  [{ key: 'p', ctrl: true }, 'file:print'],
  [{ key: 'b', ctrl: true }, 'format:bold'],
  [{ key: 'i', ctrl: true }, 'format:italic'],
  [{ key: 'u', ctrl: true }, 'format:underline'],
  [{ key: 'l', alt: true }, 'format:char-shape'],
  [{ key: 'ㄹ', alt: true }, 'format:char-shape'],
  [{ key: 't', alt: true }, 'format:para-shape'],
  [{ key: 'ㅅ', alt: true }, 'format:para-shape'],
  [{ key: 'f6' }, 'format:style-dialog'],
  [{ key: 'f7' }, 'file:page-setup'],
  [{ key: '=', ctrl: true }, 'view:zoom-in'],
  [{ key: '+', ctrl: true }, 'view:zoom-in'],
  [{ key: '-', ctrl: true }, 'view:zoom-out'],
  [{ key: '0', ctrl: true }, 'view:zoom-100'],
  [{ key: 'f', ctrl: true }, 'edit:find'],
  [{ key: 'f2', ctrl: true }, 'edit:find-replace'],
  [{ key: 'l', ctrl: true }, 'edit:find-again'],
  [{ key: 'v', alt: true, shift: true }, 'edit:compare-documents'],
  [{ key: 'h', ctrl: true, shift: true }, 'edit:document-history'],
  [{ key: 'g', alt: true }, 'edit:goto'],
  [{ key: 'ㅎ', alt: true }, 'edit:goto'],
  [{ key: 'f10', alt: true }, 'insert:symbols'],
  [{ key: 'enter', ctrl: true }, 'page:break'],
  [{ key: 'enter', ctrl: true, shift: true }, 'page:column-break'],
  [{ key: 'enter', ctrl: true, alt: true }, 'page:col-settings'],
  [{ key: 'a', alt: true, shift: true }, 'format:line-spacing-decrease'],
  [{ key: 'ㅁ', alt: true, shift: true }, 'format:line-spacing-decrease'],
  [{ key: 'z', alt: true, shift: true }, 'format:line-spacing-increase'],
  [{ key: 'ㅋ', alt: true, shift: true }, 'format:line-spacing-increase'],
  [{ key: 'e', alt: true, shift: true }, 'format:font-size-increase'],
  [{ key: 'ㄷ', alt: true, shift: true }, 'format:font-size-increase'],
  [{ key: 'r', alt: true, shift: true }, 'format:font-size-decrease'],
  [{ key: 'ㄱ', alt: true, shift: true }, 'format:font-size-decrease'],
  [{ key: ']', ctrl: true }, 'format:font-size-increase'],
  [{ key: '[', ctrl: true }, 'format:font-size-decrease'],
  [{ key: 'j', code: 'KeyJ', alt: true, shift: true }, 'format:char-ratio-decrease'],
  [{ key: 'ㅓ', alt: true, shift: true }, 'format:char-ratio-decrease'],
  [{ key: 'k', code: 'KeyK', alt: true, shift: true }, 'format:char-ratio-increase'],
  [{ key: 'ㅏ', alt: true, shift: true }, 'format:char-ratio-increase'],
  [{ key: 'n', code: 'KeyN', alt: true, shift: true }, 'format:char-spacing-decrease'],
  [{ key: 'ㅜ', alt: true, shift: true }, 'format:char-spacing-decrease'],
  [{ key: 'w', code: 'KeyW', alt: true, shift: true }, 'format:char-spacing-increase'],
  [{ key: 'ㅈ', alt: true, shift: true }, 'format:char-spacing-increase'],
  [{ key: 'l', ctrl: true, shift: true }, 'format:align-left'],
  [{ key: 'm', ctrl: true, shift: true }, 'format:align-justify'],
  [{ key: 'h', alt: true, shift: true }, 'format:align-right'],
  [{ key: 'ㅗ', alt: true, shift: true }, 'format:align-right'],
  [{ key: 'c', alt: true, shift: true }, 'format:align-center'],
  [{ key: 'ㅊ', alt: true, shift: true }, 'format:align-center'],
  [{ key: 'd', alt: true, shift: true }, 'format:align-distribute'],
  [{ key: 'ㅇ', alt: true, shift: true }, 'format:align-distribute'],
  [{ key: 'enter', alt: true }, 'table:insert-row-col'],
  [{ key: 'delete', alt: true }, 'table:delete-row-col'],
  [{ key: 's', ctrl: true, shift: true }, 'table:block-sum'],
  [{ key: 'a', ctrl: true, shift: true }, 'table:block-avg'],
  [{ key: 'p', ctrl: true, shift: true }, 'table:block-product'],
];

function matchShortcut(event) {
  const ctrlOrMeta = event.ctrlKey || event.metaKey;
  const eventKey = String(event.key || '').toLowerCase();
  const eventCode = String(event.code || '').toLowerCase();

  for (const [def, cmdId] of DEFAULT_SHORTCUTS) {
    if (def.ctrl && !ctrlOrMeta) continue;
    if (!def.ctrl && ctrlOrMeta) continue;
    if ((def.shift ?? false) !== event.shiftKey) continue;
    if ((def.alt ?? false) !== event.altKey) continue;
    if (eventKey === def.key) return cmdId;
    if (def.code && eventCode === def.code.toLowerCase()) return cmdId;
  }
  return null;
}

/**
 * HWP 에디터를 생성하여 지정된 컨테이너에 마운트합니다.
 *
 * @param container - CSS 셀렉터 또는 HTMLElement
 * @param options - 에디터 옵션
 * @returns RhwpEditor 인스턴스
 *
 * @example
 * ```javascript
 * const editor = await createEditor('#editor');
 * await editor.loadFile(hwpBuffer, 'sample.hwp');
 * console.log(await editor.pageCount());
 * ```
 */
export async function createEditor(container, options = {}) {
  const el = typeof container === 'string'
    ? document.querySelector(container)
    : container;

  if (!el) {
    throw new Error(`Container not found: ${container}`);
  }

  let studioUrl = options.studioUrl || DEFAULT_STUDIO_URL;
  if (options.renderer !== undefined) {
    if (!['auto', 'canvas2d', 'canvaskit'].includes(options.renderer)) {
      throw new TypeError(`Unsupported renderer: ${options.renderer}`);
    }
    const resolvedStudioUrl = new URL(studioUrl, document.baseURI);
    resolvedStudioUrl.searchParams.set('renderer', options.renderer);
    studioUrl = resolvedStudioUrl.href;
  }

  // iframe 생성
  const iframe = document.createElement('iframe');
  iframe.style.width = options.width || '100%';
  iframe.style.height = options.height || '100%';
  iframe.style.border = 'none';
  iframe.allow = 'clipboard-read; clipboard-write';

  // iframe 로드 대기 리스너를 navigation 시작 전에 설치한다.
  const iframeLoaded = new Promise((resolve) => {
    iframe.addEventListener('load', resolve, { once: true });
  });
  iframe.src = studioUrl;
  el.appendChild(iframe);
  await iframeLoaded;

  // WASM 초기화 대기 (ready 메서드로 확인)
  let transport;
  try {
    transport = new EditorTransport(iframe, studioUrl, {
      requestTimeoutMs: options.requestTimeoutMs,
      handshakeTimeoutMs: options.handshakeTimeoutMs,
    });
    await transport.connect();
    const editor = new RhwpEditor(iframe, transport);
    await editor._waitReady();
    return editor;
  } catch (error) {
    transport?.destroy();
    iframe.remove();
    throw error;
  }
}

/**
 * HWP 에디터 인스턴스
 *
 * iframe 내부의 rhwp-studio와 postMessage로 통신합니다.
 */
export class RhwpEditor {
  constructor(iframe, transport) {
    this._iframe = iframe;
    this._transport = transport;
    this._documentChangedListeners = new Set();
    this._offDocumentChanged = null;
    this._lastDocumentEpoch = null;
    this._lastDocumentChangeSeq = null;
    this._eventHandlers = new Map();
    this._keydownHandler = null;
    this._offOperation = null;
    this._installHostShortcutForwarding();
  }

  _installHostShortcutForwarding() {
    if (!globalThis.window) return;
    this._keydownHandler = (event) => {
      const target = event.target;
      const tagName = target?.tagName;
      if (target?.isContentEditable || tagName === 'INPUT' || tagName === 'TEXTAREA' || tagName === 'SELECT') {
        return;
      }

      if ((event.ctrlKey || event.metaKey) && !event.shiftKey && !event.altKey && event.key === '/') {
        event.preventDefault();
        event.stopPropagation();
        this.openCommandPalette().catch((error) => {
          console.warn('[rhwp-editor] command palette forwarding failed:', error);
        });
        return;
      }

      const cmdId = matchShortcut(event);
      if (!cmdId) return;

      event.preventDefault();
      event.stopPropagation();
      const request = cmdId === 'edit:undo'
        ? this.undo()
        : cmdId === 'edit:redo'
          ? this.redo()
          : this.dispatchCommand(cmdId);
      request.catch((error) => {
        console.warn('[rhwp-editor] shortcut forwarding failed:', error);
      });
    };
    window.addEventListener('keydown', this._keydownHandler);
  }

  /**
   * iframe에 요청을 보내고 응답을 기다립니다.
   * @internal
   */
  _request(method, params = {}, options = {}) {
    return this._transport.request(method, params, options);
  }

  /** WASM 초기화 완료 대기 @internal */
  async _waitReady() {
    for (let i = 0; i < READY_ATTEMPTS; i++) {
      try {
        const result = await this._request('ready', {}, { timeoutMs: READY_PROBE_TIMEOUT_MS });
        if (result) return;
      } catch {
        // 아직 준비 안 됨 — 재시도
      }
      await new Promise((r) => setTimeout(r, READY_RETRY_DELAY_MS));
    }
    throw new Error('Editor initialization timeout');
  }

  /**
   * HWP 파일을 로드합니다.
   *
   * @param data - HWP 파일의 ArrayBuffer 또는 Uint8Array
   * @param fileName - 파일 이름 (선택)
   * @param options - 로드 옵션 (선택)
   * @param options.skipUnsavedGuard - 미저장 변경 확인 없이 문서 교체
   * @param options.suppressDialogs - 로드 후 안내창(HWPX 검증, 로컬 글꼴 감지) 없이 열기.
   *   임베드 환경에서 안내창의 사용자 선택을 기다리느라 loadFile 응답이 지연/교착되는
   *   것을 방지한다. 검증 경고는 '그대로 열기'로 처리되고, 글꼴은 웹 대체 글꼴로 표시된다.
   * @returns { pageCount: number }
   *
   * @example
   * ```javascript
   * const resp = await fetch('document.hwp');
   * const buffer = await resp.arrayBuffer();
   * const result = await editor.loadFile(buffer, 'document.hwp');
   * console.log(`${result.pageCount}페이지`);
   * ```
   */
  async loadFile(data, fileName = 'document.hwp', options = {}) {
    const result = await this._request('loadFile', {
      data,
      fileName,
      skipUnsavedGuard: options.skipUnsavedGuard === true,
      suppressDialogs: options.suppressDialogs === undefined || options.suppressDialogs === true,
    }, options);
    this._lastDocumentEpoch = null;
    this._lastDocumentChangeSeq = null;
    return result;
  }

  /**
   * 현재 문서의 페이지 수를 반환합니다.
   * @returns 페이지 수
   */
  async pageCount() {
    return this._request('pageCount');
  }

  /**
   * 실행 취소 가능한 변경이 있는지 반환합니다.
   */
  async canUndo() {
    return this._request('canUndo');
  }

  /**
   * 다시 실행 가능한 변경이 있는지 반환합니다.
   */
  async canRedo() {
    return this._request('canRedo');
  }

  /**
   * iframe 내부 에디터에서 실행 취소를 수행합니다.
   */
  async undo() {
    return this._request('undo');
  }

  /**
   * iframe 내부 에디터에서 다시 실행을 수행합니다.
   */
  async redo() {
    return this._request('redo');
  }

  /**
   * iframe 내부 rhwp-studio 커맨드를 실행합니다.
   */
  async dispatchCommand(commandId, params = {}) {
    return this._request('dispatchCommand', { commandId, params });
  }

  /**
   * iframe 내부 커맨드 팔레트를 엽니다.
   */
  async openCommandPalette() {
    return this._request('openCommandPalette');
  }

  /**
   * 특정 페이지를 SVG 문자열로 렌더링합니다.
   * @param page - 0부터 시작하는 페이지 번호
   * @returns SVG 문자열
   */
  async getPageSvg(page = 0) {
    return this._request('getPageSvg', { page });
  }

  /**
   * 선택된 renderer와 페이지별 CanvasKit readiness 진단을 반환합니다.
   * @param page - 0부터 시작하는 페이지 번호
   */
  async getRendererDiagnostics(page = 0) {
    if (!Number.isSafeInteger(page) || page < 0) {
      throw new TypeError('page must be a non-negative safe integer');
    }
    if (!this._transport.supports('renderer-diagnostics-v1')) {
      throw new Error('Renderer diagnostics v1 is not supported by this Studio');
    }
    const result = await this._request('getRendererDiagnostics', { page });
    if (result?.schemaVersion !== 1 || result?.page?.index !== page) {
      throw new Error('Studio returned invalid renderer diagnostics v1');
    }
    return result;
  }

  /** 현재 문자의 layout/paint font 결정 계보를 bounded trace로 반환합니다. */
  async getFontDecisionTrace(page = 0, options = {}) {
    if (!Number.isSafeInteger(page) || page < 0) {
      throw new TypeError('page must be a non-negative safe integer');
    }
    const maxCharacters = options.maxCharacters ?? 1024;
    if (!Number.isSafeInteger(maxCharacters) || maxCharacters < 1 || maxCharacters > 4096) {
      throw new TypeError('maxCharacters must be a safe integer in 1..=4096');
    }
    if (!this._transport.supports('font-decision-trace-v1')) {
      throw new Error('Font decision trace v1 is not supported by this Studio');
    }
    const result = await this._request('getFontDecisionTrace', {
      page,
      limits: { maxCharacters },
    });
    if (result?.schemaVersion !== 1 || result?.scope?.pageIndex !== page
      || !Array.isArray(result?.records)) {
      throw new Error('Studio returned invalid font decision trace v1');
    }
    return result;
  }

  /**
   * 현재 문서를 HWP 바이너리로 내보냅니다.
   * @returns {Promise<Uint8Array>} HWP 파일 bytes
   */
  async exportHwp() {
    const result = await this._request('exportHwp');
    return result instanceof Uint8Array ? result : new Uint8Array(result || []);
  }

  /**
   * 현재 문서를 HWPX(ZIP+XML) 바이너리로 내보냅니다.
   * @returns {Promise<Uint8Array>} HWPX 파일 bytes
   */
  async exportHwpx() {
    const result = await this._request('exportHwpx');
    return result instanceof Uint8Array ? result : new Uint8Array(result || []);
  }

  /** 현재 문서를 HML(XML) 바이너리로 내보냅니다. */
  async exportHml() {
    const result = await this._request('exportHml');
    return result instanceof Uint8Array ? result : new Uint8Array(result || []);
  }

  /** 현재 문서의 HML 저장 가능 여부와 blocker를 반환합니다. */
  async getHmlSaveState() {
    return this._request('getHmlSaveState');
  }

  /**
   * HWP 직렬화 + 자기 재로드 검증 메타데이터를 반환합니다 (#178).
   *
   * 검증 메타데이터만 반환하며, 실제 HWP bytes 가 필요하면 `exportHwp()` 를 별도 호출하세요.
   *
   * @returns {Promise<{bytesLen: number, pageCountBefore: number, pageCountAfter: number, recovered: boolean}>}
   */
  async exportHwpVerify() {
    return this._request('exportHwpVerify');
  }

  /**
   * 본문 문단의 텍스트 범위를 읽습니다.
   * @param options - { sectionIndex, paragraphIndex, charOffset, count }
   * @returns {Promise<{ text: string, sectionIndex: number, paragraphIndex: number, charOffset: number, count: number }>}
   */
  async getTextRange(options = {}) {
    return this._request('getTextRange', options);
  }

  /**
   * 본문 위치의 글자 속성을 읽습니다.
   * @param options - { sectionIndex, paragraphIndex, charOffset }
   * @returns {Promise<{ sectionIndex: number, paragraphIndex: number, charOffset: number, properties: object }>}
   */
  async getCharProperties(options = {}) {
    return this._request('getCharProperties', options);
  }

  /**
   * 본문 문단 수와 특정 문단 길이를 읽습니다.
   * @param options - { sectionIndex, paragraphIndex }
   * @returns {Promise<{ sectionIndex: number, paragraphIndex: number, paragraphCount: number, paragraphLength: number }>}
   */
  async getParagraphInfo(options = {}) {
    return this._request('getParagraphInfo', options);
  }

  /**
   * RHWP realtime operation을 iframe 내부 문서에 적용합니다.
   * @param operation - rhwp-realtime-op/v1 operation
   * @returns {Promise<{ ok: boolean, ignored?: boolean, error?: string }>}
   */
  async applyOperation(operation) {
    return this._request('applyOperation', { operation });
  }

  /**
   * iframe 내부 이벤트를 구독합니다. 현재 operation 이벤트를 사용합니다.
   * @param eventName - 이벤트 이름
   * @param handler - 이벤트 payload 핸들러
   * @returns {() => void} 구독 해제 함수
   */
  on(eventName, handler) {
    if (eventName === 'operation' && !this._offOperation) {
      this._offOperation = this._transport.on('operation', payload => this._emit('operation', payload));
    }
    if (!this._eventHandlers.has(eventName)) {
      this._eventHandlers.set(eventName, new Set());
    }
    this._eventHandlers.get(eventName).add(handler);
    return () => {
      this._eventHandlers.get(eventName)?.delete(handler);
    };
  }

  _emit(eventName, payload) {
    this._eventHandlers.get(eventName)?.forEach((handler) => {
      try {
        handler(payload);
      } catch (error) {
        setTimeout(() => { throw error; }, 0);
      }
    });
  }

  /**
   * 내보내기 바이트의 영속화(업로드/핸드오프) 완료를 스튜디오에 통지합니다.
   *
   * dirty 상태를 해제하고 자동복구 draft의 IndexedDB 삭제 "완료"까지 기다린 뒤
   * resolve합니다 — resolve 이후 창을 닫아도 안전합니다. 업로드 실패 시에는
   * 호출하지 마세요(백업 draft가 보존되어야 합니다).
   *
   * 스튜디오가 `notify-saved-v1` capability를 광고하지 않으면(구버전 또는
   * legacy 폴백 연결) 요청을 보내지 않고 명시적으로 실패합니다.
   *
   * @param fileName - 호스트가 저장에 사용한 파일 이름 (선택)
   * @returns {Promise<{ ok: true, wasDirty: boolean }>}
   */
  async notifySaved(fileName) {
    if (!this._transport.supports('notify-saved-v1')) {
      throw new Error('notifySaved is not supported by this Studio');
    }
    const params = typeof fileName === 'string' && fileName.length > 0 ? { fileName } : {};
    return this._request('notifySaved', params);
  }

  /** 현재 문서의 에이전트 명령 fence와 SHA-256 상태를 반환합니다. */
  async getDocumentState() {
    assertCapability(this._transport, 'document-state-v1');
    const state = validateDocumentState(await this._request('getDocumentState'));
    this._rememberDocumentVersion(state.documentEpoch, state.changeSeq);
    return state;
  }

  /** 현재 캐럿/선택을 body paragraph exact target으로 정규화해 반환합니다. */
  async getSelectionContext() {
    assertCapability(this._transport, 'selection-context-v1');
    const selection = validateSelectionContext(await this._request('getSelectionContext'));
    this._rememberDocumentVersion(selection.documentEpoch, selection.changeSeq);
    return selection;
  }

  /** exact preimage fence를 검증하고 한 Studio 트랜잭션으로 문단 전체를 교체합니다. */
  async applyTextCommand(command) {
    assertCapability(this._transport, 'document-agent-command-v1');
    const validCommand = validateApplyTextCommand(command);
    const receipt = validateTextCommandReceipt(await this._request('applyTextCommand', {
      command: validCommand,
    }));
    this._rememberDocumentVersion(receipt.documentEpoch, receipt.afterChangeSeq);
    return receipt;
  }

  /** 가장 최근에 성공한 exact command를 한 Studio 트랜잭션으로 되돌립니다. */
  async revertTextCommand(command) {
    assertCapability(this._transport, 'document-agent-command-v1');
    const validCommand = validateRevertTextCommand(command);
    const receipt = validateTextCommandReceipt(await this._request('revertTextCommand', {
      command: validCommand,
    }));
    this._rememberDocumentVersion(receipt.documentEpoch, receipt.afterChangeSeq);
    return receipt;
  }

  /** exact body paragraph target으로 캐럿과 뷰포트를 이동합니다. */
  async focusTarget(target) {
    assertCapability(this._transport, 'target-navigation-v1');
    const validTarget = validateBodyParagraphTarget(target);
    return validateFocusTargetResult(await this._request('focusTarget', {
      target: validTarget,
    }));
  }

  /** agent apply/revert가 commit된 뒤 strict v1 변경 이벤트를 구독합니다. */
  onDocumentChanged(listener) {
    assertCapability(this._transport, 'document-change-events-v1');
    if (typeof listener !== 'function') throw new TypeError('listener must be a function');
    this._documentChangedListeners.add(listener);
    if (!this._offDocumentChanged) {
      this._offDocumentChanged = this._transport.on('documentChanged', (payload) => {
        let event;
        try {
          event = validateDocumentChangedEvent(payload);
        } catch {
          return;
        }
        if (!this._isFreshDocumentEvent(event.documentEpoch, event.changeSeq)) return;
        this._rememberDocumentVersion(event.documentEpoch, event.changeSeq);
        for (const subscriber of this._documentChangedListeners) {
          try { subscriber(event); } catch { /* 한 listener가 다른 listener를 막지 않는다. */ }
        }
      });
    }
    return () => {
      this._documentChangedListeners.delete(listener);
      if (this._documentChangedListeners.size === 0) {
        this._offDocumentChanged?.();
        this._offDocumentChanged = null;
      }
    };
  }

  _isFreshDocumentEvent(epoch, changeSeq) {
    if (this._lastDocumentEpoch === null) return true;
    if (epoch !== this._lastDocumentEpoch) return epoch > this._lastDocumentEpoch;
    return this._lastDocumentChangeSeq === null || changeSeq > this._lastDocumentChangeSeq;
  }

  _rememberDocumentVersion(epoch, changeSeq) {
    if (this._lastDocumentEpoch === null || epoch > this._lastDocumentEpoch) {
      this._lastDocumentEpoch = epoch;
      this._lastDocumentChangeSeq = changeSeq;
      return;
    }
    if (epoch === this._lastDocumentEpoch
        && (this._lastDocumentChangeSeq === null || changeSeq > this._lastDocumentChangeSeq)) {
      this._lastDocumentChangeSeq = changeSeq;
    }
  }

  /**
   * iframe 엘리먼트를 반환합니다.
   */
  get element() {
    return this._iframe;
  }

  // ── 브리지 표면 (studio 자동화·플러그인·창 제어) ─────────────────

  /**
   * studio 의 커맨드·메뉴를 다룹니다.
   *
   * `list()` 는 레지스트리 전체이고 `menuModel()` 은 실제 메뉴 구조입니다 — 같지 않습니다.
   * 메뉴에 없는 커맨드(컨텍스트 메뉴·단축키 전용)도 실행할 수 있습니다.
   */
  get commands() {
    return {
      list: () => this._request('automation.list'),
      menuModel: () => this._request('automation.menuModel'),
      isEnabled: (id) => this._request('automation.isEnabled', { id }),
      /**
       * 커맨드 실행. 대화상자를 여는 커맨드는 기본 거절(`needs-dialog`)이다 —
       * 자동화가 그것을 열면 사람이 누를 때까지 응답이 멈춘다.
       * 사용자가 앞에 있는 통합에서는 `{ allowDialog: true }` 로 푼다.
       */
      execute: (id, params, options = {}) => this._request('automation.execute', {
        id, params, allowDialog: options.allowDialog === true,
      }),
      context: () => this._request('automation.context'),
    };
  }

  /** 플러그인을 올리고 내립니다. 올릴 수 있는 이름은 studio 가 정합니다(allowlist). */
  get plugins() {
    return {
      list: () => this._request('plugin.list'),
      load: (id) => this._request('plugin.load', { id }),
      unload: (id) => this._request('plugin.unload', { id }),
      invoke: (id, method, args = []) => this._request('plugin.invoke', { id, method, args }),
    };
  }

  /**
   * HwpCtrl API — studio 가 들고 있는 **그 문서**를 조작합니다.
   *
   * `hwpctrl` 플러그인이 올라와 있어야 합니다(`plugins.load('hwpctrl')`).
   */
  get hwpctrl() {
    const invoke = (method, args) =>
      this._request('plugin.invoke', { id: 'hwpctrl', method: 'invoke', args: [method, args] });
    return {
      call: invoke,
      /**
       * 여러 호출을 **한 메시지·한 트랜잭션**으로 보냅니다. undo 도 1스텝입니다.
       *
       * 콜백은 부모에서 실행되지 않습니다 — 호출을 기록해 배열로 직렬화한 뒤 한 번에 보냅니다.
       * 함수 문자열을 iframe 에서 평가하는 경로는 만들지 않습니다.
       */
      batch: (build) => {
        const ops = [];
        const recorder = new Proxy({}, {
          get: (_t, name) => (...args) => { ops.push({ m: String(name), a: args }); return recorder; },
        });
        if (typeof build === 'function') build(recorder);
        else if (Array.isArray(build)) ops.push(...build);
        return this._request('plugin.invoke', { id: 'hwpctrl', method: 'batch', args: [ops] });
      },
      exportBytes: (format) =>
        this._request('plugin.invoke', { id: 'hwpctrl', method: 'exportBytes', args: [format] }),
      undo: () => this._request('plugin.invoke', { id: 'hwpctrl', method: 'undo', args: [] }),
      redo: () => this._request('plugin.invoke', { id: 'hwpctrl', method: 'redo', args: [] }),
    };
  }

  /** 메뉴·툴바·상태표시줄 표시. 숨겨도 커맨드는 그대로 실행됩니다. */
  get chrome() {
    return {
      get: () => this._request('chrome.get'),
      set: (visibility) => this._request('chrome.set', { visibility }),
    };
  }

  /**
   * 에디터를 제거합니다.
   *
   * iframe 과 채널을 회수합니다. **다른 컨테이너로 옮기려면 destroy 후 다시 만들어야 합니다** —
   * iframe 을 DOM 이동시키면 브라우저가 문서를 재로드해 편집 상태가 사라집니다.
   */
  destroy() {
    this._offDocumentChanged?.();
    this._offDocumentChanged = null;
    this._documentChangedListeners.clear();
    this._transport.destroy();
    this._iframe.remove();
    this._offOperation?.();
    globalThis.window?.removeEventListener('keydown', this._keydownHandler);
    this._eventHandlers.clear();
  }
}

/**
 * studio 인스턴스를 만들어 컨테이너에 심습니다.
 *
 * `createEditor` 의 상위 집합입니다 — 같은 것을 만들고, 플러그인 로드와 창 제어 초기값을
 * 함께 처리합니다.
 *
 * @example
 * ```javascript
 * const studio = await createStudio('#app', { plugins: ['hwpctrl'], chrome: { menu: false } });
 * await studio.hwpctrl.batch(h => { h.SetTextFile('안녕', 'TEXT', ''); });
 * ```
 */
export async function createStudio(container, options = {}) {
  const studio = await createEditor(container, options);
  try {
    if (options.chrome) await studio.chrome.set(options.chrome);
    for (const id of options.plugins ?? []) await studio.plugins.load(id);
  } catch (error) {
    studio.destroy();
    throw error;
  }
  return studio;
}
