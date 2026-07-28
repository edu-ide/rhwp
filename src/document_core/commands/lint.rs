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
