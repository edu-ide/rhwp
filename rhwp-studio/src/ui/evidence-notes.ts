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

export type EvidenceAiReview = { verdict: string; reason: string; kept_sources: number[]; span_before?: number; span_after?: number };

export type EvidenceNote = {
  sentence: string;
  verdict: '근거확보' | '근거불명' | '숫자불일치' | '양식문구';
  /** "작성"(작성자가 쓴 줄) | "양식"(양식 제공 문구) — 구버전 감사기는 생략. */
  origin?: '작성' | '양식';
  unsupported_numbers: string[];
  sources: EvidenceSource[];
  /** 다축 감사일 때 축별 대조 결과 (양식 축 등). */
  axes?: EvidenceAxisNote[];
  /** AI 재심 소견 — 규칙이 검증한 후보 안에서 로컬 LLM이 고른 결과. */
  ai_review?: EvidenceAiReview;
};

type LineBox = { page: number; x: number; y: number; w: number; h: number; text: string; start: number; block: number; pi: number };

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
  private blockSeq = 0;

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
    this.blockSeq = 0;
    const pageCount = this.wasm.pageCount;
    for (let p = 0; p < pageCount; p++) {
      const tree = this.wasm.getPageRenderTree(p);
      this.walk(tree, p);
    }
    // 줄바꿈은 문장 경계가 아니다 — 같은 문단(pi)의 줄들은 공백으로 잇고,
    // 문단이 바뀔 때만 개행을 넣는다. 줄 단위로 끊으면 "전환하라는 방향을
    // 받음" 같은 문장 조각이 각각 감사되어 근거불명으로 쏠린다는 것이
    // 실측으로 드러났다. (블록(block)은 표 셀·머리글마다 증가해, 다른
    // 셀의 같은 pi 가 한 문단으로 붙는 것을 막는다.)
    let offset = 0;
    const parts: string[] = [];
    this.lines.forEach((line, i) => {
      line.start = offset;
      parts.push(line.text);
      const next = this.lines[i + 1];
      const samePara = !!next && next.block === line.block && next.pi === line.pi && next.page === line.page;
      parts.push(samePara ? ' ' : '\n');
      offset += line.text.length + 1;
    });
    this.fullText = parts.join('').replace(/[\s\n]$/, '');
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
        const pi = typeof n.pi === 'number' ? n.pi : -1;
        this.lines.push({ page, x: b.x, y: b.y, w: b.w, h: b.h, text, start: 0, block: this.blockSeq, pi });
      }
      return;
    }
    // 표 셀·머리글·꼬리글은 별개 텍스트 블록이다 — 문단 인덱스(pi)가 셀마다
    // 0부터 다시 시작하므로, 블록 경계를 넘어 문단을 잇지 않는다.
    if (n.type === 'Cell' || n.type === 'Header' || n.type === 'Footer') {
      this.blockSeq++;
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
    // 문장마다 배지를 붙이면 문서가 배지로 뒤덮인다 — 구간으로 병합한다.
    // 구간의 경계는 하이브리드다: 규칙(같은 판정·같은 문단·연속 줄)이
    // 기본이고, AI 재심이 "같은 주장"이라고 지정한 이웃(span)은 판정이
    // 달라도 한 구간으로 묶는다 — 주장의 의미 단위는 규칙만으로 못 본다.
    type Entry = { note: EvidenceNote; line: LineBox; idx: number; noteIdx: number };
    const entries: Entry[] = [];
    notes.forEach((note, noteIdx) => {
      const line = this.locate(note.sentence);
      if (!line) {
        unmapped++;
        return;
      }
      entries.push({ note, line, idx: this.lines.indexOf(line), noteIdx });
      shown++;
    });
    entries.sort((a, b) => a.idx - b.idx);
    const aiLinked = (a: Entry, b: Entry): boolean => {
      // 노트 순서 기준 서로 이웃 — 어느 한쪽의 span 이 상대를 포함하면 연결.
      const d = b.noteIdx - a.noteIdx;
      if (d < 1 || d > 2) return false;
      const fwd = a.note.ai_review?.span_after ?? 0;
      const bwd = b.note.ai_review?.span_before ?? 0;
      return fwd >= d || bwd >= d;
    };
    let i = 0;
    while (i < entries.length) {
      let j = i;
      while (
        j + 1 < entries.length &&
        entries[j + 1].line.page === entries[i].line.page &&
        entries[j + 1].idx - entries[j].idx <= 1 &&
        ((entries[j + 1].note.verdict === entries[i].note.verdict &&
          entries[j + 1].line.pi === entries[i].line.pi &&
          entries[j + 1].line.block === entries[i].line.block) ||
          aiLinked(entries[j], entries[j + 1]))
      ) j++;
      this.segment(entries.slice(i, j + 1));
      i = j + 1;
    }
    return { shown, unmapped };
  }

  /** 구간(같은 판정의 연속 줄) 하나를 그린다 — 줄 띠 + 배지 1개. */
  private segment(entries: { note: EvidenceNote; line: LineBox; idx: number; noteIdx?: number }[]): void {
    const first = entries[0];
    // 구간 대표 판정은 가장 위험한 등급 — AI 가 묶은 혼합 구간에서 초록이
    // 빨강을 가리면 안 된다.
    const RANK: Record<string, number> = { 숫자불일치: 0, 근거불명: 1, 양식문구: 2, 근거확보: 3 };
    const verdict = entries.reduce(
      (worst, e) => ((RANK[e.note.verdict] ?? 9) < (RANK[worst] ?? 9) ? e.note.verdict : worst),
      first.note.verdict,
    );
    const layer = this.layer(first.line.page);
    const zoom = this.viewportManager.getZoom();
    const color = VERDICT_COLOR[verdict] ?? '#64748b';

    const seenIdx = new Set<number>();
    for (const { line, idx } of entries) {
      if (seenIdx.has(idx)) continue;
      seenIdx.add(idx);
      const band = document.createElement('div');
      band.className = 'evidence-note-band';
      band.style.cssText = `position:absolute;left:${line.x * zoom}px;top:${line.y * zoom}px;` +
        `width:${line.w * zoom}px;height:${line.h * zoom}px;` +
        `background:${VERDICT_BG[verdict] ?? 'transparent'};border-left:3px solid ${color};` +
        `pointer-events:none;border-radius:2px;`;
      layer.appendChild(band);
    }

    const dot = document.createElement('button');
    dot.className = 'evidence-note-dot';
    dot.type = 'button';
    dot.title = entries.length > 1 ? `${verdict} · 문장 ${entries.length}개` : verdict;
    dot.setAttribute('aria-label', dot.title);
    dot.textContent = verdict === '숫자불일치' ? '!' : verdict === '근거불명' ? '?' : verdict === '양식문구' ? '§' : '✓';
    dot.style.cssText = `position:absolute;left:${Math.max(2, first.line.x * zoom - 22)}px;top:${first.line.y * zoom}px;` +
      `width:16px;height:16px;border-radius:8px;border:none;cursor:pointer;` +
      `background:${color};color:#fff;font-size:11px;line-height:16px;text-align:center;` +
      `pointer-events:auto;padding:0;`;
    dot.addEventListener('click', (ev) => {
      ev.stopPropagation();
      this.openPopover(dot, entries.map((e) => e.note));
    });
    layer.appendChild(dot);
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

  private openPopover(anchor: HTMLElement, notes: EvidenceNote[]): void {
    this.closePopover();
    const pop = document.createElement('div');
    pop.className = 'evidence-note-popover';
    pop.style.cssText =
      'position:fixed;z-index:9999;max-width:380px;max-height:320px;overflow:auto;' +
      'background:#fff;border:1px solid #e2e8f0;border-radius:10px;padding:10px 12px;' +
      'box-shadow:0 8px 24px rgba(15,23,42,0.18);font-size:12px;color:#334155;';
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

    const noteBody = (note: EvidenceNote): string => {
      if (note.verdict === '양식문구') {
        // 양식 제공 문구 — 작성자의 주장이 아니므로 사실 감사 대상이 아니다.
        return (
          '<div style="color:#64748b;margin-top:4px;">양식이 제공한 문구입니다 — 작성 내용 감사 대상이 아닙니다.</div>' +
          renderSources(note.sources)
        );
      }
      if (note.axes && note.axes.length > 0) {
        // 다축: 축마다 판정과 출처를 구분해 보여 준다 (내용 축 / 양식 축).
        return note.axes
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
      }
      const numbers = note.unsupported_numbers.length
        ? `<div style="color:#dc2626;margin-top:4px;">원문에서 확인되지 않은 수치: ${note.unsupported_numbers.join(', ')}</div>`
        : '';
      return numbers + renderSources(note.sources);
    };

    // 구간 노트: 첫 문장은 근거까지 전부, 나머지 문장은 접어서(요약 줄로)
    // 보여 준다 — 구간이 길어도 팝오버가 문서만큼 길어지지 않는다.
    const first = notes[0];
    const color = VERDICT_COLOR[first.verdict];
    const MAX_LISTED = 8;
    const rest = notes.slice(1, 1 + MAX_LISTED);
    const restHtml = rest
      .map((n) => {
        const nums = n.unsupported_numbers.length
          ? ` <span style="color:#dc2626;">(미확인: ${n.unsupported_numbers.join(', ')})</span>`
          : '';
        return `<li style="margin-top:2px;color:#475569;">${escapeHtml(n.sentence)}${nums}</li>`;
      })
      .join('');
    const moreCount = notes.length - 1 - rest.length;
    const aiBadge = first.ai_review
      ? `<div style="margin-top:6px;padding:6px 8px;border-radius:6px;background:#eef2ff;color:#4338ca;font-size:11px;">` +
        `<b>AI 재심(로컬):</b> ${escapeHtml(first.ai_review.verdict)} — ${escapeHtml(first.ai_review.reason)}</div>`
      : '';
    pop.innerHTML =
      `<div style="font-weight:700;color:${color};">${first.verdict}` +
      (notes.length > 1 ? `<span style="margin-left:6px;font-weight:400;font-size:10.5px;color:#94a3b8;">문장 ${notes.length}개 구간</span>` : '') +
      `</div>` +
      `<div style="margin-top:4px;">${escapeHtml(first.sentence)}</div>` +
      aiBadge +
      noteBody(first) +
      (rest.length
        ? `<div style="margin-top:8px;padding-top:6px;border-top:1px solid #f1f5f9;font-weight:600;color:#64748b;">같은 구간의 문장</div>` +
          `<ul style="margin:2px 0 0 14px;padding:0;">${restHtml}</ul>` +
          (moreCount > 0 ? `<div style="margin-top:2px;color:#94a3b8;">…외 ${moreCount}건</div>` : '')
        : '');
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
