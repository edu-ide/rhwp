//! 한컴 호환성 lint — "저장은 되는데 한컴에서만 깨지는" 결함의 사전 진단.
//!
//! 2026-07-28 실기동(한컴독스)에서 라운드트립 자기검증이 못 잡은 결함들을
//! 규칙화했다: 셀 안 표의 tac 비트 누락(표 전멸), 매달린 BorderFill(테두리
//! 증발), instance id 충돌, 페이지를 넘는 셀(간이 뷰어 잘림), 세로 가운데
//! 정렬의 상단 공백. 검사는 보고만 하고 문서를 바꾸지 않는다.

use crate::model::control::Control;
use crate::model::shape::common_obj_offsets;
use crate::model::table::{Table, TablePageBreak};

use super::super::DocumentCore;

fn esc(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

struct Finding {
    level: &'static str, // "error" | "warn" | "info"
    code: &'static str,
    location: String,
    message: String,
}

impl Finding {
    fn json(&self) -> String {
        format!(
            "{{\"level\":\"{}\",\"code\":\"{}\",\"location\":\"{}\",\"message\":\"{}\"}}",
            self.level,
            self.code,
            esc(&self.location),
            esc(&self.message)
        )
    }
}

impl DocumentCore {
    /// 문서 전체를 검사해 한컴 호환성 소견을 JSON 으로 돌려준다.
    pub fn lint_native(&self) -> Result<String, crate::error::HwpError> {
        let mut findings: Vec<Finding> = Vec::new();
        let bf_count = self.document.doc_info.border_fills.len();
        let cs_count = self.document.doc_info.char_shapes.len();
        let ps_count = self.document.doc_info.para_shapes.len();

        let mut iids: std::collections::HashMap<u32, Vec<String>> =
            std::collections::HashMap::new();

        for (si, sec) in self.document.sections.iter().enumerate() {
            // 페이지 내지 높이 (HWPU): 셀이 이보다 크면 이월이 필요하다.
            let pd = &sec.section_def.page_def;
            let body_height = pd.height as i64
                - pd.margin_top as i64
                - pd.margin_bottom as i64
                - pd.margin_header as i64
                - pd.margin_footer as i64;

            for (pi, para) in sec.paragraphs.iter().enumerate() {
                for (ci, ctrl) in para.controls.iter().enumerate() {
                    let Control::Table(outer) = ctrl else { continue };
                    let loc = format!("sec{} para{} ctrl{}", si, pi, ci);
                    Self::lint_table_common(
                        outer,
                        &loc,
                        false,
                        &mut findings,
                        &mut iids,
                        bf_count,
                    );

                    for (cell_i, cell) in outer.cells.iter().enumerate() {
                        let cloc = format!("{} cell{}(r{}c{})", loc, cell_i, cell.row, cell.col);
                        if cell.border_fill_id == 0 {
                            findings.push(Finding {
                                level: "warn",
                                code: "cell-borderfill-zero",
                                location: cloc.clone(),
                                message: "셀 borderFillId 0 — 한컴에서 무테·무채움으로 보인다"
                                    .into(),
                            });
                        } else if cell.border_fill_id as usize > bf_count {
                            findings.push(Finding {
                                level: "error",
                                code: "cell-borderfill-dangling",
                                location: cloc.clone(),
                                message: format!(
                                    "셀 borderFillId {} 가 DocInfo({}개) 범위를 벗어남 — 재로드 시 0으로 클램프",
                                    cell.border_fill_id, bf_count
                                ),
                            });
                        }

                        // 셀 콘텐츠 높이(문단 줄배치 합) — sync 가 쓰는 정의와 동일
                        let mut content: i64 = 0;
                        for cp in &cell.paragraphs {
                            if let Some(last) = cp.line_segs.last() {
                                let bottom = last.vertical_pos as i64
                                    + last.line_height as i64
                                    + last.line_spacing as i64;
                                content = content.max(bottom);
                            }
                            for c2 in &cp.controls {
                                if let Control::Table(nested) = c2 {
                                    let nloc = format!("{} 중첩표", cloc);
                                    Self::lint_table_common(
                                        nested,
                                        &nloc,
                                        true,
                                        &mut findings,
                                        &mut iids,
                                        bf_count,
                                    );
                                }
                            }
                            if cp.para_shape_id as usize >= ps_count.max(1) {
                                findings.push(Finding {
                                    level: "error",
                                    code: "parashape-dangling",
                                    location: cloc.clone(),
                                    message: format!(
                                        "문단모양 id {} 범위 초과({}개)",
                                        cp.para_shape_id, ps_count
                                    ),
                                });
                            }
                            for csr in &cp.char_shapes {
                                if csr.char_shape_id as usize >= cs_count.max(1) {
                                    findings.push(Finding {
                                        level: "error",
                                        code: "charshape-dangling",
                                        location: cloc.clone(),
                                        message: format!(
                                            "글자모양 id {} 범위 초과({}개)",
                                            csr.char_shape_id, cs_count
                                        ),
                                    });
                                }
                            }
                        }
                        content += cell.padding.top.max(0) as i64 + cell.padding.bottom.max(0) as i64;

                        if (cell.height as i64) > body_height {
                            findings.push(Finding {
                                level: "info",
                                code: "cell-taller-than-page",
                                location: cloc.clone(),
                                message: format!(
                                    "셀 높이 {} > 페이지 내지 {} — 페이지 이월 셀. 데스크톱 한글은 이어 그리지만 한컴독스 웹·업스트림 뷰어는 초과분을 자른다",
                                    cell.height, body_height
                                ),
                            });
                        }

                        // 세로 가운데/아래 + 남는 공간 → 위 공백으로 보인다 (양식 셀에서 실측)
                        let slack = cell.height as i64 - content;
                        let is_center_or_bottom = !matches!(
                            cell.vertical_align,
                            crate::model::table::VerticalAlign::Top
                        );
                        if is_center_or_bottom && slack > 3_000 && !cell.paragraphs.is_empty() {
                            findings.push(Finding {
                                level: "warn",
                                code: "cell-valign-slack",
                                location: cloc.clone(),
                                message: format!(
                                    "세로 {}정렬 + 여유 {}HWPU — 한컴에서 내용 위에 공백이 뜬다 (위 정렬 권장)",
                                    if matches!(cell.vertical_align, crate::model::table::VerticalAlign::Center) { "가운데" } else { "아래" },
                                    slack
                                ),
                            });
                        }
                    }
                }
            }
        }

        for (iid, locs) in iids.iter() {
            if locs.len() > 1 {
                findings.push(Finding {
                    level: "warn",
                    code: "instance-id-duplicate",
                    location: locs.join(", "),
                    message: format!("표 instance id {:#x} 가 {}곳에서 중복", iid, locs.len()),
                });
            }
        }

        self.lint_table_style(&mut findings);
        self.lint_layout(&mut findings);

        let errors = findings.iter().filter(|f| f.level == "error").count();
        let warns = findings.iter().filter(|f| f.level == "warn").count();
        let body: Vec<String> = findings.iter().map(|f| f.json()).collect();
        Ok(format!(
            "{{\"ok\":true,\"errors\":{},\"warnings\":{},\"findings\":[{}]}}",
            errors,
            warns,
            body.join(",")
        ))
    }

    /// 표의 서식이 문서 안에서 제각각인지 검사한다.
    ///
    /// 왜 필요한가. 2026-07-29 에 「맨 위 셀 아래 두 줄이 왜 없냐」는 지적을
    /// 받아 고쳤는데, 07-31 에 새로 만든 표에서 **같은 결함이 그대로 재발**했다.
    /// 규칙이 없으면 고친 것이 돌아온다. 채움색·이중선·정렬·세로정렬은 저장도
    /// 되고 렌더도 되고 기존 lint 도 전부 통과하므로, 사람 눈 말고는 잡을
    /// 수단이 없었다.
    ///
    /// 절대 기준을 세우지 않고 **문서 안의 다수결**과 비교한다. 표가 하나뿐인
    /// 문서에는 비교 대상이 없으니 다수결 검사는 건너뛰고, 머리행이 본문과
    /// 구분되지 않는 경우만 따로 짚는다.
    fn lint_table_style(&self, findings: &mut Vec<Finding>) {
        use crate::document_core::helpers::{border_line_type_to_u8_val, color_ref_to_css};
        use crate::model::style::FillType;
        use crate::model::table::VerticalAlign;

        struct Style {
            loc: String,
            head_fill: String,
            head_rule: u8,
            head_align: String,
            body_align: String,
            body_valign: String,
        }

        // 배포 양식의 문항 표는 우리가 만든 게 아니고 「항목 및 서식 임의 수정
        // 불가」 대상이다. 그걸 우리 표와 같은 잣대로 재면 고칠 수 없는 소견만
        // 쌓인다(실측: 양식 문항 표 5개가 전부 걸렸다). 그래서 답변 칸 안에
        // 우리가 넣은 **중첩 표**가 있으면 그것들끼리만 비교하고, 없으면
        // (양식이 아닌 일반 문서다) 최상위 표끼리 비교한다.

        let fill_of = |bf_id: u16| -> String {
            if bf_id == 0 {
                return "none".into();
            }
            match self.document.doc_info.border_fills.get((bf_id - 1) as usize) {
                Some(bf) => match &bf.fill.solid {
                    Some(sf) if bf.fill.fill_type == FillType::Solid => {
                        color_ref_to_css(sf.background_color).to_lowercase()
                    }
                    _ => "none".into(),
                },
                None => "none".into(),
            }
        };
        // borders 는 [좌, 우, 상, 하] 순이다 — 아래는 3번이다
        let bottom_rule_of = |bf_id: u16| -> u8 {
            if bf_id == 0 {
                return 0;
            }
            self.document
                .doc_info
                .border_fills
                .get((bf_id - 1) as usize)
                .map(|bf| border_line_type_to_u8_val(bf.borders[3].line_type))
                .unwrap_or(0)
        };
        let align_of = |cell: &crate::model::table::Cell| -> String {
            cell.paragraphs
                .first()
                .and_then(|p| {
                    self.document
                        .doc_info
                        .para_shapes
                        .get(p.para_shape_id as usize)
                })
                .map(|ps| format!("{:?}", ps.alignment))
                .unwrap_or_else(|| "?".into())
        };
        let valign_name = |v: VerticalAlign| match v {
            VerticalAlign::Top => "Top",
            VerticalAlign::Center => "Center",
            VerticalAlign::Bottom => "Bottom",
        };

        let mut top: Vec<Style> = Vec::new();
        let mut nested: Vec<Style> = Vec::new();
        let mut shade_warn: Vec<Finding> = Vec::new();
        let mut collect = |t: &Table, loc: String, out: &mut Vec<Style>| {
            if t.row_count < 2 || t.col_count < 2 {
                return; // 머리행/본문 구분이 없는 표는 대상이 아니다
            }
            let head: Vec<&crate::model::table::Cell> =
                t.cells.iter().filter(|c| c.row == 0).collect();
            let body: Vec<&crate::model::table::Cell> =
                t.cells.iter().filter(|c| c.row > 0).collect();
            let (Some(h0), Some(b0)) = (head.first(), body.first()) else {
                return;
            };
            let head_fill = fill_of(h0.border_fill_id);
            let body_fill = fill_of(b0.border_fill_id);
            if head_fill == body_fill {
                out.push(Style {
                    loc: loc.clone(),
                    head_fill: head_fill.clone(),
                    head_rule: bottom_rule_of(h0.border_fill_id),
                    head_align: align_of(h0),
                    body_align: align_of(b0),
                    body_valign: valign_name(b0.vertical_align).into(),
                });
                shade_warn.push(Finding {
                    level: "warn",
                    code: "table-header-no-shade",
                    location: loc,
                    message: format!(
                        "머리행 채움({})이 본문과 같아 구분되지 않는다 — 관공서 작성 예시는 머리행에 음영(#d9d9d9)을 준다",
                        head_fill
                    ),
                });
                return;
            }
            out.push(Style {
                loc,
                head_fill,
                head_rule: bottom_rule_of(h0.border_fill_id),
                head_align: align_of(h0),
                body_align: align_of(b0),
                body_valign: valign_name(b0.vertical_align).into(),
            });
        };

        for (si, sec) in self.document.sections.iter().enumerate() {
            for (pi, para) in sec.paragraphs.iter().enumerate() {
                for (ci, ctrl) in para.controls.iter().enumerate() {
                    let Control::Table(outer) = ctrl else { continue };
                    let loc = format!("sec{} para{} ctrl{}", si, pi, ci);
                    collect(outer, loc.clone(), &mut top);
                    for cell in &outer.cells {
                        for cp in &cell.paragraphs {
                            for c2 in &cp.controls {
                                if let Control::Table(inner) = c2 {
                                    collect(
                                        inner,
                                        format!("{} cell(r{}c{}) 중첩표", loc, cell.row, cell.col),
                                        &mut nested,
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        // 우리가 넣은 표(중첩)가 있으면 그것만 본다 — 양식 표는 손댈 수 없다
        let use_nested = !nested.is_empty();
        let styles = if use_nested { nested } else { top };
        let keep: std::collections::HashSet<&str> =
            styles.iter().map(|s| s.loc.as_str()).collect();
        for f in shade_warn {
            if keep.contains(f.location.as_str()) {
                findings.push(f);
            }
        }

        if styles.len() < 2 {
            return; // 비교 대상이 없다
        }

        // 항목별 다수값을 구해 어긋난 표를 짚는다
        let mode = |vals: Vec<String>| -> String {
            let mut cnt: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
            for v in vals {
                *cnt.entry(v).or_insert(0) += 1;
            }
            cnt.into_iter()
                .max_by_key(|(v, n)| (*n, v.clone()))
                .map(|(v, _)| v)
                .unwrap_or_default()
        };
        let m_fill = mode(styles.iter().map(|s| s.head_fill.clone()).collect());
        let m_rule = mode(styles.iter().map(|s| s.head_rule.to_string()).collect());
        let m_ha = mode(styles.iter().map(|s| s.head_align.clone()).collect());
        let m_ba = mode(styles.iter().map(|s| s.body_align.clone()).collect());
        let m_va = mode(styles.iter().map(|s| s.body_valign.clone()).collect());

        for s in &styles {
            let mut diffs: Vec<String> = Vec::new();
            if s.head_fill != m_fill {
                diffs.push(format!("머리행 채움 {} (다수 {})", s.head_fill, m_fill));
            }
            if s.head_rule.to_string() != m_rule {
                diffs.push(format!(
                    "머리행 아래 선 종류 {} (다수 {})",
                    s.head_rule, m_rule
                ));
            }
            if s.head_align != m_ha {
                diffs.push(format!("머리행 정렬 {} (다수 {})", s.head_align, m_ha));
            }
            if s.body_align != m_ba {
                diffs.push(format!("본문 정렬 {} (다수 {})", s.body_align, m_ba));
            }
            if s.body_valign != m_va {
                diffs.push(format!("세로 정렬 {} (다수 {})", s.body_valign, m_va));
            }
            if !diffs.is_empty() {
                findings.push(Finding {
                    level: "warn",
                    code: "table-style-inconsistent",
                    location: s.loc.clone(),
                    message: format!("문서 안 다른 표와 서식이 다름 — {}", diffs.join(", ")),
                });
            }
        }
    }

    /// 레이아웃을 실제로 돌려야만 보이는 결함을 검사한다.
    ///
    /// 모델만 봐서는 절대 안 보인다 — 2026-07-31 실측: 답변 칸 글자를 몇 자
    /// 줄였더니 그 행이 쪽 바닥에 7px 못 미치는 데서 끝났고, 옆 테두리가
    /// 허공에서 끊긴 채 다음 쪽에서 상자가 다시 열렸다. 저장·추출·쪽수·
    /// 클리핑 검사와 기존 lint 를 전부 통과했고 `errors 0` 이 떴다.
    ///
    /// **판정은 "쪽 바닥에 닿았나"가 아니라 "상자가 닫혔나"다.** 조각 아래에
    /// 가로 테두리가 그려져 있으면 바닥보다 위에서 끝나도 상자는 닫혀 있다
    /// — 그건 아래 여백이 조금 넓어 보일 뿐 결함이 아니다. 반대로 테두리가
    /// 없으면 세로선만 허공에서 끊긴다. 그래서 두 조건을 함께 본다.
    fn lint_layout(&self, findings: &mut Vec<Finding>) {
        use crate::renderer::render_tree::{RenderNode, RenderNodeType};

        /// 쪽 바닥에 닿았다고 볼 여유(px).
        ///
        /// 2px 로 잡으면 **정상 문서를 오탐한다.** 배포 엔진은 쪽 예산에 정확히
        /// 맞춘 행을 다음 쪽으로 밀어내므로(머리행만 남아 12쪽→15쪽) 일부러
        /// 여유를 남겨야 하고, 실측한 최소 여유가 320 HWPU = 4.27px 였다
        /// (250 은 실패, 320 은 성공). 실제 결함은 7.2px 이상에서 나타난다.
        const SLACK_PX: f64 = 5.0;

        type Key = (usize, usize, usize);

        fn collect(
            node: &RenderNode,
            body_bottom: &mut Option<f64>,
            tables: &mut std::collections::HashMap<Key, f64>,
            inside_table: bool,
        ) {
            match node.node_type {
                RenderNodeType::Body { ref clip_rect } => {
                    if let Some(r) = clip_rect {
                        *body_bottom = Some(r.y + r.height);
                    }
                }
                RenderNodeType::Table(ref t) if !inside_table => {
                    // 최상위 표만 본다. 답변 칸 안의 중첩 표는 쪽 중간에서
                    // 끝나는 게 정상이라 대상이 아니다.
                    if let (Some(s), Some(p), Some(c)) =
                        (t.section_index, t.para_index, t.control_index)
                    {
                        let bottom = node.bbox.y + node.bbox.height;
                        tables
                            .entry((s, p, c))
                            .and_modify(|b| {
                                if bottom > *b {
                                    *b = bottom
                                }
                            })
                            .or_insert(bottom);
                    }
                    for ch in &node.children {
                        collect(ch, body_bottom, tables, true);
                    }
                    return;
                }
                _ => {}
            }
            for ch in &node.children {
                collect(ch, body_bottom, tables, inside_table);
            }
        }

        let pages = self.page_count() as usize;
        if pages < 2 {
            return;
        }
        let mut per_page: Vec<(Option<f64>, std::collections::HashMap<Key, f64>)> =
            Vec::with_capacity(pages);
        for p in 0..pages {
            let tree = match self.build_page_tree_cached(p as u32) {
                Ok(t) => t,
                Err(_) => return, // 레이아웃을 못 돌리면 이 검사만 건너뛴다
            };
            let mut bottom = None;
            let mut tables = std::collections::HashMap::new();
            collect(&tree.root, &mut bottom, &mut tables, false);
            per_page.push((bottom, tables));
        }

        for p in 0..pages - 1 {
            let (body_bottom, ref cur) = per_page[p];
            let Some(body_bottom) = body_bottom else {
                continue;
            };
            for (key, &frag_bottom) in cur {
                // 다음 쪽으로 이어지는 표만 대상이다. 그 쪽에서 끝나는 표는
                // 내용이 끝난 자리에서 닫히는 게 정상이다.
                if !per_page[p + 1].1.contains_key(key) {
                    continue;
                }
                let gap = body_bottom - frag_bottom;
                if gap <= SLACK_PX {
                    continue; // 쪽 바닥에 닿았다 — 이음매가 쪽 경계에 앉은 정상
                }
                // **우리 렌더러가 조각 아래에 선을 그렸는지는 근거가 못 된다.**
                // 2026-07-31 실측: 상자가 확실히 열려 보이는 문서인데도 우리 트리에는
                // 조각 바닥(1020.8)에 전폭 Line 이 있었다. 배포 엔진에는 없다.
                // 그 선을 "닫혔다"의 증거로 쓰자 6건이 전부 침묵했다 — 사용자가 열린
                // 상자를 보는 동안 검사는 통과를 내주던 상태와 정확히 같다.
                // 그래서 판정은 오직 "쪽 바닥에 닿았나" 하나로 한다.
                findings.push(Finding {
                    level: "error",
                    code: "table-fragment-unclosed",
                    location: format!("sec{} para{} ctrl{} p{}", key.0, key.1, key.2, p + 1),
                    message: format!(
                        "다음 쪽으로 이어지는 표 조각이 쪽 바닥보다 {:.1}px 위에서 끝난다 \
                         — 배포 엔진에서 옆 테두리가 허공에서 끊겨 상자가 열린 채로 보인다. \
                         그 행 높이를 쪽 경계까지 채워라(첫 행 예산 = 쪽내지-머리행, 이후 행 = 쪽내지)",
                        gap
                    ),
                });
            }
        }
    }

    fn lint_table_common(
        table: &Table,
        loc: &str,
        nested: bool,
        findings: &mut Vec<Finding>,
        iids: &mut std::collections::HashMap<u32, Vec<String>>,
        bf_count: usize,
    ) {
        let raw = &table.raw_ctrl_data;
        if raw.len() < common_obj_offsets::MIN_LEN {
            findings.push(Finding {
                level: "warn",
                code: "ctrl-header-short",
                location: loc.to_string(),
                message: format!(
                    "컨트롤 공통 헤더 {}B < 최소 {}B — 한컴 파서가 필드를 어긋나게 읽을 수 있다",
                    raw.len(),
                    common_obj_offsets::MIN_LEN
                ),
            });
            return;
        }
        let flags = u32::from_le_bytes(raw[common_obj_offsets::FLAGS].try_into().unwrap());
        let iid = u32::from_le_bytes(raw[common_obj_offsets::INSTANCE_ID].try_into().unwrap());

        if nested && flags & 1 == 0 {
            findings.push(Finding {
                level: "error",
                code: "nested-table-not-tac",
                location: loc.to_string(),
                message: format!(
                    "셀 안 표의 like_char(tac) 비트가 0 (flags={:#010x}) — 한컴이 표를 통째로 버린다 (repair-nested-tables 로 수리)",
                    flags
                ),
            });
        }
        if nested && !matches!(table.page_break, TablePageBreak::None) {
            findings.push(Finding {
                level: "warn",
                code: "nested-table-pagebreak",
                location: loc.to_string(),
                message: "셀 안 표가 쪽 경계 나눔 허용 — 간이 뷰어에서 반쯤 잘린다 (나누지 않음 권장)"
                    .into(),
            });
        }
        if iid == 0 {
            findings.push(Finding {
                level: "warn",
                code: "instance-id-zero",
                location: loc.to_string(),
                message: "표 instance id 0 — 한컴은 고유한 비-0 값을 기대".into(),
            });
        } else {
            iids.entry(iid).or_default().push(loc.to_string());
        }
        for cell in &table.cells {
            if cell.border_fill_id as usize > bf_count {
                findings.push(Finding {
                    level: "error",
                    code: "cell-borderfill-dangling",
                    location: format!("{} r{}c{}", loc, cell.row, cell.col),
                    message: format!(
                        "셀 borderFillId {} 가 DocInfo({}개) 범위를 벗어남",
                        cell.border_fill_id, bf_count
                    ),
                });
            }
        }
    }
}
