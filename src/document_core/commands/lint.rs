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

/// 검사가 **실제로 무엇을 봤는지** 세는 계수기.
///
/// 왜 필요한가. 2026-07-31 에 클리핑 검사기가 렌더 트리의 키를 잘못 읽어
/// **한 줄도 안 보고 "0건"** 을 보고한 적이 있다. `findings: []` 만으로는
/// "결함이 없다"와 "검사기가 죽었다"를 구분할 수 없다. 그래서 lint 응답에
/// 훑은 대상 수를 항상 함께 실어 계기가 살아 있는지 눈으로 확인하게 한다.
#[derive(Default)]
struct LintScan {
    /// 레이아웃을 돌린 쪽 수 (0 이면 레이아웃 검사가 통째로 건너뛰어졌다)
    pages: usize,
    /// 렌더 트리에서 만난 셀 수
    cells: usize,
    /// 렌더 트리에서 만난 글자 런 수
    text_runs: usize,
    /// 모델에서 훑은 문단 수 (본문 + 셀 + 중첩 셀)
    paragraphs: usize,
    /// 모델에서 훑은 글자 수 (공백·제어문자 제외)
    chars: usize,
    /// 위아래로 이어지는 답변 칸 쌍 (소제목 고아 검사가 실제로 본 이음매 수)
    cell_seams: usize,
}

impl LintScan {
    fn json(&self) -> String {
        format!(
            "{{\"pages\":{},\"cells\":{},\"textRuns\":{},\"paragraphs\":{},\"chars\":{},\"cellSeams\":{}}}",
            self.pages, self.cells, self.text_runs, self.paragraphs, self.chars, self.cell_seams
        )
    }
}

/// 글머리표로 읽은 문단의 계층. 작을수록 위 계층이다.
///
/// 본문(글머리표 없음)을 가장 깊은 값으로 두는 것이 핵심이다. `□ 표제` 다음에
/// 서술 문단이 오는 것도 "표제와 그 내용"이므로 같은 규칙으로 걸려야 한다.
fn outline_level(text: &str) -> u8 {
    match text.trim_start().chars().next() {
        Some('□') | Some('■') | Some('▣') | Some('◆') => 1,
        Some('◦') | Some('○') | Some('●') | Some('ㅇ') => 2,
        Some('-') | Some('–') | Some('‐') | Some('•') | Some('∙') => 3,
        _ => 9,
    }
}


/// 쪽 내지(본문이 놓이는) 높이. 셀이 이만큼 크면 그 셀 경계가 곧 쪽 경계다.
fn page_body_height(sec: &crate::model::document::Section) -> i64 {
    let pd = &sec.section_def.page_def;
    pd.height as i64
        - pd.margin_top as i64
        - pd.margin_bottom as i64
        - pd.margin_header as i64
        - pd.margin_footer as i64
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

        let mut scan = LintScan::default();
        self.lint_table_style(&mut findings);
        self.lint_orphan_heading(&mut findings, &mut scan);
        self.lint_pushed_paragraph(&mut findings, &mut scan);
        self.lint_char_style(&mut findings, &mut scan);
        self.lint_layout(&mut findings, &mut scan);

        let errors = findings.iter().filter(|f| f.level == "error").count();
        let warns = findings.iter().filter(|f| f.level == "warn").count();
        let body: Vec<String> = findings.iter().map(|f| f.json()).collect();
        Ok(format!(
            "{{\"ok\":true,\"errors\":{},\"warnings\":{},\"scanned\":{},\"findings\":[{}]}}",
            errors,
            warns,
            scan.json(),
            body.join(",")
        ))
    }

    /// 소제목만 칸 끝에 남고 그 내용은 다음 칸에서 시작하는 곳을 찾는다.
    ///
    /// 왜 필요한가. 이 양식들은 한 문항의 답변을 **쪽마다 별도 셀**로 쪼개
    /// 담는다(§4-1). 그래서 셀 경계가 곧 쪽 경계이고, 한글의 「다음 문단과
    /// 함께」(keepWithNext)는 흐름이 셀을 넘지 않으므로 아무 효과가 없다.
    /// 즉 소제목이 쪽 맨 아래 홀로 남는 사고를 막아 주는 장치가 **하나도
    /// 없다**. 글을 한 줄 넣고 빼는 것만으로도 조용히 생기고, 쪽수·글자수·
    /// 잘림 검사는 전부 통과한다 — 사람 눈 말고는 잡을 수단이 없었다.
    ///
    /// 판정은 글머리표 계층으로 한다. 다만 **모든 위 계층이 표제는 아니다** —
    /// 처음에 「앞이 뒤보다 위 계층이면 표제」로 잡았더니 5건 중 4건이 오탐이었다.
    /// `-` 항목은 잎이라 다음에 무엇이 오든 표제가 아니고(목록이 끝나고 그림이
    /// 시작하는 정상적인 자리까지 걸렸다), `◦` 는 짧은 표제일 때도 있고 그
    /// 자체로 완결된 서술일 때도 있다. 그래서 표제로 인정하는 자리는 둘뿐이다:
    ///
    /// - `□` 뒤에 무엇이든 딸려 오는 경우 (절 표제는 항상 표제다)
    /// - `◦` 뒤에 `-` 자식이 오는 경우 (자식이 있다는 것이 표제라는 증거다)
    ///
    /// 목록이 쪽을 넘어 이어지는 것(`-` 다음 `-`)은 계층이 같으므로 안 걸린다.
    fn lint_orphan_heading(&self, findings: &mut Vec<Finding>, scan: &mut LintScan) {
        let meaningful = |p: &crate::model::paragraph::Paragraph| {
            !p.text.trim().is_empty() || !p.controls.is_empty()
        };

        for (si, sec) in self.document.sections.iter().enumerate() {
            let half_page = page_body_height(sec) / 2;
            for (pi, para) in sec.paragraphs.iter().enumerate() {
                for (ci, ctrl) in para.controls.iter().enumerate() {
                    let Control::Table(t) = ctrl else { continue };

                    for above in &t.cells {
                        // 쪽만큼 큰 칸에서만 셀 경계가 쪽 경계다. 표지의 이름·
                        // 닉네임 같은 라벨 행은 위아래로 이어져 있어도 쪽을
                        // 가르지 않으므로 여기서 걸러낸다(실측 오탐).
                        if (above.height as i64) < half_page {
                            continue;
                        }
                        // 바로 아래에서 이어지는 칸: 같은 열·같은 병합 폭이어야
                        // 한 답변 칸의 연속이다. 머리행(병합 폭이 다르다)은
                        // 여기서 자동으로 걸러진다.
                        let Some(below) = t.cells.iter().find(|c| {
                            c.row == above.row + above.row_span
                                && c.col == above.col
                                && c.col_span == above.col_span
                        }) else {
                            continue;
                        };
                        scan.cell_seams += 1;

                        let Some(tail) = above.paragraphs.iter().rev().find(|p| meaningful(p))
                        else {
                            continue;
                        };
                        let Some(head) = below.paragraphs.iter().find(|p| meaningful(p)) else {
                            continue;
                        };
                        // 표가 든 문단은 표제가 아니다.
                        if !tail.controls.is_empty() {
                            continue;
                        }

                        let up = outline_level(&tail.text);
                        let down = outline_level(&head.text);
                        let is_heading = (up == 1 && down > 1) || (up == 2 && down == 3);
                        if !is_heading {
                            continue;
                        }

                        let title: String = tail.text.trim().chars().take(28).collect();
                        findings.push(Finding {
                            level: "warn",
                            code: "heading-orphaned-at-cell-end",
                            location: format!(
                                "sec{} para{} ctrl{} r{}c{}",
                                si, pi, ci, above.row, above.col
                            ),
                            message: format!(
                                "소제목 「{}」만 이 칸 끝에 남고 내용은 다음 칸(=다음 쪽)에서 시작한다 — 다음 칸 맨 앞으로 옮길 것",
                                title
                            ),
                        });
                    }
                }
            }
        }
    }

    /// 앞 칸에 자리가 남는데 뒤 칸으로 밀려간 문단을 찾는다.
    ///
    /// 소제목 고아의 거울상이다. 답변을 쪽마다 칸으로 쪼갠 뒤 글자 크기·줄간격·
    /// 문장을 고치면 앞 칸에 **새로 자리가 생기는데**, 쪼갠 자리는 그때 그대로
    /// 남는다. 그래서 쪽 아래가 다섯 줄쯤 비었는데 같은 목록의 마지막 항목만
    /// 다음 쪽으로 넘어가 있는 모양이 된다(실측: 사업계획서 Q3 「- 이후 정부
    /// 조달·바우처와 업종 세미나로 확산」).
    ///
    /// 판정 단위는 문단이 아니라 **옮겨도 안전한 덩어리**다. 처음에 「자식 없는
    /// 문단 하나」로 좁혔더니, 앞이 ◦-자식 블록·그림인 이음매의 17줄짜리
    /// 구멍(사용자가 눈으로 잡은 바로 그것)을 규칙이 전혀 못 봤다. 덩어리는:
    ///
    /// - 표제(□/◦) + 그보다 깊은 뒤따름 전부 — 통째로 가면 고아가 안 생긴다
    /// - 개체(그림/표) 문단 + 그 캡션 — 따로 가면 그림과 캡션이 쪽으로 갈라진다
    /// - 잎(`-`·보통 문단) 하나 — 목록은 쪽을 넘어 이어져도 된다
    ///
    /// 덩어리가 통째로 들어갈 때만 짚는다.
    fn lint_pushed_paragraph(&self, findings: &mut Vec<Finding>, scan: &mut LintScan) {
        let meaningful = |p: &crate::model::paragraph::Paragraph| {
            !p.text.trim().is_empty() || !p.controls.is_empty()
        };
        // ParaShape 는 문단 간격을 HWPUNIT 2배로 저장한다(style_resolver 규약).
        let space_before = |p: &crate::model::paragraph::Paragraph| -> i64 {
            self.document
                .doc_info
                .para_shapes
                .get(p.para_shape_id as usize)
                .map(|ps| ps.spacing_before as i64 / 2)
                .unwrap_or(0)
                .max(0)
        };
        let para_height = |p: &crate::model::paragraph::Paragraph| -> i64 {
            space_before(p)
                + p.line_segs
                    .iter()
                    .map(|s| s.line_height as i64 + s.line_spacing as i64)
                    .sum::<i64>()
        };

        for (si, sec) in self.document.sections.iter().enumerate() {
            let half_page = page_body_height(sec) / 2;
            for (pi, para) in sec.paragraphs.iter().enumerate() {
                for (ci, ctrl) in para.controls.iter().enumerate() {
                    let Control::Table(t) = ctrl else { continue };

                    for above in &t.cells {
                        if (above.height as i64) < half_page {
                            continue;
                        }
                        let Some(below) = t.cells.iter().find(|c| {
                            c.row == above.row + above.row_span
                                && c.col == above.col
                                && c.col_span == above.col_span
                        }) else {
                            continue;
                        };

                        // 앞 칸이 쓰고 남은 높이 (cell-valign-slack 과 같은 셈)
                        let mut used: i64 = 0;
                        for cp in &above.paragraphs {
                            if let Some(last) = cp.line_segs.last() {
                                used = used.max(
                                    last.vertical_pos as i64
                                        + last.line_height as i64
                                        + last.line_spacing as i64,
                                );
                            }
                        }
                        used += above.padding.top.max(0) as i64 + above.padding.bottom.max(0) as i64;
                        let slack = above.height as i64 - used;
                        if slack <= 0 {
                            continue;
                        }

                        let Some((hi, head)) =
                            below.paragraphs.iter().enumerate().find(|(_, p)| meaningful(p))
                        else {
                            continue;
                        };

                        // 덩어리 경계: 개체는 캡션까지, 표제는 더 깊은 뒤따름까지.
                        let ps = &below.paragraphs;
                        let mut end = hi + 1;
                        if !head.controls.is_empty() {
                            if end < ps.len()
                                && matches!(
                                    ps[end].text.trim_start(),
                                    t if t.starts_with("[그림]") || t.starts_with("[표]")
                                )
                            {
                                end += 1;
                            }
                        } else {
                            let mine = outline_level(&head.text);
                            if mine <= 2 {
                                while end < ps.len()
                                    && (!meaningful(&ps[end])
                                        || outline_level(&ps[end].text) > mine
                                        || !ps[end].controls.is_empty())
                                {
                                    end += 1;
                                }
                            }
                        }
                        // 덩어리 뒤에 아무것도 안 남으면 출발 칸이 비어 버린다.
                        if end >= ps.len() {
                            continue;
                        }
                        let h: i64 = ps[..end].iter().map(|p| para_height(p)).sum();
                        if h == 0 || h > slack {
                            continue;
                        }

                        let title: String = head.text.trim().chars().take(28).collect();
                        findings.push(Finding {
                            level: "warn",
                            code: "paragraph-pushed-despite-room",
                            location: format!(
                                "sec{} para{} ctrl{} r{}c{}",
                                si, pi, ci, above.row, above.col
                            ),
                            message: format!(
                                "앞 칸에 {}HWPU 여유가 있는데 다음 칸 첫 덩어리 「{}」({}문단, {}HWPU)가 넘어가 있다 — 앞 칸으로 끌어올릴 것",
                                slack, title, end, h
                            ),
                        });
                        let _ = scan;
                    }
                }
            }
        }
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

    /// 본문·셀 문단을 위치와 함께 모두 모은다 (중첩 표 포함).
    fn collect_paragraphs(&self) -> Vec<(String, &crate::model::paragraph::Paragraph)> {
        fn walk_table<'a>(
            t: &'a Table,
            loc: &str,
            out: &mut Vec<(String, &'a crate::model::paragraph::Paragraph)>,
        ) {
            for cell in &t.cells {
                let cloc = format!("{} cell(r{}c{})", loc, cell.row, cell.col);
                for (pi, cp) in cell.paragraphs.iter().enumerate() {
                    out.push((format!("{} para{}", cloc, pi), cp));
                    for c2 in &cp.controls {
                        if let Control::Table(inner) = c2 {
                            walk_table(inner, &format!("{} 중첩표", cloc), out);
                        }
                    }
                }
            }
        }

        let mut out = Vec::new();
        for (si, sec) in self.document.sections.iter().enumerate() {
            for (pi, para) in sec.paragraphs.iter().enumerate() {
                out.push((format!("sec{} para{}", si, pi), para));
                for (ci, ctrl) in para.controls.iter().enumerate() {
                    if let Control::Table(t) = ctrl {
                        walk_table(t, &format!("sec{} para{} ctrl{}", si, pi, ci), &mut out);
                    }
                }
            }
        }
        out
    }

    /// 글자모양이 남긴 자국(색·크기)을 검사한다.
    ///
    /// 왜 필요한가. 두 결함 다 2026-07-31 산출물에서 사용자 눈에만 걸렸다.
    ///
    /// 1. **파란 안내문 글자모양이 본문에 남는다.** 배포 양식의 안내문은
    ///    파란 12pt 다. 그 문단을 지우고 우리 글을 쓰면 새 글자가 안내문의
    ///    글자모양을 그대로 물려받는다. 실측: 한 산출물에서 파란 글자
    ///    1,390자 = 전체 29%. 저장도 되고 렌더도 된다.
    /// 2. **글자 수를 줄이면 꼬리 몇 자가 옛 크기로 남는다.** 캡션을 81→72자로
    ///    줄였더니 마지막 3자만 12pt 로 남아 화면에서 크기가 갈렸다.
    ///
    /// 둘 다 **비율**로 본다. 강조용으로 몇 자 색을 주거나 문단을 절반씩 두
    /// 크기로 쓰는 건 정상이고, 결함은 "대부분과 다른 소수"로 나타난다.
    fn lint_char_style(&self, findings: &mut Vec<Finding>, scan: &mut LintScan) {
        use crate::document_core::helpers::color_ref_to_css;

        /// 검정 계열로 볼 상한. 순검정(#000000)만 통과시키면 진회색 본문
        /// (#333333)이 전부 걸린다. 파랑(#0000ff)·빨강은 그대로 걸린다.
        const BLACKISH: u32 = 0x40;
        /// 색 글자 비율이 이보다 크면 「물려받은 안내문 서식」으로 본다.
        /// 강조 몇 군데는 보통 1% 미만이고, 실측 결함은 29% 였다.
        const COLOR_RATIO: f64 = 0.03;
        /// 비율을 말할 만한 최소 글자 수 (이보다 짧으면 표본이 못 된다)
        const MIN_DOC_CHARS: usize = 100;
        /// 한 문단 안에서 크기가 갈렸다고 볼 소수 쪽 상한
        const SIZE_MINORITY_RATIO: f64 = 0.20;
        /// 크기 검사를 할 최소 문단 길이 (짧은 문단은 1~2자가 20%를 넘는다)
        const MIN_PARA_CHARS: usize = 10;

        let is_blackish = |c: u32| {
            let (r, g, b) = (c & 0xFF, (c >> 8) & 0xFF, (c >> 16) & 0xFF);
            r <= BLACKISH && g <= BLACKISH && b <= BLACKISH
        };

        let paras = self.collect_paragraphs();
        scan.paragraphs = paras.len();

        let mut total = 0usize;
        let mut colored = 0usize;
        let mut by_color: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        let mut by_loc: Vec<(String, usize)> = Vec::new();

        for (loc, para) in &paras {
            let mut sizes: std::collections::HashMap<i32, usize> = std::collections::HashMap::new();
            let mut para_total = 0usize;
            let mut para_colored = 0usize;

            for (i, ch) in para.text.chars().enumerate() {
                // 공백·제어문자는 색도 크기도 눈에 안 보인다 — 세지 않는다
                if ch.is_whitespace() || ch.is_control() {
                    continue;
                }
                let Some(id) = para.char_shape_id_at(i) else {
                    continue;
                };
                let Some(cs) = self.document.doc_info.char_shapes.get(id as usize) else {
                    continue;
                };
                para_total += 1;
                if !is_blackish(cs.text_color) {
                    para_colored += 1;
                    *by_color.entry(color_ref_to_css(cs.text_color)).or_insert(0) += 1;
                }
                *sizes.entry(cs.base_size).or_insert(0) += 1;
            }

            total += para_total;
            colored += para_colored;
            if para_colored > 0 {
                by_loc.push((loc.clone(), para_colored));
            }

            // char-size-mixed — 한 문단 안 소수 크기
            if para_total >= MIN_PARA_CHARS && sizes.len() >= 2 {
                let (dom_size, dom_n) = sizes
                    .iter()
                    .map(|(s, n)| (*s, *n))
                    .max_by_key(|(s, n)| (*n, *s))
                    .unwrap();
                let minority = para_total - dom_n;
                let ratio = minority as f64 / para_total as f64;
                if minority > 0 && ratio < SIZE_MINORITY_RATIO {
                    let mut rest: Vec<(i32, usize)> = sizes
                        .iter()
                        .filter(|(s, _)| **s != dom_size)
                        .map(|(s, n)| (*s, *n))
                        .collect();
                    rest.sort_by(|a, b| b.1.cmp(&a.1));
                    let rest_txt: Vec<String> = rest
                        .iter()
                        .map(|(s, n)| {
                            format!(
                                "{:.1}pt {}자({:.1}%)",
                                *s as f64 / 100.0,
                                n,
                                *n as f64 * 100.0 / para_total as f64
                            )
                        })
                        .collect();
                    findings.push(Finding {
                        level: "warn",
                        code: "char-size-mixed",
                        location: loc.clone(),
                        message: format!(
                            "문단 {}자 중 대부분은 {:.1}pt({}자)인데 {} 가 섞였다 — 글자 수를 줄일 때 꼬리 몇 자가 옛 크기로 남은 자국이다 (문단 전체에 크기를 다시 걸어라)",
                            para_total,
                            dom_size as f64 / 100.0,
                            dom_n,
                            rest_txt.join(", ")
                        ),
                    });
                }
            }
        }

        scan.chars = total;

        // char-color-placeholder — 문서 전체 색 글자 비율
        if total >= MIN_DOC_CHARS && colored as f64 / total as f64 >= COLOR_RATIO {
            by_loc.sort_by(|a, b| b.1.cmp(&a.1));
            let top_loc: Vec<String> = by_loc.iter().take(3).map(|(l, _)| l.clone()).collect();
            let mut colors: Vec<(String, usize)> = by_color.into_iter().collect();
            colors.sort_by(|a, b| b.1.cmp(&a.1));
            let color_txt: Vec<String> = colors
                .iter()
                .take(3)
                .map(|(c, n)| format!("{} {}자", c, n))
                .collect();
            findings.push(Finding {
                level: "warn",
                code: "char-color-placeholder",
                location: format!(
                    "{}{}",
                    top_loc.join(", "),
                    if by_loc.len() > 3 {
                        format!(" 외 {}곳", by_loc.len() - 3)
                    } else {
                        String::new()
                    }
                ),
                message: format!(
                    "검정이 아닌 글자가 {}자 / 전체 {}자 = {:.1}% ({}) — 배포 양식 안내문(파란 12pt)의 글자모양을 물려받은 본문이 남아 있다 (해당 범위에 검정을 다시 걸어라)",
                    colored,
                    total,
                    colored as f64 * 100.0 / total as f64,
                    color_txt.join(", ")
                ),
            });
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
    fn lint_layout(&self, findings: &mut Vec<Finding>, scan: &mut LintScan) {
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
        if pages == 0 {
            return;
        }
        let mut per_page: Vec<(Option<f64>, std::collections::HashMap<Key, f64>)> =
            Vec::with_capacity(pages);
        let mut seen_overflow: std::collections::HashSet<String> = std::collections::HashSet::new();
        for p in 0..pages {
            let tree = match self.build_page_tree_cached(p as u32) {
                Ok(t) => t,
                Err(_) => return, // 레이아웃을 못 돌리면 이 검사만 건너뛴다
            };
            let mut bottom = None;
            let mut tables = std::collections::HashMap::new();
            collect(&tree.root, &mut bottom, &mut tables, false);
            Self::lint_cell_overflow(
                &tree.root,
                p,
                "표",
                None,
                None,
                findings,
                &mut seen_overflow,
                scan,
            );
            scan.pages += 1;
            per_page.push((bottom, tables));
        }
        if pages < 2 {
            return;
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

    /// 셀 안 글자가 셀 밖으로 삐져나갔는지 검사한다.
    ///
    /// 왜 필요한가. **한컴은 셀이 좁으면 줄바꿈이 아니라 가로로 잘라낸다.**
    /// 끊을 자리가 없는 덩어리(영문·괄호·슬래시가 붙은 토큰)는 다음 줄로
    /// 넘어가지 못하고 셀 밖으로 나가며, 화면에서는 셀 경계에서 잘린 채
    /// 보인다. 2026-07-31 실측: 「표준 인터페이스(」, 「한글·오피스 문」,
    /// 「폐쇄망 구동 실」 세 곳이 그렇게 잘렸는데 저장·라운드트립·기존 lint
    /// 를 전부 통과했다. 사람 눈 말고는 잡을 수단이 없었다.
    ///
    /// 경계는 **줄(TextLine) 상자의 오른쪽 끝**을 쓴다. 줄 상자는 셀 안쪽
    /// 여백(실측 510 HWPU = 6.8px)을 이미 뺀 콘텐츠 폭이라 여백을 따로
    /// 계산할 필요가 없다. 줄이 없으면 셀 상자 오른쪽 끝으로 대신한다.
    #[allow(clippy::too_many_arguments)]
    fn lint_cell_overflow(
        node: &crate::renderer::render_tree::RenderNode,
        page: usize,
        table_loc: &str,
        cell: Option<(u16, u16, f64)>,
        line_right: Option<f64>,
        findings: &mut Vec<Finding>,
        seen: &mut std::collections::HashSet<String>,
        scan: &mut LintScan,
    ) {
        use crate::renderer::render_tree::RenderNodeType;

        /// 넘쳤다고 볼 최소 여유(px). 아래 자폭 기준의 바닥값이다.
        ///
        /// 0 으로 두면 **정상 문서를 오탐한다.** 배포 문서 11쪽을 훑어보니
        /// 양쪽정렬 줄의 마지막 런이 매번 정확히 0.6px 씩 경계를 넘었다
        /// (줄 폭을 채우고 남은 반올림).
        const SLACK_PX: f64 = 2.0;

        let mut table_loc = table_loc.to_string();
        let mut cell = cell;
        let mut line_right = line_right;

        match node.node_type {
            RenderNodeType::Table(ref t) => {
                // 중첩 표는 (section, para, control) 이 비어 있다 — 바깥 표
                // 위치에 「중첩표」를 덧붙여 어디인지 알아볼 수 있게 한다.
                table_loc = match (t.section_index, t.para_index, t.control_index) {
                    (Some(s), Some(p), Some(c)) => format!("sec{} para{} ctrl{}", s, p, c),
                    _ => format!("{} 중첩표", table_loc),
                };
            }
            RenderNodeType::TableCell(ref c) => {
                scan.cells += 1;
                cell = Some((c.row, c.col, node.bbox.x + node.bbox.width));
                line_right = None; // 셀이 바뀌면 바깥 줄 상자는 더 이상 경계가 아니다
            }
            RenderNodeType::TextLine(_) => {
                line_right = Some(node.bbox.x + node.bbox.width);
            }
            RenderNodeType::TextRun(ref run) => {
                scan.text_runs += 1;
                if let Some((row, col, cell_right)) = cell {
                    // 공백만 있는 런은 경계를 넘어도 눈에 안 보인다. 실측:
                    // 정상 문서에서 빈 런이 17.3px, 공백 한 칸 런이 7.9px
                    // 밖에 나와 있었다 — 이걸 세면 오탐만 쌓인다.
                    if !run.text.trim().is_empty() {
                        let n = run.text.chars().count().max(1);
                        let trail = run
                            .text
                            .chars()
                            .rev()
                            .take_while(|c| c.is_whitespace())
                            .count();
                        // 줄 끝 공백도 안 보이므로 평균 자폭만큼 빼고 잰다
                        let advance = node.bbox.width / n as f64;
                        let right = node.bbox.x + node.bbox.width - advance * trail as f64;
                        let limit = match line_right {
                            Some(l) => l.min(cell_right),
                            None => cell_right,
                        };
                        // **글자 한 자가 통째로 밖에 밀려났을 때만 결함이다.**
                        // 고정값 2px 로 재 봤더니 337개 문서에서 1,426건이
                        // 걸렸는데 그 중 852건(60%)이 10px 미만 — 한 글자
                        // (12pt 한글 ≈ 16px)의 절반도 안 되는, 줄 끝 공백이
                        // 매달린 자국이었다. 자폭을 문턱으로 쓰면 글꼴 크기에
                        // 따라 문턱이 같이 움직여 이 잡음이 사라진다.
                        let over = right - limit;
                        if over > advance.max(SLACK_PX) {
                            let key = format!("{}|r{}c{}|{}", table_loc, row, col, run.text);
                            if seen.insert(key) {
                                let snippet: String = run.text.chars().take(24).collect();
                                findings.push(Finding {
                                    level: "error",
                                    code: "cell-text-overflow",
                                    location: format!(
                                        "{} cell(r{}c{}) p{}",
                                        table_loc,
                                        row,
                                        col,
                                        page + 1
                                    ),
                                    message: format!(
                                        "셀 안 글자가 오른쪽 경계를 {:.1}px 넘는다 — 한컴은 줄바꿈 대신 가로로 잘라내므로 「{}」의 뒷부분이 화면에서 사라진다 (열 폭을 넓히거나 끊을 자리를 넣어라)",
                                        over, snippet
                                    ),
                                });
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        for ch in &node.children {
            Self::lint_cell_overflow(ch, page, &table_loc, cell, line_right, findings, seen, scan);
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

#[cfg(test)]
mod orphan_heading_tests {
    //! 소제목이 칸 끝에 홀로 남는 것을 잡는지 — 그리고 **잡지 말아야 할 것을
    //! 안 잡는지** 함께 본다. 처음 규칙은 "앞이 뒤보다 위 계층이면 표제"였는데
    //! 실제 문서에서 5건 중 4건이 오탐이었다(`-` 잎 항목, 목록 뒤 그림).
    use crate::document_core::DocumentCore;
    use crate::model::control::Control;
    use crate::model::paragraph::Paragraph;
    use crate::model::table::{Cell, Table};

    fn para(text: &str) -> Paragraph {
        Paragraph { text: text.to_string(), ..Default::default() }
    }

    fn cell(row: u16, texts: &[&str]) -> Cell {
        Cell {
            row,
            col: 0,
            col_span: 3,
            row_span: 1,
            paragraphs: texts.iter().map(|t| para(t)).collect(),
            border_fill_id: 1,
            ..Default::default()
        }
    }

    /// 답변 칸 두 개가 위아래로 이어지는 최소 문서.
    fn doc(above: &[&str], below: &[&str]) -> DocumentCore {
        let mut d = DocumentCore::new_empty();
        let mut table = Table::default();
        table.row_count = 2;
        table.col_count = 3;
        table.cells = vec![cell(0, above), cell(1, below)];
        let mut p = Paragraph::default();
        p.controls.push(Control::Table(Box::new(table)));
        d.document.sections.push(Default::default());
        d.document.sections[0].paragraphs.push(p);
        d
    }

    fn orphans(d: &DocumentCore) -> Vec<String> {
        let mut findings = Vec::new();
        let mut scan = super::LintScan::default();
        d.lint_orphan_heading(&mut findings, &mut scan);
        assert_eq!(scan.cell_seams, 1, "이음매를 못 찾았으면 검사가 죽은 것이다");
        findings
            .iter()
            .filter(|f| f.code == "heading-orphaned-at-cell-end")
            .map(|f| f.message.clone())
            .collect()
    }

    #[test]
    fn section_heading_left_alone_is_reported() {
        let d = doc(&["◦ 앞 내용", "□ 문서 호환성 검증 결과"], &["◦ 사내 실무 문서 표본으로 검증함"]);
        let got = orphans(&d);
        assert_eq!(got.len(), 1, "□ 표제가 칸 끝에 혼자 남았는데 안 잡혔다: {got:?}");
        assert!(got[0].contains("문서 호환성"), "{got:?}");
    }

    #[test]
    fn item_heading_whose_children_are_overleaf_is_reported() {
        let d = doc(&["□ 절", "◦ 경쟁 분석"], &["- 코난테크놀로지·솔트룩스", "- 올거나이즈"]);
        assert_eq!(orphans(&d).len(), 1, "◦ 표제의 - 자식이 다음 쪽인데 안 잡혔다");
    }

    #[test]
    fn heading_with_its_content_in_the_same_cell_is_silent() {
        let d = doc(&["□ 절", "◦ 내용이 같은 칸에 있음"], &["□ 다음 절", "◦ 그 내용"]);
        assert!(orphans(&d).is_empty(), "정상인데 잡혔다");
    }

    #[test]
    fn list_continuing_across_the_page_is_silent() {
        let d = doc(&["◦ 항목", "- 첫째"], &["- 둘째", "- 셋째"]);
        assert!(orphans(&d).is_empty(), "쪽을 넘어 이어지는 목록은 표제가 아니다");
    }

    #[test]
    fn leaf_item_followed_by_a_figure_is_silent() {
        // 실제 오탐 사례: `-` 목록이 끝나고 다음 쪽이 그림으로 시작한다.
        let d = doc(&["◦ 항목", "- 타깃은 업종명이 아니라 구조임"], &[" ", "[그림] 업무 화면 구성안"]);
        assert!(orphans(&d).is_empty(), "잎 항목은 표제가 아니다: {:?}", orphans(&d));
    }

    #[test]
    fn a_long_bullet_sentence_followed_by_prose_is_silent() {
        // `◦` 는 짧은 표제일 수도, 완결된 서술일 수도 있다. 자식(`-`)이 없으면
        // 표제로 단정하지 않는다.
        let d = doc(&["□ 절", "◦ 사내 문서 표본으로 형식별 처리 능력을 검증함"], &["앞에서 본 것처럼 결과는 다음과 같습니다."]);
        assert!(orphans(&d).is_empty(), "자식 없는 ◦ 를 표제로 단정했다");
    }
}

#[cfg(test)]
mod pushed_paragraph_tests {
    //! 앞 칸에 자리가 남는데 뒤 칸으로 밀려간 문단을 잡는지.
    //!
    //! 끌어올려도 **안전한 것만** 짚어야 한다 — 표제를 끌어올려 자식을 두고 오면
    //! 방금 고친 고아를 새로 만드는 셈이고, 표지의 라벨 행처럼 쪽을 가르지 않는
    //! 이음매에 대고 말하면 무의미한 소견이 된다(실측 오탐 「이름」).
    use crate::document_core::DocumentCore;
    use crate::model::control::Control;
    use crate::model::paragraph::{LineSeg, Paragraph};
    use crate::model::table::{Cell, Table};

    const PITCH: i32 = 1860; // 15pt · 줄간격 124%
    const PAGE: u32 = 67_684;

    /// 한 줄짜리 문단. vpos 는 칸 안 누적 좌표다.
    fn line(text: &str, vpos: i32) -> Paragraph {
        Paragraph {
            text: text.to_string(),
            line_segs: vec![LineSeg {
                vertical_pos: vpos,
                line_height: 1500,
                line_spacing: 360,
                ..Default::default()
            }],
            ..Default::default()
        }
    }

    fn cell(row: u16, height: u32, texts: &[&str]) -> Cell {
        Cell {
            row,
            col: 0,
            col_span: 3,
            row_span: 1,
            height,
            paragraphs: texts
                .iter()
                .enumerate()
                .map(|(i, t)| line(t, i as i32 * PITCH))
                .collect(),
            border_fill_id: 1,
            ..Default::default()
        }
    }

    fn doc(above: Cell, below: Cell) -> DocumentCore {
        let mut d = DocumentCore::new_empty();
        let mut table = Table::default();
        table.row_count = 2;
        table.col_count = 3;
        table.cells = vec![above, below];
        let mut p = Paragraph::default();
        p.controls.push(Control::Table(Box::new(table)));
        let mut sec = crate::model::document::Section::default();
        sec.section_def.page_def.height = 84_186;
        sec.section_def.page_def.margin_top = 4_252;
        sec.section_def.page_def.margin_bottom = 4_252;
        sec.section_def.page_def.margin_header = 2_835;
        sec.section_def.page_def.margin_footer = 2_835;
        sec.paragraphs.push(p);
        d.document.sections.push(sec);
        d
    }

    fn pushed(d: &DocumentCore) -> Vec<String> {
        let mut findings = Vec::new();
        let mut scan = super::LintScan::default();
        d.lint_pushed_paragraph(&mut findings, &mut scan);
        findings
            .iter()
            .filter(|f| f.code == "paragraph-pushed-despite-room")
            .map(|f| f.message.clone())
            .collect()
    }

    #[test]
    fn a_leaf_item_that_fits_in_the_room_above_is_reported() {
        // 앞 칸은 쪽만큼 크고 두 줄만 썼다 — 뒤 칸 첫 줄이 넉넉히 들어간다.
        let d = doc(
            cell(0, PAGE, &["◦ 시장 진입 전략", "- 협회·파트너 채널로 무료 베타 5곳 확보"]),
            cell(1, PAGE, &["- 이후 정부 조달·바우처로 확산", "□ 추진 일정"]),
        );
        let got = pushed(&d);
        assert_eq!(got.len(), 1, "앞 쪽이 비었는데 밀려간 문단을 못 잡았다: {got:?}");
        assert!(got[0].contains("정부 조달"), "{got:?}");
    }

    #[test]
    fn no_room_above_is_silent() {
        // 앞 칸이 꽉 찼다 — 끌어올릴 자리가 없다.
        let texts: Vec<String> = (0..36).map(|i| format!("- 줄 {i}")).collect();
        let refs: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();
        let d = doc(cell(0, PAGE, &refs), cell(1, PAGE, &["- 다음 줄", "□ 다음 절"]));
        assert!(pushed(&d).is_empty(), "자리가 없는데 올리라고 했다: {:?}", pushed(&d));
    }

    #[test]
    fn a_block_that_would_empty_the_source_cell_is_silent() {
        // ◦+자식 전부가 덩어리로 올라가면 출발 칸이 비어 버린다.
        let d = doc(
            cell(0, PAGE, &["□ 절", "◦ 앞 항목"]),
            cell(1, PAGE, &["◦ 경쟁 분석", "- 코난테크놀로지", "- 올거나이즈"]),
        );
        assert!(pushed(&d).is_empty(), "출발 칸을 비우는 이동을 권했다");
    }

    #[test]
    fn a_heading_block_that_fits_whole_is_reported() {
        // 표제 덩어리(◦+자식들)가 통째로 들어가면 고아 걱정 없이 올릴 수 있다 —
        // 단일 문단 판정이던 시절 이 모양의 17줄 구멍을 규칙이 못 봤다.
        let d = doc(
            cell(0, PAGE, &["□ 절", "◦ 앞 항목"]),
            cell(1, PAGE, &["◦ 경쟁 분석", "- 코난테크놀로지", "- 올거나이즈", "□ 다음 절"]),
        );
        let got = pushed(&d);
        assert_eq!(got.len(), 1, "통째로 들어가는 표제 덩어리를 못 봤다: {got:?}");
        assert!(got[0].contains("3문단"), "{got:?}");
    }

    #[test]
    fn a_heading_block_too_tall_for_the_room_is_silent() {
        // 표제는 들어가는데 자식까지는 안 들어간다 — 올리면 고아가 된다.
        let texts: Vec<String> = (0..34).map(|i| format!("- 줄 {i}")).collect();
        let mut above: Vec<&str> = vec!["□ 절"];
        above.extend(texts.iter().map(|s| s.as_str()));
        let d = doc(
            cell(0, PAGE, &above),                       // 여유 ~2줄
            cell(1, PAGE, &["◦ 경쟁 분석", "- 코난테크놀로지", "- 올거나이즈", "□ 다음 절"]),
        );
        assert!(pushed(&d).is_empty(), "자식이 못 들어가는 표제 덩어리를 올리라고 했다");
    }

    #[test]
    fn an_object_with_caption_that_fits_is_reported_as_one_unit() {
        // 개체+캡션은 한 몸으로만 움직인다 — 따로 가면 그림·캡션이 쪽으로 갈라진다.
        let mut obj = line(" ", 0);
        obj.controls.push(Control::Table(Box::new(Table::default())));
        let below = Cell {
            row: 1, col: 0, col_span: 3, row_span: 1, height: PAGE,
            paragraphs: vec![obj, line("[그림] 목표 시장 규모", 1860),
                             line("◦ 확인 방법", 3720)],
            border_fill_id: 1,
            ..Default::default()
        };
        let d = doc(cell(0, PAGE, &["□ 절", "◦ 앞 항목"]), below);
        let got = pushed(&d);
        assert_eq!(got.len(), 1, "개체+캡션 덩어리를 못 봤다: {got:?}");
        assert!(got[0].contains("2문단"), "{got:?}");
    }

    #[test]
    fn a_label_row_that_does_not_break_the_page_is_silent() {
        // 표지의 멘토기관/이름 같은 라벨 행 — 위아래로 이어져도 쪽을 안 가른다.
        let d = doc(cell(0, 2_331, &["멘토기관"]), cell(1, 2_331, &["이름"]));
        assert!(pushed(&d).is_empty(), "쪽을 가르지 않는 이음매에 대고 말했다");
    }

    #[test]
    fn a_bare_object_paragraph_is_movable_on_its_own() {
        // 캡션 없는 개체 문단은 홑덩어리로 움직여도 갈라질 짝이 없다.
        let mut below = cell(1, PAGE, &["", "□ 다음 절"]);
        below.paragraphs[0].controls.push(Control::Table(Box::new(Table::default())));
        let d = doc(cell(0, PAGE, &["◦ 앞 항목"]), below);
        assert_eq!(pushed(&d).len(), 1, "캡션 없는 개체 문단을 못 봤다");
    }
}
