//! Issue #1486: HWPX 분할 표 내부 TAC 중첩 표가 오른쪽 밖으로 밀리는 회귀 방지.

use std::fs;
use std::path::Path;

use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/hwpx_sample2.hwpx";
const TARGET_PAGE: u32 = 8; // 9쪽, 0-based

fn find_body_bbox(node: &RenderNode) -> Option<BoundingBox> {
    if matches!(node.node_type, RenderNodeType::Body { .. }) {
        return Some(node.bbox);
    }

    node.children.iter().find_map(find_body_bbox)
}

/// [#4334] 이 표를 식별하는 조건은 원래 "para/control 인덱스가 없는 표"(`is_none()`)
/// 였다. #4334 가 `layout_embedded_table`(중첩 표 host 경로 플러밍 결손)을 고치면서
/// 그 인덱스가 채워지자 후보가 0개가 되어 이 테스트가 깨졌다 — 즉 이 실패 자체가
/// #4334 수정이 실제로 이 표에 적용됐다는 증거였다.
///
/// 실제 문서 구조를 직접 확인해 얻은 값으로 바꿨다(`document_core::DocumentCore`
/// 로 `samples/hwpx_sample2.hwpx` 를 모델 레벨에서 추적):
/// section 0 → paragraph 74(빈 문단) 의 control[0] 이 1×1 "래퍼" 표이고, 그 표의
/// 유일한 셀 안에 29개 문단이 쌓여 있다. 그중 cell 안 문단 인덱스 21번이 TAC
/// (text-as-char) 로 박은 **3행 2열** 중첩 표를 담고 있다 — 이게 렌더 트리에서
/// `para_index=Some(21), control_index=Some(0)` 로 나오는 표다(같은 셀 안에 1×1
/// 표 2개·2행15열 표 1개가 더 있지만 cell 문단 인덱스가 다르다: 2, 9, 12).
/// `doc_path_for_node`(render_tree.rs)가 이 인덱스 쌍을 "재귀 중첩 표"용
/// `derived_table_meta` 유도식(부모 셀 경로의 마지막 두 항목)으로 채운다.
///
/// 인덱스 조건만으로도 이미 유일하지만(같은 셀 안 다른 중첩 표는 para 2/9/12),
/// **기하 조건을 지우지 않고 그대로 남겨** 다른 표가 우연히 같은 인덱스를 갖는
/// 경우(예: 문서가 바뀌어 표가 재배치된 경우)까지 방어한다 — 인덱스와 기하 둘 다
/// 맞아야 통과.
fn collect_issue_1486_tables<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    if let RenderNodeType::Table(table) = &node.node_type {
        let b = &node.bbox;
        if table.section_index == Some(0)
            && table.para_index == Some(21)
            && table.control_index == Some(0)
            && b.y < 220.0
            && b.width > 600.0
            && b.width < 680.0
            && b.height > 100.0
            && b.height < 220.0
        {
            out.push(node);
        }
    }

    for child in &node.children {
        collect_issue_1486_tables(child, out);
    }
}

fn collect_render_tree_text(node: &RenderNode, out: &mut String) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        out.push_str(&run.text);
    }

    for child in &node.children {
        collect_render_tree_text(child, out);
    }
}

fn render_tree_contains_text(node: &RenderNode, needle: &str) -> bool {
    let mut text = String::new();
    collect_render_tree_text(node, &mut text);
    text.contains(needle)
}

fn collect_images<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    if matches!(node.node_type, RenderNodeType::Image(_)) {
        out.push(node);
    }

    for child in &node.children {
        collect_images(child, out);
    }
}

fn find_image_bbox_by_ref(
    node: &RenderNode,
    para_index: usize,
    control_index: usize,
) -> Option<BoundingBox> {
    if let RenderNodeType::Image(image) = &node.node_type {
        if image.para_index == Some(para_index) && image.control_index == Some(control_index) {
            return Some(node.bbox);
        }
    }

    node.children
        .iter()
        .find_map(|child| find_image_bbox_by_ref(child, para_index, control_index))
}

fn collect_tables<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    if matches!(node.node_type, RenderNodeType::Table(_)) {
        out.push(node);
    }

    for child in &node.children {
        collect_tables(child, out);
    }
}

fn collect_text_run_bboxes(node: &RenderNode, needle: &str, out: &mut Vec<BoundingBox>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.text == needle {
            out.push(node.bbox);
        }
    }

    for child in &node.children {
        collect_text_run_bboxes(child, needle, out);
    }
}

#[test]
fn issue_1486_blank_tac_continuation_does_not_repeat_previous_fragment_text_height() {
    for sample in ["samples/hwpx_sample2.hwp", SAMPLE] {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
        let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {sample}: {e}"));
        let doc = rhwp::document_core::DocumentCore::from_bytes(&bytes)
            .unwrap_or_else(|e| panic!("parse {sample}: {e}"));
        assert_eq!(doc.page_count(), 29, "{sample}: source PDF page count");

        let rhwp::model::control::Control::Table(wrapper) =
            &doc.document().sections[0].paragraphs[74].controls[0]
        else {
            panic!("{sample}: expected paragraph 74 wrapper table");
        };
        let host = &wrapper.cells[0].paragraphs[21];
        assert_eq!(host.line_segs.len(), 2, "{sample}: mixed TAC host lines");
        assert_eq!(host.line_segs[1].vertical_pos, 0);
        assert_eq!(host.line_segs[1].line_height, 12080);
        assert_eq!(host.line_segs[1].line_spacing, 600);
        assert_eq!(
            doc.document().doc_info.para_shapes[host.para_shape_id as usize].spacing_before,
            600,
            "{sample}: host paragraph keeps its own leading"
        );
        let rhwp::model::control::Control::Table(nested) = &host.controls[0] else {
            panic!("{sample}: expected TAC table in host paragraph");
        };
        assert!(nested.common.treat_as_char);
        assert_eq!(
            host.line_segs[1].line_height as u32,
            nested.common.height
                + nested.outer_margin_top as u32
                + nested.outer_margin_bottom as u32,
            "{sample}: stored continuation line owns the table outer band"
        );
        let continuation_start = host.line_seg_text_start(1);
        let visible_before_cut: String = host
            .text
            .chars()
            .zip(&host.char_offsets)
            .filter(|(_, offset)| **offset < continuation_start)
            .map(|(character, _)| character)
            .collect();
        let continuation_text: String = host
            .text
            .chars()
            .zip(&host.char_offsets)
            .filter(|(_, offset)| **offset >= continuation_start)
            .map(|(character, _)| character)
            .collect();
        assert!(
            visible_before_cut.contains("[청약신청주택]"),
            "{sample}: text before the TAC continuation: {visible_before_cut:?}"
        );
        assert!(
            continuation_text.trim().is_empty(),
            "{sample}: blank TAC continuation: {continuation_text:?}"
        );

        let previous_page = doc.build_page_render_tree(7).expect("render page 8");
        let continuation_page = doc.build_page_render_tree(8).expect("render page 9");
        assert!(render_tree_contains_text(
            &previous_page.root,
            "[청약신청주택]"
        ));
        let mut headers = Vec::new();
        collect_text_run_bboxes(&continuation_page.root, "조회방법", &mut headers);
        assert_eq!(headers.len(), 1, "{sample}: continuation table header");
        let mut tables = Vec::new();
        collect_issue_1486_tables(&continuation_page.root, &mut tables);
        assert_eq!(tables.len(), 1, "{sample}: continuation table border");
        let table_border_y = tables[0]
            .children
            .iter()
            .filter_map(|child| match &child.node_type {
                RenderNodeType::Line(line)
                    if child.visible
                        && line.style.width > 0.0
                        && (line.y1 - line.y2).abs() <= 0.1
                        && (line.y1 - tables[0].bbox.y).abs() <= 0.6
                        && (line.x1 - line.x2).abs() > tables[0].bbox.width / 4.0 =>
                {
                    Some(line.y1)
                }
                _ => None,
            })
            .min_by(f64::total_cmp)
            .unwrap_or_else(|| panic!("{sample}: missing painted table top border"));
        // 같은 PDF의 표 위 테두리: top=39.077pt.
        let pdf_table_y = 39.077 * 96.0 / 72.0;
        assert!(
            (table_border_y - pdf_table_y).abs() <= 4.0,
            "{sample}: TAC continuation leading: border_y={table_border_y}, PDF_y={pdf_table_y}",
        );
        // 한컴 HWP/HWPX 2024 및 HWPX 2020 PDF 9쪽: top=43.872pt.
        let pdf_header_y = 43.872 * 96.0 / 72.0;
        eprintln!(
            "[issue_1486 continuation] sample={sample} table_y={} border_y={table_border_y} PDF_table_y={pdf_table_y} header_y={} PDF_header_y={pdf_header_y}",
            tables[0].bbox.y,
            headers[0].y,
        );
        assert!(
            (headers[0].y - pdf_header_y).abs() <= 4.0,
            "{sample}: blank TAC host repeated its table-sized line height: header_y={}, PDF_y={pdf_header_y}",
            headers[0].y,
        );
        assert_eq!(
            doc.take_overflow_cell_lines(),
            0,
            "{sample}: following table text must stay inside page 9"
        );
    }
}

#[test]
fn issue_1486_partial_table_tac_nested_table_stays_inside_page_body() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {SAMPLE}: {e}"));

    let tree = doc
        .build_page_render_tree(TARGET_PAGE)
        .unwrap_or_else(|e| panic!("render {SAMPLE} page {}: {e}", TARGET_PAGE + 1));

    let body = find_body_bbox(&tree.root).expect("Body bbox");
    let page_right = tree.root.bbox.x + tree.root.bbox.width;
    let expected_body_right = page_right - body.x;

    let mut candidates = Vec::new();
    collect_issue_1486_tables(&tree.root, &mut candidates);
    assert!(
        !candidates.is_empty(),
        "9쪽 상단의 문제 TAC 중첩 표를 찾지 못함"
    );

    let table = candidates
        .into_iter()
        .min_by(|a, b| a.bbox.y.partial_cmp(&b.bbox.y).unwrap())
        .expect("candidate table");
    let table_right = table.bbox.x + table.bbox.width;

    eprintln!(
        "[issue_1486] page={} body_x={:.2} page_right={:.2} table=[x={:.2} y={:.2} w={:.2} h={:.2}] right={:.2} expected_body_right={:.2}",
        TARGET_PAGE + 1,
        body.x,
        page_right,
        table.bbox.x,
        table.bbox.y,
        table.bbox.width,
        table.bbox.height,
        table_right,
        expected_body_right,
    );

    assert!(
        table.bbox.x < body.x + 120.0,
        "분할 표 내부 TAC 중첩 표가 본문 좌측에서 과도하게 밀림: table_x={:.2}, body_x={:.2}",
        table.bbox.x,
        body.x,
    );
    assert!(
        table_right <= expected_body_right + 1.0,
        "분할 표 내부 TAC 중첩 표가 페이지 본문 오른쪽을 초과함: table_right={:.2}, expected_body_right={:.2}",
        table_right,
        expected_body_right,
    );
}

#[test]
fn issue_1486_terminal_rowbreak_sliver_does_not_push_pdf_page22_content() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {SAMPLE}: {e}"));

    let page22 = doc
        .build_page_render_tree(21)
        .expect("render issue #1486 page 22");
    let page23 = doc
        .build_page_render_tree(22)
        .expect("render issue #1486 page 23");

    assert!(
        render_tree_contains_text(&page22.root, "lisfranc"),
        "한컴 PDF 기준 22쪽 하단의 lisfranc 줄이 rhwp 22쪽에 있어야 함"
    );
    assert!(
        !render_tree_contains_text(&page23.root, "lisfranc"),
        "무가시 RowBreak terminal sliver 때문에 lisfranc 줄이 23쪽으로 밀리면 안 됨"
    );
}

#[test]
fn issue_1486_rowspan_block_tail_stays_on_pdf_page14() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {SAMPLE}: {e}"));

    let page13 = doc
        .build_page_render_tree(12)
        .expect("render issue #1486 page 13");
    let page14 = doc
        .build_page_render_tree(13)
        .expect("render issue #1486 page 14");

    assert!(
        render_tree_contains_text(&page13.root, "아동복지시설"),
        "한컴 PDF 기준 13쪽 끝에는 사회취약 계층 (아) 항목까지 보여야 함"
    );
    assert!(
        !render_tree_contains_text(&page13.root, "제2조제10호"),
        "사회취약 계층 하단 행이 13쪽에서 먼저 소비되면 안 됨"
    );
    assert!(
        render_tree_contains_text(&page14.root, "(자)"),
        "한컴 PDF 기준 14쪽은 사회취약 계층 (자) 항목으로 이어져야 함"
    );
    assert!(
        render_tree_contains_text(&page14.root, "제2조제10호"),
        "국민기초생활 보장법 행은 14쪽에 남아야 함"
    );
}

#[test]
fn issue_1486_page19_nested_square_picture_is_not_page_clipped() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {SAMPLE}: {e}"));

    let page19 = doc
        .build_page_render_tree(18)
        .expect("render issue #1486 page 19");
    let page_bottom = page19.root.bbox.y + page19.root.bbox.height;

    let mut images = Vec::new();
    collect_images(&page19.root, &mut images);
    let diagram = images
        .into_iter()
        .filter(|n| n.bbox.width > 300.0 && n.bbox.height > 90.0)
        .max_by(|a, b| a.bbox.y.partial_cmp(&b.bbox.y).unwrap())
        .expect("19쪽 하단 무허가건축물 확인 절차 그림");
    let diagram_bottom = diagram.bbox.y + diagram.bbox.height;

    assert!(
        diagram_bottom <= page_bottom - 4.0,
        "19쪽 하단 그림이 페이지 아래로 잘리면 안 됨: bottom={diagram_bottom:.2}, page_bottom={page_bottom:.2}"
    );
}

#[test]
fn issue_1486_page13_page_number_keeps_footer_gap() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {SAMPLE}: {e}"));

    let page13 = doc
        .build_page_render_tree(12)
        .expect("render issue #1486 page 13");

    let mut tables = Vec::new();
    collect_tables(&page13.root, &mut tables);
    let bottom_table = tables
        .into_iter()
        .filter(|n| n.bbox.y > 500.0 && n.bbox.width > 700.0)
        .max_by(|a, b| {
            let ab = a.bbox.y + a.bbox.height;
            let bb = b.bbox.y + b.bbox.height;
            ab.partial_cmp(&bb).unwrap()
        })
        .expect("13쪽 하단 배점기준표 조각");
    let table_bottom = bottom_table.bbox.y + bottom_table.bbox.height;

    let mut page_numbers = Vec::new();
    // [#3048] 대시 장식은 한글처럼 번호와 공백 한 칸을 둔다 (`-13-` → `- 13 -`).
    // 이 테스트의 단언은 표 하단과 쪽번호 사이 간격이고, 문자열은 위치 지정용이다.
    collect_text_run_bboxes(&page13.root, "- 13 -", &mut page_numbers);
    let page_number = page_numbers.into_iter().next().expect("13쪽 쪽번호");

    assert!(
        page_number.y - table_bottom >= 12.0,
        "13쪽 하단 표와 쪽번호 사이 간격이 너무 좁음: table_bottom={table_bottom:.2}, page_number_y={:.2}",
        page_number.y
    );
}

#[test]
fn issue_1486_page29_tac_logo_aligns_with_text_line() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {SAMPLE}: {e}"));

    let page29 = doc
        .build_page_render_tree(28)
        .expect("render issue #1486 page 29");
    let logo = find_image_bbox_by_ref(&page29.root, 218, 0).expect("29쪽 LH 로고 TAC 그림 bbox");

    assert!(
        (logo.y - 417.6).abs() <= 4.0,
        "29쪽 LH 로고 y가 한컴 PDF 기준에서 벗어남: y={:.2}, bbox={:?}",
        logo.y,
        logo
    );
    assert!(
        (logo.x - 38.4).abs() <= 3.0 && (logo.width - 115.6).abs() <= 3.0,
        "29쪽 LH 로고 x/폭이 한컴 PDF 기준에서 벗어남: bbox={:?}",
        logo
    );
}
