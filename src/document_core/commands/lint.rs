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

    /// 레이아웃을 실제로 돌려야만 보이는 결함을 검사한다.
    ///
    /// 모델만 봐서는 절대 안 보인다 — 2026-07-31 실측: 답변 칸 글자를 몇 자
    /// 줄였더니 그 행이 쪽 바닥에 7px 못 미치는 데서 끝났고, 옆 테두리가
    /// 허공에서 끊긴 채 다음 쪽에서 상자가 다시 열렸다. 저장·추출·쪽수·
    /// 클리핑 검사와 기존 lint 를 전부 통과했고 `errors 0` 이 떴다.
    fn lint_layout(&self, findings: &mut Vec<Finding>) {
        use crate::renderer::render_tree::{RenderNode, RenderNodeType};

        /// 쪽 바닥에 닿았다고 볼 여유(px). 사업계획서 정상 쪽의 실측 오차가
        /// 0.5px 라 그보다 크고, 눈에 띄는 최소 어긋남(7px)보다 작게 잡는다.
        const SLACK_PX: f64 = 2.0;

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
                if gap > SLACK_PX {
                    findings.push(Finding {
                        level: "error",
                        code: "table-fragment-unclosed",
                        location: format!("sec{} para{} ctrl{} p{}", key.0, key.1, key.2, p + 1),
                        message: format!(
                            "다음 쪽으로 이어지는 표 조각이 쪽 바닥보다 {:.1}px 위에서 끝난다 \
                             — 옆 테두리가 허공에서 끊겨 상자가 안 닫힌 채로 보인다. \
                             그 행 높이를 쪽 경계에 맞춰 채워라(첫 행 예산 = 쪽내지-머리행, 이후 행 = 쪽내지)",
                            gap
                        ),
                    });
                }
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
