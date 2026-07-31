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

const DEFAULT_STUDIO_URL = 'https://edwardkim.github.io/rhwp/';
const DEFAULT_REQUEST_TIMEOUT_MS = 10000;
const READY_PROBE_TIMEOUT_MS = 750;
const READY_RETRY_DELAY_MS = 250;
const READY_ATTEMPTS = 80;

let requestId = 0;

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

  const studioUrl = options.studioUrl || DEFAULT_STUDIO_URL;

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
  const editor = new RhwpEditor(iframe);
  await editor._waitReady();
  return editor;
}

/**
 * HWP 에디터 인스턴스
 *
 * iframe 내부의 rhwp-studio와 postMessage로 통신합니다.
 */
class RhwpEditor {
  constructor(iframe) {
    this._iframe = iframe;
    this._pending = new Map();
    this._eventHandlers = new Map();
    this._keydownHandler = null;

    // 응답 수신 리스너
    this._messageHandler = (e) => {
      if (e.source !== this._iframe.contentWindow) return;
      if (e.data?.type === 'rhwp-event' && e.data.event) {
        this._emit(e.data.event, e.data.operation ?? e.data.payload ?? e.data);
        return;
      }
      if (e.data?.type === 'rhwp-response' && e.data.id != null) {
        const resolver = this._pending.get(e.data.id);
        if (!resolver) return;
        this._pending.delete(e.data.id);
        if (e.data.error) {
          resolver.reject(new Error(e.data.error));
        } else {
          resolver.resolve(e.data.result);
        }
      }
    };
    window.addEventListener('message', this._messageHandler);
    this._installHostShortcutForwarding();
  }

  _installHostShortcutForwarding() {
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
    return new Promise((resolve, reject) => {
      const id = ++requestId;
      let timeout = null;
      this._pending.set(id, {
        resolve: (value) => {
          clearTimeout(timeout);
          resolve(value);
        },
        reject: (error) => {
          clearTimeout(timeout);
          reject(error);
        },
      });
      this._iframe.contentWindow.postMessage(
        { type: 'rhwp-request', id, method, params },
        '*'
      );
      const timeoutMs = options.timeoutMs ?? DEFAULT_REQUEST_TIMEOUT_MS;
      timeout = setTimeout(() => {
        if (this._pending.has(id)) {
          this._pending.delete(id);
          reject(new Error(`Request timeout: ${method}`));
        }
      }, timeoutMs);
    });
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
    const bytes = data instanceof ArrayBuffer ? Array.from(new Uint8Array(data)) : Array.from(data);
    return this._request('loadFile', { data: bytes, fileName }, options);
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
   * iframe 엘리먼트를 반환합니다.
   */
  get element() {
    return this._iframe;
  }

  /**
   * 에디터를 제거합니다.
   */
  destroy() {
    window.removeEventListener('message', this._messageHandler);
    window.removeEventListener('keydown', this._keydownHandler);
    this._iframe.remove();
    this._pending.clear();
    this._eventHandlers.clear();
  }
}
