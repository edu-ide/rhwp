/**
 * 원문 근거 노트 오버레이 — "이 줄의 근거는 무엇인가"를 편집기 위에 그린다.
 *
 * 표시는 여기(스튜디오), 판정은 부모(데스크톱)가 한다. 스튜디오는 렌더 트리의
 * TextLine 좌표를 알고, 부모는 브레인의 근거 감사를 안다 — 각자 아는 것만
 * 맡는다. 흐름:
 *
 *   1. 부모가 `evidenceCollect` 를 부르면 전 페이지 렌더 트리에서 줄 텍스트를
 *      모아 전문을 돌려준다. 이때 줄별 문자 오프셋을 기억해 둔다.
 *   2. 부모가 그 전문으로 감사를 돌리고 `evidenceShow(notes)` 로 결과를 준다.
 *   3. 각 노트 문장을 전문에서 다시 찾아(감사기가 전문을 그대로 잘랐으므로
 *      indexOf 가 성립한다) 줄 → 페이지·좌표로 환산해 여백 배지를 그린다.
 *
 * 배지는 페이지 캔버스와 동일한 배치 규칙(top=페이지 오프셋, 단일 열이면 CSS
 * 중앙 정렬)을 따르는 페이지별 오버레이 층에 얹는다 — 캔버스가 가상 스크롤로
 * 재활용되어도 오버레이는 남는다.
 */
import type { WasmBridge } from '@/core/wasm-bridge';
import type { EventBus } from '@/core/event-bus';
import type { VirtualScroll } from '@/view/virtual-scroll';
import type { ViewportManager } from '@/view/viewport-manager';

export type EvidenceSource = {
  source_path: string;
  anchor: string;
  locator: string;
  text: string;
  score: number;
};

export type EvidenceAxisNote = {
  axis: string;
  verdict: '근거확보' | '근거불명' | '숫자불일치';
  unsupported_numbers: string[];
  sources: EvidenceSource[];
};

export type EvidenceNote = {
  sentence: string;
  verdict: '근거확보' | '근거불명' | '숫자불일치' | '양식문구';
  /** "작성"(작성자가 쓴 줄) | "양식"(양식 제공 문구) — 구버전 감사기는 생략. */
  origin?: '작성' | '양식';
  unsupported_numbers: string[];
  sources: EvidenceSource[];
  /** 다축 감사일 때 축별 대조 결과 (양식 축 등). */
  axes?: EvidenceAxisNote[];
};

type LineBox = { page: number; x: number; y: number; w: number; h: number; text: string; start: number };

const VERDICT_COLOR: Record<EvidenceNote['verdict'], string> = {
  숫자불일치: '#dc2626',
  근거불명: '#d97706',
  근거확보: '#059669',
  양식문구: '#64748b',
};

const VERDICT_BG: Record<EvidenceNote['verdict'], string> = {
  숫자불일치: 'rgba(220,38,38,0.10)',
  근거불명: 'rgba(217,119,6,0.10)',
  근거확보: 'rgba(5,150,105,0.08)',
  양식문구: 'rgba(100,116,139,0.08)',
};

export class EvidenceNotesOverlay {
  private lines: LineBox[] = [];
  private fullText = '';
  private layers: HTMLElement[] = [];
  private notes: EvidenceNote[] = [];
  private popover: HTMLElement | null = null;
  private unsubscribe: (() => void) | null = null;
  private visible = true;

  constructor(
    private scrollContent: HTMLElement,
    private wasm: WasmBridge,
    private virtualScroll: VirtualScroll,
    private viewportManager: ViewportManager,
    eventBus: EventBus,
  ) {
    const onZoom = () => this.relayout();
    eventBus.on('zoom-changed', onZoom);
    this.unsubscribe = () => (eventBus as any).off?.('zoom-changed', onZoom);
  }

  /** 전 페이지 렌더 트리에서 줄 텍스트·좌표를 모으고 감사용 전문을 돌려준다. */
  collect(): { text: string; lineCount: number; pageCount: number } {
    this.lines = [];
    const pageCount = this.wasm.pageCount;
    for (let p = 0; p < pageCount; p++) {
      const tree = this.wasm.getPageRenderTree(p);
      this.walk(tree, p);
    }
    let offset = 0;
    const parts: string[] = [];
    for (const line of this.lines) {
      line.start = offset;
      parts.push(line.text);
      offset += line.text.length + 1; // '\n'
    }
    this.fullText = parts.join('\n');
    return { text: this.fullText, lineCount: this.lines.length, pageCount };
  }

  private walk(node: unknown, page: number): void {
    if (Array.isArray(node)) {
      for (const child of node) this.walk(child, page);
      return;
    }
    if (!node || typeof node !== 'object') return;
    const n = node as Record<string, unknown>;
    if (n.type === 'TextLine' && n.bbox && typeof n.bbox === 'object') {
      const b = n.bbox as Record<string, number>;
      const text = this.lineText(n).trim();
      if (text) {
        this.lines.push({ page, x: b.x, y: b.y, w: b.w, h: b.h, text, start: 0 });
      }
      return;
    }
    for (const key of ['children', 'nodes']) {
      if (key in n) this.walk(n[key], page);
    }
  }

  private lineText(node: unknown): string {
    if (Array.isArray(node)) return node.map((c) => this.lineText(c)).join('');
    if (!node || typeof node !== 'object') return '';
    const n = node as Record<string, unknown>;
    if (n.type === 'TextRun' && typeof n.text === 'string') return n.text;
    let out = '';
    for (const key of ['children', 'nodes']) {
      if (key in n) out += this.lineText(n[key]);
    }
    return out;
  }

  /** 감사 결과를 받아 줄 배지로 그린다. 매핑 실패 건수를 돌려준다. */
  show(notes: EvidenceNote[]): { shown: number; unmapped: number } {
    this.clear(false);
    this.notes = notes;
    let shown = 0;
    let unmapped = 0;
    for (const note of notes) {
      const line = this.locate(note.sentence);
      if (!line) {
        unmapped++;
        continue;
      }
      this.badge(line, note);
      shown++;
    }
    return { shown, unmapped };
  }

  /** 문장 → 첫 줄. 감사기가 전문을 그대로 잘랐으므로 indexOf 가 1차 수단이다. */
  private locate(sentence: string): LineBox | null {
    const needle = sentence.trim();
    if (!needle) return null;
    let at = this.fullText.indexOf(needle);
    if (at < 0) {
      const squish = (s: string) => s.replace(/\s+/g, '');
      const head = squish(needle).slice(0, 24);
      if (!head) return null;
      const hay = squish(this.fullText);
      const hayAt = hay.indexOf(head);
      if (hayAt < 0) return null;
      let seen = 0;
      at = 0;
      for (let i = 0; i < this.fullText.length && seen < hayAt; i++) {
        if (!/\s/.test(this.fullText[i])) seen++;
        at = i + 1;
      }
    }
    for (let i = this.lines.length - 1; i >= 0; i--) {
      if (this.lines[i].start <= at) return this.lines[i];
    }
    return null;
  }

  private layer(page: number): HTMLElement {
    let el = this.layers[page];
    if (el) return el;
    el = document.createElement('div');
    el.className = 'evidence-notes-layer';
    el.dataset.page = String(page);
    el.style.position = 'absolute';
    el.style.pointerEvents = 'none';
    // 가상 스크롤러가 스크롤 중 페이지 캔버스를 나중 형제로 재부착하므로,
    // z-index 없이는 배지가 캔버스 밑에 깔린다.
    el.style.zIndex = '30';
    el.style.display = this.visible ? 'block' : 'none';
    this.scrollContent.appendChild(el);
    this.layers[page] = el;
    this.placeLayer(page);
    return el;
  }

  /** 페이지 캔버스와 동일한 배치 규칙으로 오버레이 층을 앉힌다. */
  private placeLayer(page: number): void {
    const el = this.layers[page];
    if (!el) return;
    const zoom = this.viewportManager.getZoom();
    el.style.top = `${this.virtualScroll.getPageOffset(page)}px`;
    el.style.width = `${this.virtualScroll.getPageWidth(page)}px`;
    el.style.height = `${this.virtualScroll.getPageHeight(page)}px`;
    const left = this.virtualScroll.getPageLeft(page);
    if (left >= 0) {
      el.style.left = `${left}px`;
      el.style.transform = 'none';
    } else {
      el.style.left = '50%';
      el.style.transform = 'translateX(-50%)';
    }
    el.dataset.zoom = String(zoom);
  }

  private badge(line: LineBox, note: EvidenceNote): void {
    const layer = this.layer(line.page);
    const zoom = this.viewportManager.getZoom();
    const color = VERDICT_COLOR[note.verdict] ?? '#64748b';

    const band = document.createElement('div');
    band.className = 'evidence-note-band';
    band.style.cssText = `position:absolute;left:${line.x * zoom}px;top:${line.y * zoom}px;` +
      `width:${line.w * zoom}px;height:${line.h * zoom}px;` +
      `background:${VERDICT_BG[note.verdict] ?? 'transparent'};border-left:3px solid ${color};` +
      `pointer-events:none;border-radius:2px;`;
    layer.appendChild(band);

    const dot = document.createElement('button');
    dot.className = 'evidence-note-dot';
    dot.type = 'button';
    dot.title = note.verdict;
    dot.textContent = note.verdict === '숫자불일치' ? '!' : note.verdict === '근거불명' ? '?' : '✓';
    dot.style.cssText = `position:absolute;left:${Math.max(2, line.x * zoom - 22)}px;top:${line.y * zoom}px;` +
      `width:16px;height:16px;border-radius:8px;border:none;cursor:pointer;` +
      `background:${color};color:#fff;font-size:11px;line-height:16px;text-align:center;` +
      `pointer-events:auto;padding:0;`;
    dot.addEventListener('click', (ev) => {
      ev.stopPropagation();
      this.openPopover(dot, note);
    });
    layer.appendChild(dot);
  }

  private openPopover(anchor: HTMLElement, note: EvidenceNote): void {
    this.closePopover();
    const pop = document.createElement('div');
    pop.className = 'evidence-note-popover';
    pop.style.cssText =
      'position:fixed;z-index:9999;max-width:380px;max-height:320px;overflow:auto;' +
      'background:#fff;border:1px solid #e2e8f0;border-radius:10px;padding:10px 12px;' +
      'box-shadow:0 8px 24px rgba(15,23,42,0.18);font-size:12px;color:#334155;';
    const color = VERDICT_COLOR[note.verdict];
    const renderSources = (sources: EvidenceSource[]) =>
      sources.length
        ? sources
            .map(
              (s) =>
                `<div style="margin-top:6px;padding:6px;border-radius:6px;background:#f8fafc;">` +
                `<div style="font-weight:600;color:#475569;">${escapeHtml(s.source_path)} · ${escapeHtml(s.locator)}</div>` +
                `<div style="margin-top:2px;color:#64748b;white-space:pre-wrap;">${escapeHtml(s.text)}</div></div>`,
            )
            .join('')
        : '<div style="color:#94a3b8;margin-top:4px;">접점이 있는 출처를 찾지 못했습니다.</div>';

    let body: string;
    if (note.verdict === '양식문구') {
      // 양식 제공 문구 — 작성자의 주장이 아니므로 사실 감사 대상이 아니다.
      body =
        '<div style="color:#64748b;margin-top:4px;">양식이 제공한 문구입니다 — 작성 내용 감사 대상이 아닙니다.</div>' +
        renderSources(note.sources);
    } else if (note.axes && note.axes.length > 0) {
      // 다축: 축마다 판정과 출처를 구분해 보여 준다 (내용 축 / 양식 축).
      body = note.axes
        .map((ax) => {
          const axColor = VERDICT_COLOR[ax.verdict] ?? '#334155';
          const nums = ax.unsupported_numbers.length
            ? `<div style="color:#dc2626;margin-top:2px;">확인되지 않은 수치: ${ax.unsupported_numbers.join(', ')}</div>`
            : '';
          return (
            `<div style="margin-top:8px;padding-top:6px;border-top:1px solid #f1f5f9;">` +
            `<div><span style="font-weight:700;color:#475569;">[${escapeHtml(ax.axis)}]</span> ` +
            `<span style="font-weight:700;color:${axColor};">${ax.verdict}</span></div>` +
            nums +
            renderSources(ax.sources) +
            `</div>`
          );
        })
        .join('');
    } else {
      const numbers = note.unsupported_numbers.length
        ? `<div style="color:#dc2626;margin-top:4px;">원문에서 확인되지 않은 수치: ${note.unsupported_numbers.join(', ')}</div>`
        : '';
      body = numbers + renderSources(note.sources);
    }
    pop.innerHTML =
      `<div style="font-weight:700;color:${color};">${note.verdict}</div>` +
      `<div style="margin-top:4px;">${escapeHtml(note.sentence)}</div>` +
      body;
    document.body.appendChild(pop);
    const rect = anchor.getBoundingClientRect();
    pop.style.left = `${Math.min(rect.right + 8, window.innerWidth - 400)}px`;
    pop.style.top = `${Math.min(rect.top, window.innerHeight - 340)}px`;
    this.popover = pop;
    const close = (ev: MouseEvent) => {
      if (!pop.contains(ev.target as Node)) {
        this.closePopover();
        document.removeEventListener('mousedown', close, true);
      }
    };
    document.addEventListener('mousedown', close, true);
  }

  private closePopover(): void {
    this.popover?.remove();
    this.popover = null;
  }

  /** 줌이 바뀌면 저장된 노트로 전부 다시 그린다. */
  private relayout(): void {
    if (this.notes.length === 0) return;
    const notes = this.notes;
    this.clear(false);
    this.show(notes);
  }

  clear(reset = true): void {
    this.closePopover();
    for (const layer of this.layers) layer?.remove();
    this.layers = [];
    if (reset) {
      this.notes = [];
      this.visible = true;
    }
  }

  /** 감사 결과를 버리지 않고 배지만 보이거나 숨긴다 (재감사 없는 토글용). */
  setVisible(visible: boolean): { visible: boolean } {
    this.visible = visible;
    if (!visible) this.closePopover();
    for (const layer of this.layers) {
      if (layer) layer.style.display = visible ? 'block' : 'none';
    }
    return { visible };
  }

  destroy(): void {
    this.clear();
    this.unsubscribe?.();
    this.unsubscribe = null;
  }
}

function escapeHtml(s: string): string {
  return s.replace(/[&<>"']/g, (c) =>
    c === '&' ? '&amp;' : c === '<' ? '&lt;' : c === '>' ? '&gt;' : c === '"' ? '&quot;' : '&#39;',
  );
}
