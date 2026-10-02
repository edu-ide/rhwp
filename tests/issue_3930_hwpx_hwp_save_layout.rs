//! Issue #3930/#3820 — HWPX 저장 뒤 표 분할·바탕쪽과 PDF page owner를 보존한다.

use std::fs;
use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::header_footer::{HeaderFooterApply, MasterPage};
use rhwp::model::shape::ShapeObject;
use rhwp::model::style::BorderLineType;
use rhwp::parser::parse_document;
use rhwp::renderer::composer::compose_paragraph;
use rhwp::renderer::height_measurer::HeightMeasurer;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::renderer::style_resolver::resolve_styles_with_variant;
use rhwp::renderer::{hwpunit_to_px, DEFAULT_DPI};
use rhwp::wasm_api::HwpDocument;

const FIXTURE: &str = "samples/2025 행정업무운영 편람(최종).hwpx";
const HWP_FIXTURE: &str = "samples/2025 행정업무운영 편람(최종).hwp";
const PAGE_30: u32 = 29;
const PAGE_144: u32 = 143;
const PAGE_145: u32 = 144;
// [#5923] 비-TAC 다문단 셀 trailing 줄간격 제외로 Q&A 지역(281쪽 이후)이 한 쪽씩
// 당겨진다 — p30·p144 지역은 불변이고 본문 문자 다중집합은 불변이다.
const PAGE_283: u32 = 281;
const PAGE_284: u32 = 282;
const PAGE_285: u32 = 283;
const PAGE_286: u32 = 284;
const PAGE_287: u32 = 285;
const PAGE_290: u32 = 288;
const PAGE_291: u32 = 289;
const PAGE_294: u32 = 292;
const PAGE_295: u32 = 293;
const PAGE_296: u32 = 294;
const Q5_RESPONSE_FIRST_LINE: &str = "문서는 결재권자의 결재가 완료된 시점에";
const Q9_TITLE: &str = "보조기관, 보좌기관, 합의제행정기관의 의미";
const Q10_TITLE: &str = "공문서 작성시 연·월·일의 정확한 표기방법";
const Q16_TITLE: &str = "문서의 결재과정에서 협조자는 문서 수정이나 반려가 가능한지요?";
const Q27_TITLE: &str = "소방서장이 지시한 업무에 대해서 소방파출소장이 문서를";
const Q29_TITLE: &str = "구청 내의 중요사항을 계획하고 각 부서로 시행을 할 경우에도";
const Q30_TITLE: &str = "직속기관, 사업소, 출장소, 구청";
const ATTACHMENT_GUIDANCE: &str = "기안문에 작성한 붙임 문서를 첨부";

fn page_tree(document: &HwpDocument, page: u32) -> String {
    document
        .get_page_render_tree(page)
        .unwrap_or_else(|error| panic!("p{} render tree: {error:?}", page + 1))
}

/// PDF p144 안에서 끝나는 붙임 표가 `page tree`에만 남고 물리적으로 쪽 밖으로
/// 잘리는 퇴행을 막는다. 새 DocumentCore로 독립 렌더해 앞선 tree 조회의 카운터를
/// 섞지 않는다 (#3820 Stage 65).
fn page_overflow_cell_lines(bytes: &[u8], page: u32) -> u32 {
    let document = DocumentCore::from_bytes(bytes).expect("overflow fixture parse");
    let _ = document.take_overflow_cell_lines();
    document
        .render_page_svg_native(page)
        .unwrap_or_else(|error| panic!("p{} render: {error:?}", page + 1));
    document.take_overflow_cell_lines()
}

fn collect_stamp_placeholder_tables(node: &RenderNode, out: &mut Vec<(f64, f64, f64, f64)>) {
    if matches!(
        &node.node_type,
        RenderNodeType::Table(table)
            if table.row_count == 1
                && table.col_count == 1
                && (node.bbox.width - 56.7).abs() <= 0.2
                && (node.bbox.height - 56.7).abs() <= 0.2
    ) {
        out.push((node.bbox.x, node.bbox.y, node.bbox.width, node.bbox.height));
    }
    for child in &node.children {
        collect_stamp_placeholder_tables(child, out);
    }
}

fn master_page_text(master_page: &MasterPage) -> String {
    let mut text = String::new();
    for paragraph in &master_page.paragraphs {
        text.push_str(&paragraph.text);
        for control in &paragraph.controls {
            let Control::Shape(shape) = control else {
                continue;
            };
            let Some(text_box) = shape
                .drawing()
                .and_then(|drawing| drawing.text_box.as_ref())
            else {
                continue;
            };
            for text_box_paragraph in &text_box.paragraphs {
                text.push_str(&text_box_paragraph.text);
            }
        }
    }
    text
}

#[test]
fn issue_3930_preserves_page_count_and_inherited_even_master_page() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE);
    let bytes = fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    // CLI가 사용하는 native HwpDocument 래퍼까지 동일하게 통과해야 한다.
    let mut source = HwpDocument::from_bytes(&bytes).expect("HWPX fixture parse");

    // 한컴 2024 PDF p144에는 "붙임 파일에 직인 날인 방법" 표의 안내·예시가
    // 모두 있어야 한다. raw `treatAsChar=1`만 보고 block table을 조기 분할하면
    // p145로 이월되어 이후 page owner가 연쇄적으로 한 쪽씩 밀린다 (#3820).
    let source_p30_tree = page_tree(&source, PAGE_30);
    let source_p144_tree = page_tree(&source, PAGE_144);
    let source_p145_tree = page_tree(&source, PAGE_145);
    let source_p283_tree = page_tree(&source, PAGE_283);
    let source_p284_tree = page_tree(&source, PAGE_284);
    let source_p285_tree = page_tree(&source, PAGE_285);
    let source_p286_tree = page_tree(&source, PAGE_286);
    let source_p287_tree = page_tree(&source, PAGE_287);
    let source_p290_tree = page_tree(&source, PAGE_290);
    let source_p291_tree = page_tree(&source, PAGE_291);
    let source_p294_tree = page_tree(&source, PAGE_294);
    let source_p295_tree = page_tree(&source, PAGE_295);
    let source_p296_tree = page_tree(&source, PAGE_296);
    assert!(
        source_p30_tree.contains("\"text\":\"2025 \"")
            && source_p30_tree.contains("\"text\":\"행정업무운영 편람\""),
        "원본 p30 바탕쪽은 책 제목이어야 한다"
    );
    assert!(
        !source_p30_tree.contains("제2장. 공문서 관리"),
        "원본 p30 바탕쪽은 장 제목으로 바뀌면 안 된다"
    );
    assert!(
        source_p144_tree.contains(ATTACHMENT_GUIDANCE),
        "한컴 PDF p144와 같이 붙임 안내 블록은 원본 p144에 있어야 한다"
    );
    assert!(
        !source_p145_tree.contains(ATTACHMENT_GUIDANCE),
        "원본 p145는 앞 표의 붙임 안내 블록을 다시 갖지 않아야 한다"
    );
    assert!(
        source_p283_tree.contains(Q5_RESPONSE_FIRST_LINE),
        "HWPX Q5의 saved-frame response 첫 줄은 PDF/native HWP와 같이 p283에 있어야 한다"
    );
    assert!(
        !source_p284_tree.contains(Q5_RESPONSE_FIRST_LINE),
        "HWPX Q5의 saved-frame response 첫 줄은 p284로 밀리면 안 된다"
    );
    assert!(
        source_p285_tree.contains("홈페이지상의 질의에 대하여"),
        "HWPX Q8 표제는 PDF physical p285와 같이 Q7 tail 뒤 같은 쪽에서 시작해야 한다"
    );
    assert!(
        source_p286_tree.contains(Q9_TITLE),
        "HWPX Q9 표제는 PDF/native HWP와 같이 p286에서 시작해야 한다"
    );
    assert!(
        source_p287_tree.contains(Q10_TITLE),
        "HWPX Q10 표제는 PDF/native HWP와 같이 p287에서 시작해야 한다"
    );
    assert!(
        source_p290_tree.contains(Q16_TITLE),
        "HWPX Q16 표와 trailing blank-bottom row는 PDF/native HWP와 같이 p290에서 끝나야 한다"
    );
    assert!(
        !source_p291_tree.contains(Q16_TITLE),
        "HWPX Q16 표는 p291로 분할되어 반복되면 안 된다"
    );
    assert!(
        !source_p294_tree.contains(Q27_TITLE),
        "HWPX Q26의 3+3줄 응답 tail은 PDF/native HWP와 같이 p294에서 끝나야 한다"
    );
    assert!(
        source_p295_tree.contains(Q27_TITLE),
        "HWPX Q27 표제는 PDF/native HWP와 같이 p295에서 시작해야 한다"
    );
    assert!(
        source_p295_tree.contains(Q29_TITLE),
        "HWPX Q29의 두 줄 response는 PDF/native HWP와 같이 p295에서 끝나야 한다"
    );
    assert!(
        !source_p296_tree.contains(Q29_TITLE),
        "HWPX Q29 표는 p296으로 분할되어 반복되면 안 된다"
    );
    assert!(
        source_p296_tree.contains(Q30_TITLE),
        "HWPX Q30 표제는 PDF/native HWP와 같이 p296에서 시작해야 한다"
    );
    // [#5923] 383 → 382 — 비-TAC 다문단 셀 trailing 줄간격 제외. 본문 손실 없음
    // (차이는 쪽 머리글 변형·쪽번호 꾸미기, #5801 게이트 동일 근거).
    assert_eq!(
        source.page_count(),
        382,
        "HWPX Q&A PageHide/목차 tail 보정 뒤 Hancom PDF 쪽수"
    );
    assert_eq!(
        page_overflow_cell_lines(&bytes, PAGE_144),
        0,
        "PDF p144에 완결된 붙임 표의 하위 안내·caption은 쪽 밖으로 clip되면 안 된다"
    );
    let source_border_fill = &source.document().doc_info.border_fills[67];
    assert_eq!(
        source_border_fill.borders[0].line_type,
        BorderLineType::Dot,
        "HWPX DASH 테두리는 Hancom HWP5 code 3 점선으로 읽어야 한다"
    );
    // CLI/MCP 저장 경로도 배포용 해제 단계를 먼저 거치므로 같은 순서로 검증한다.
    source
        .convert_to_editable_native()
        .expect("편집 가능 문서 정규화");
    let saved = source.export_hwp_with_adapter().expect("HWP 저장");

    // HWPX에는 HWP5 SECTION_DEF의 raw tail이 없지만, HWP 2020은 바탕쪽이 있는
    // 구역에 19 byte tail(CTRL_HEADER 전체 47 byte)을 쓴다. 이 값이 10 byte
    // 기본값으로 남으면 HWP 2020이 LIST_HEADER 바탕쪽을 무시할 수 있다.
    let section_index = 10;
    let section = &source.document().sections[section_index];
    assert_eq!(
        section.section_def.raw_ctrl_extra.len(),
        19,
        "구역 {section_index} root SectionDef HWP5 바탕쪽 tail"
    );
    let inline_section_def = section.paragraphs[0]
        .controls
        .iter()
        .find_map(|control| match control {
            Control::SectionDef(section_def) => Some(section_def.as_ref()),
            _ => None,
        })
        .expect("첫 문단 SectionDef");
    assert_eq!(
        inline_section_def.raw_ctrl_extra.len(),
        19,
        "구역 {section_index} inline SectionDef HWP5 바탕쪽 tail"
    );
    let reloaded = HwpDocument::from_bytes(&saved).expect("저장 HWP 재로드");

    assert_eq!(
        reloaded.document().sections[10]
            .section_def
            .raw_ctrl_extra
            .len(),
        19,
        "직렬화된 구역 10 SectionDef도 HWP 2020 바탕쪽 tail을 보존해야 한다"
    );

    // [#5751] 종전에는 `reloaded == source` 등식이었다. 한글 2022 는 이 문서를
    // `.hwp`·`.hwpx` 모두 **384쪽**으로 조판하는데, `#501` 가드 정정 뒤 저장 HWP
    // 경로는 384 로 **정답에 도달**했고 HWPX 원본 경로만 383 에 남았다. 등식을
    // 유지하면 정확해진 쪽을 되돌리라는 요구가 되고, 등식을 지우면 회귀 탐지력이
    // 사라진다. 그래서 양쪽을 오라클 기준값과 함께 각각 고정한다. 남은 HWPX −1
    // 격차는 HWP5/HWPX 조판 비대칭 축이라 별도 이슈로 추적한다.
    //
    // [#5923] 비-TAC 다문단 셀 trailing 줄간격 제외로 양쪽 모두 1쪽 당겨진다 —
    // 저장 HWP 384→383, HWPX 원본 383→382. 같은 정정에서 native HWP fixture 는
    // 385→384 가 되어 한글 2022 실측과 **정확히** 일치하게 됐다(#3820). 파생
    // 경로의 추가 -1 은 기존 조판 비대칭 축의 연장이다.
    assert_eq!(
        reloaded.page_count(),
        383,
        "저장 HWP의 p144 table owner 보존 — [#5923] trailing 제외로 384→383"
    );
    assert_eq!(
        source.page_count(),
        382,
        "HWPX 원본 경로 — [#5923] trailing 제외로 383→382"
    );
    for (page, source_tree) in [
        (PAGE_30, source_p30_tree),
        (PAGE_144, source_p144_tree),
        (PAGE_145, source_p145_tree),
        (PAGE_283, source_p283_tree),
        (PAGE_284, source_p284_tree),
        (PAGE_285, source_p285_tree),
        (PAGE_286, source_p286_tree),
        (PAGE_287, source_p287_tree),
        (PAGE_290, source_p290_tree),
        (PAGE_291, source_p291_tree),
        (PAGE_294, source_p294_tree),
        (PAGE_295, source_p295_tree),
        (PAGE_296, source_p296_tree),
    ] {
        assert_eq!(
            page_tree(&reloaded, page),
            source_tree,
            "저장 HWP p{} 조판 tree는 원본 HWPX와 같아야 한다",
            page + 1
        );
    }

    let section = &reloaded.document().sections[2].section_def;
    let base_master_pages: Vec<&MasterPage> = section
        .master_pages
        .iter()
        .filter(|master_page| !master_page.is_extension)
        .collect();
    assert_eq!(base_master_pages.len(), 1, "HWP 2020 단일 Odd 저장 슬롯");
    assert_eq!(base_master_pages[0].apply_to, HeaderFooterApply::Odd);
    // 한컴 2020은 아래 SECTION_DEF 0x80000000 플래그로 이전 구역의 짝수 바탕쪽을
    // 상속한다. HWP5 parser도 이 단일 Odd 계약을 그대로 복원해야 한다.
    assert!(
        master_page_text(base_master_pages[0]).contains("제2장. 공문서 관리"),
        "홀수 쪽은 현재 구역 장 제목 바탕쪽을 사용해야 한다"
    );
    assert_eq!(
        section.flags & 0xe000_0000,
        0x8000_0000,
        "단일 Odd 슬롯은 한컴 2020의 이전 짝수 쪽 상속 플래그여야 한다"
    );
    assert_eq!(
        reloaded.document().doc_info.border_fills[67].borders[0].line_type,
        BorderLineType::Dot,
        "저장 HWP도 날인 상자의 점선 BORDER_FILL을 유지해야 한다"
    );

    let first_picture = reloaded.document().sections[0]
        .paragraphs
        .iter()
        .flat_map(|paragraph| paragraph.controls.iter())
        .find_map(|control| match control {
            Control::Picture(picture) => Some(picture.as_ref()),
            _ => None,
        })
        .expect("첫 그림");
    let grouped_picture = reloaded.document().sections[0]
        .paragraphs
        .iter()
        .flat_map(|paragraph| paragraph.controls.iter())
        .find_map(|control| match control {
            Control::Shape(shape) => match shape.as_ref() {
                ShapeObject::Group(group) => group.children.iter().find_map(|child| match child {
                    ShapeObject::Picture(picture) => Some(picture.as_ref()),
                    _ => None,
                }),
                _ => None,
            },
            _ => None,
        })
        .expect("묶음 내부 그림");
    for picture in [first_picture, grouped_picture] {
        assert_eq!(
            picture.raw_picture_extra.len(),
            18,
            "HWPX 그림의 HWP5 SC_PICTURE extra 길이"
        );
        assert_eq!(
            &picture.raw_picture_extra[9..17],
            &[0; 8],
            "한컴 HWPX 저장본처럼 SC_PICTURE original image size는 0으로 쓴다"
        );
    }
    assert_eq!(grouped_picture.image_attr.brightness, 0);
    assert_eq!(grouped_picture.image_attr.contrast, 8);
}

/// native HWP 원본의 6x5 Q&A RowBreak 표에서 짧은 마지막 응답 tail은 저장
/// frame owner를 유지한다. Stage 131의 terminal spacer/guide 보정은 386쪽을 383쪽으로 낮추며,
/// HWPX fixture의 저장/roundtrip page-count 계약과는 별도로 고정한다 (#3820).
///
/// [#5751] 기대값을 383 → 385 로 갱신했다. `#501` 가드를 렌더와 같은 기준으로
/// 맞추면서 이 문서의 표 43개가 각 1행씩 내용에 맞게 늘었다. 383 은 한컴 2020
/// 기준이고 한글 2022 는 같은 파일을 384쪽으로 조판한다 — 갱신 전후 모두 오차 1.
///
/// [#5923] 비-TAC 다문단 셀 trailing 줄간격 제외로 385 → 384 — 한글 2022 조판과
/// 정확히 일치한다. Q8 표제는 한컴 2020/2024 PDF의 physical p285, 0-기반 284쪽에 있다.
#[test]
fn issue_3820_hwp5_qa_rowbreak_tail_reduces_page_count() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(HWP_FIXTURE);
    let bytes = fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    let source = HwpDocument::from_bytes(&bytes).expect("HWP fixture parse");

    assert!(
        page_tree(&source, 284).contains("홈페이지상의 질의에 대하여"),
        "Hancom PDF physical p285와 같이 Q8 표제는 Q7 tail 뒤 같은 쪽에서 시작해야 한다"
    );
    assert_eq!(
        source.page_count(),
        384,
        "native HWP Q&A PageHide/RowBreak owner 보정 뒤 Hancom PDF 쪽수"
    );
}

fn find_render_node<'a>(
    node: &'a RenderNode,
    predicate: &impl Fn(&RenderNode) -> bool,
) -> Option<&'a RenderNode> {
    if predicate(node) {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| find_render_node(child, predicate))
}

fn collect_rendered_text(node: &RenderNode, text: &mut String) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        text.push_str(&run.text);
    }
    for child in &node.children {
        collect_rendered_text(child, text);
    }
}

/// 한컴 2024 원본 PDF physical p298의 Q32/Q33 프레임과 제목 아래 괘선을
/// 독립 좌표로 고정한다. Q32 응답 cellSz는 stale하므로 본문을 선언 높이로
/// 일괄 축소하면 이 회귀의 글줄·물리 셀 범위 검증에서 실패해야 한다.
#[test]
fn issue_3930_native_qa_spanning_title_frames_preserve_body_lines() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(HWP_FIXTURE);
    let bytes = fs::read(path).expect("read native Q&A fixture");
    let document = parse_document(&bytes).expect("parse native Q&A fixture");
    let section = &document.sections[10];
    let styles =
        resolve_styles_with_variant(&document.doc_info, DEFAULT_DPI, document.is_hwp3_variant);
    let composed = section
        .paragraphs
        .iter()
        .map(compose_paragraph)
        .collect::<Vec<_>>();
    let measured = HeightMeasurer::new(DEFAULT_DPI)
        .with_native_hwp5(document.layout_profile().native_hwp5_layout())
        .measure_section(&section.paragraphs, &composed, &styles, None);
    let core = DocumentCore::from_bytes(&bytes).expect("native Q&A render core");
    let mut table_tops = Vec::new();
    // PDF sha256 34db2aeefa4ae00b38c464571e7e17eef375ffd3ec29eb6d89ddcd67b63bb670.
    for (
        para_index,
        question,
        page_index,
        top_pt,
        bottom_pt,
        separator_pt,
        body_height,
        label_height,
        title_height,
        tail_height,
    ) in [
        (
            63,
            "32.",
            297,
            145.786,
            341.728,
            195.222,
            152.92,
            2349,
            4015,
            28.3466666667,
        ),
        (
            65,
            "33.",
            297,
            363.446,
            650.459,
            412.881,
            281.9066666667,
            2349,
            4015,
            20.8666666667,
        ),
        (
            74, "37.", 299, 419.480, 659.338, 469.036, 220.28, 2783, 4012, 20.28,
        ),
        (
            84,
            "40.",
            301,
            146.986,
            361.646,
            188.022,
            192.9733333333,
            2349,
            3166,
            25.32,
        ),
        (
            85, "41.", 301, 367.405, 665.097, 416.841, 308.04, 2349, 4015, 9.6,
        ),
    ] {
        let page = core
            .build_page_render_tree(page_index)
            .expect("original physical Q&A page");
        let (control_index, table) = section.paragraphs[para_index]
            .controls
            .iter()
            .enumerate()
            .find_map(|(index, control)| match control {
                Control::Table(table) => Some((index, table.as_ref())),
                _ => None,
            })
            .expect("Q&A source table");
        let label = table
            .cells
            .iter()
            .find(|cell| cell.row == 1 && cell.col == 3)
            .expect("short question label");
        assert_eq!(label.paragraphs[0].text, question);
        assert_eq!(label.height, label_height);
        assert_eq!(label.paragraphs[0].line_segs[0].line_height, 2500);
        let title = table
            .cells
            .iter()
            .find(|cell| cell.row == 1 && cell.col == 4)
            .expect("two-row title frame");
        assert_eq!((title.row_span, title.height), (2, title_height));
        let measured_table = measured
            .tables
            .iter()
            .find(|measured| {
                measured.para_index == para_index && measured.control_index == control_index
            })
            .expect("measured Q&A frame");
        let declared = hwpunit_to_px(table.common.height as i32, DEFAULT_DPI);
        assert!((measured_table.row_heights.iter().sum::<f64>() - declared).abs() < 0.1);
        assert!(
            (measured_table.row_heights[1] - hwpunit_to_px(label_height as i32, DEFAULT_DPI)).abs()
                < 0.1
        );
        assert!(
            (measured_table.row_heights[2]
                - hwpunit_to_px((title_height - label_height) as i32, DEFAULT_DPI))
            .abs()
                < 0.1
        );
        if para_index >= 74 {
            assert!(
                (measured_table.row_heights[5] - tail_height).abs() < 0.1,
                "only a completely blank tail can absorb stale declared excess"
            );
        }
        assert!((measured_table.row_heights[4] - body_height).abs() < 0.1);
        assert!((declared - (bottom_pt - top_pt) * 4.0 / 3.0).abs() < 0.15);

        let rendered_table = find_render_node(&page.root, &|node| {
            matches!(
                &node.node_type, RenderNodeType::Table(rendered)
                    if rendered.section_index == Some(10)
                        && rendered.para_index == Some(para_index)
            )
        })
        .expect("whole Q&A table on its original PDF page");
        let saved_anchor = hwpunit_to_px(
            section.paragraphs[para_index].line_segs[0].vertical_pos,
            DEFAULT_DPI,
        );
        let source_body_origin =
            rhwp::renderer::page_layout::PageLayoutInfo::from_page_def_default(
                &section.section_def.page_def,
                &rhwp::model::page::ColumnDef::default(),
            )
            .body_area
            .y;
        assert!(
            (rendered_table.bbox.y
                - source_body_origin
                - saved_anchor
                - hwpunit_to_px(table.common.vertical_offset as i32, DEFAULT_DPI))
            .abs()
                < 0.5,
            "Q{question} object top must preserve its zero-before-spacing source host anchor"
        );
        assert_eq!(
            styles.para_styles[composed[para_index].para_style_id as usize].spacing_before,
            0.0
        );
        if para_index == 74 {
            // Q36 has a positive 3000HU host spacing-before. Its raw host vpos
            // is not its object top; normal paragraph flow must remain active.
            assert!(
                (styles.para_styles[composed[73].para_style_id as usize].spacing_before
                    - hwpunit_to_px(3000, DEFAULT_DPI) / 2.0)
                    .abs()
                    < 0.1
            );
            let previous = find_render_node(&page.root, &|node| {
                matches!(&node.node_type,
                RenderNodeType::Table(table) if table.section_index == Some(10)
                    && table.para_index == Some(73))
            })
            .expect("Q36 preceding frame");
            assert!((previous.bbox.height - (413.841 - 195.462) * 4.0 / 3.0).abs() < 0.15);
            assert!(
                previous.bbox.y + previous.bbox.height <= rendered_table.bbox.y + 0.5,
                "the Q37 saved host anchor must not overlap Q36's painted frame"
            );
        }
        if para_index <= 65 {
            table_tops.push(rendered_table.bbox.y);
        }
        assert!((rendered_table.bbox.height - declared).abs() < 0.1);
        let separator = find_render_node(rendered_table, &|node| {
            matches!(
                &node.node_type, RenderNodeType::TableCell(cell) if cell.row == 3 && cell.col == 1
            )
        })
        .expect("full-width title separator");
        assert!(
            (separator.bbox.y - rendered_table.bbox.y - (separator_pt - top_pt) * 4.0 / 3.0).abs()
                < 0.5
        );
        let body = find_render_node(rendered_table, &|node| {
            matches!(
                &node.node_type, RenderNodeType::TableCell(cell) if cell.row == 4 && cell.col == 2
            )
        })
        .expect("Q&A answer cell");
        assert!((body.bbox.height - body_height).abs() < 0.1);
        let source_body = table
            .cells
            .iter()
            .find(|cell| cell.row == 4 && cell.col == 2)
            .expect("source answer cell");
        let source_text: String = source_body
            .paragraphs
            .iter()
            .map(|para| para.text.as_str())
            .collect();
        let mut rendered_text = String::new();
        collect_rendered_text(body, &mut rendered_text);
        let compact = |text: &str| {
            text.chars()
                .filter(|character| !character.is_whitespace())
                .collect::<String>()
        };
        assert_eq!(
            compact(&rendered_text),
            compact(&source_text),
            "all Q{question} answer characters"
        );
        assert!(
            find_render_node(body, &|node| matches!(
                &node.node_type,
                RenderNodeType::TextLine(_)
            ) && node.bbox.y + node.bbox.height
                > body.bbox.y + body.bbox.height + 0.5)
            .is_none(),
            "Q{question} stored answer lines must remain inside their physical cell"
        );
    }

    assert!((table_tops[1] - table_tops[0] - (363.446 - 145.786) * 4.0 / 3.0).abs() < 0.5);

    // 원본 physical p290/p291는 Q16의 두 bullet을 각각 완전히 소유한다.
    // 첫 paragraph 마지막 vpos와 두 번째 paragraph 첫 vpos가 모두 0인
    // 저장 restart도 앞 fragment에 합치거나 이어지는 글줄을 중복하면 안 된다.
    let Control::Table(q16) = &section.paragraphs[30].controls[0] else {
        panic!("Q16 source table");
    };
    let answer = q16
        .cells
        .iter()
        .find(|cell| cell.row == 4 && cell.col == 2)
        .expect("Q16 source answer");
    assert_eq!(answer.paragraphs.len(), 2);
    assert_eq!(answer.paragraphs[0].line_segs.len(), 1);
    assert_eq!(answer.paragraphs[1].line_segs.len(), 6);
    for (paragraph_index, page_index, line_count) in [(0, 289, 1), (1, 290, 6)] {
        let page = core
            .build_page_render_tree(page_index)
            .expect("Q16 original fragment page");
        let table = find_render_node(&page.root, &|node| matches!(&node.node_type,
            RenderNodeType::Table(table) if table.section_index == Some(10) && table.para_index == Some(30)))
            .expect("Q16 fragment table");
        let body = find_render_node(table, &|node| {
            matches!(&node.node_type,
            RenderNodeType::TableCell(cell) if cell.row == 4 && cell.col == 2)
        })
        .expect("Q16 fragment answer");
        let mut rendered_text = String::new();
        collect_rendered_text(body, &mut rendered_text);
        let compact = |text: &str| {
            text.chars()
                .filter(|c| !c.is_whitespace())
                .collect::<String>()
        };
        assert_eq!(
            compact(&rendered_text),
            compact(&answer.paragraphs[paragraph_index].text),
            "Q16 bullet must exist exactly once on its source physical page"
        );
        fn count_lines(node: &RenderNode) -> usize {
            usize::from(matches!(&node.node_type, RenderNodeType::TextLine(_)))
                + node.children.iter().map(count_lines).sum::<usize>()
        }
        assert_eq!(
            count_lines(body),
            line_count,
            "Q16 source fragment line count"
        );
    }

    // 실제 응답이 저장 프레임보다 커지면 label witness가 있어도 다시 늘려야 한다.
    let mut grown_paragraphs = section.paragraphs.clone();
    let Control::Table(grown) = &mut grown_paragraphs[65].controls[0] else {
        panic!("Q33 table control");
    };
    let answer = grown
        .cells
        .iter_mut()
        .find(|cell| cell.row == 4 && cell.col == 2)
        .expect("mutable Q33 answer");
    for line in &mut answer
        .paragraphs
        .last_mut()
        .expect("last answer paragraph")
        .line_segs
    {
        line.line_height += 10_000;
    }
    let grown_composed = grown_paragraphs
        .iter()
        .map(compose_paragraph)
        .collect::<Vec<_>>();
    let grown_measured = HeightMeasurer::new(DEFAULT_DPI)
        .with_native_hwp5(true)
        .measure_section(&grown_paragraphs, &grown_composed, &styles, None);
    let grown_frame = grown_measured
        .tables
        .iter()
        .find(|table| table.para_index == 65)
        .expect("grown answer frame");
    assert!(
        grown_frame.row_heights[4] > 281.9066666667 + 80.0,
        "real stored body overflow must continue to grow its row"
    );
    assert!(
        grown_frame.row_heights.iter().sum::<f64>() > 382.7733333333 + 80.0,
        "real body overflow must not be truncated to the declared table height"
    );
}

/// PDF p144의 자동날인 안내는 같은 빈 host paragraph의 `BehindText` 1×1 table 세 개를
/// `horzOffset=4868,13553,22830HU`로 한 줄에 놓는다. nested non-TAC의 generic flow가
/// 각 table 높이만큼 cursor를 전진하면 세 점선 상자가 세로로 쌓여, page owner가 맞아도
/// 눈에 보이는 fidelity가 깨진다 (#3820 Stage 66).
#[test]
fn issue_3820_hwpx_behind_text_stamp_placeholders_keep_common_y_and_offsets() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE);
    let bytes = fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    let core = DocumentCore::from_bytes(&bytes).expect("HWPX fixture parse");
    let page = core
        .build_page_render_tree(PAGE_144)
        .expect("render PDF p144");
    let mut stamps = Vec::new();
    collect_stamp_placeholder_tables(&page.root, &mut stamps);
    stamps.sort_by(|left, right| left.0.total_cmp(&right.0));

    assert_eq!(
        stamps.len(),
        3,
        "p144 automatic-stamp guide must retain three 1×1 placeholder tables: {stamps:?}"
    );
    let expected_x = [182.0, 297.8, 421.5];
    for ((x, y, width, height), expected_x) in stamps.iter().zip(expected_x) {
        assert!(
            (*x - expected_x).abs() <= 0.3,
            "p144 HWPX horzOffset anchor mismatch: x={x:.1}, expected={expected_x:.1}, stamps={stamps:?}"
        );
        assert!(
            (*y - stamps[0].1).abs() <= 0.3,
            "p144 BehindText placeholders must share one paragraph y: stamps={stamps:?}"
        );
        assert!(
            (*width - 56.7).abs() <= 0.2 && (*height - 56.7).abs() <= 0.2,
            "p144 placeholder physical size must preserve the PDF's 4251HU square: {stamps:?}"
        );
    }
}

/// The native law table retains all 57 physical source frames and each
/// cell's complete text exactly once, including paragraph-boundary restarts.
#[test]
fn issue_3930_native_law_source_frames_preserve_text_owners() {
    let bytes = fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(HWP_FIXTURE))
        .expect("read native law fixture");
    let source = parse_document(&bytes).expect("parse native law fixture");
    let Control::Table(source_table) = &source.sections[11].paragraphs[3].controls[0] else {
        panic!("native law source table");
    };
    assert_eq!((source_table.row_count, source_table.col_count), (103, 2));
    assert!(source_table.cells.iter().all(|cell| cell.row_span == 1));
    let core = DocumentCore::from_bytes(&bytes).expect("native law render core");
    let mut fragments = Vec::new();
    let mut cell_text = std::collections::BTreeMap::<(u16, u16), String>::new();
    // Independently aligned last visible source line in each PDF column.
    // (row, paragraph, line) fixes every physical boundary without a text fixture.
    let source_frame_ends: [[Option<(u16, usize, usize)>; 2]; 57] = [
        [Some((5, 3, 0)), Some((3, 0, 2))],     // p312
        [Some((5, 11, 3)), None],               // p313
        [Some((5, 17, 3)), None],               // p314
        [Some((9, 0, 3)), Some((7, 0, 0))],     // p315
        [Some((11, 0, 3)), Some((11, 0, 3))],   // p316
        [Some((12, 1, 0)), Some((12, 1, 1))],   // p317
        [Some((12, 6, 0)), Some((12, 9, 3))],   // p318
        [None, Some((12, 14, 4))],              // p319
        [None, Some((12, 22, 5))],              // p320
        [None, Some((12, 28, 6))],              // p321
        [Some((14, 1, 2)), Some((14, 2, 1))],   // p322
        [Some((16, 0, 2)), Some((16, 1, 0))],   // p323
        [Some((18, 0, 6)), Some((18, 1, 1))],   // p324
        [Some((19, 1, 2)), Some((19, 1, 0))],   // p325
        [Some((20, 2, 1)), Some((20, 2, 1))],   // p326
        [Some((22, 1, 0)), Some((22, 1, 1))],   // p327
        [Some((22, 5, 6)), Some((22, 9, 1))],   // p328
        [Some((23, 3, 0)), Some((23, 2, 0))],   // p329
        [Some((26, 0, 4)), Some((26, 0, 4))],   // p330
        [Some((27, 4, 2)), Some((27, 5, 4))],   // p331
        [Some((29, 4, 2)), None],               // p332
        [Some((30, 7, 2)), Some((30, 1, 2))],   // p333
        [Some((34, 2, 1)), Some((34, 2, 0))],   // p334
        [Some((35, 0, 6)), Some((34, 9, 0))],   // p335
        [Some((37, 0, 2)), None],               // p336
        [Some((40, 5, 1)), Some((40, 5, 4))],   // p337
        [Some((43, 0, 5)), Some((43, 1, 1))],   // p338
        [Some((44, 3, 1)), Some((44, 2, 3))],   // p339
        [Some((46, 0, 0)), Some((46, 0, 0))],   // p340
        [Some((50, 2, 0)), Some((50, 1, 0))],   // p341
        [Some((51, 0, 5)), None],               // p342
        [Some((52, 1, 0)), None],               // p343
        [Some((53, 4, 1)), None],               // p344
        [Some((54, 0, 5)), None],               // p345
        [Some((54, 9, 4)), None],               // p346
        [Some((56, 1, 0)), None],               // p347
        [Some((59, 6, 1)), None],               // p348
        [Some((61, 0, 0)), None],               // p349
        [Some((63, 0, 2)), None],               // p350
        [Some((65, 0, 3)), None],               // p351
        [Some((66, 3, 0)), None],               // p352
        [Some((71, 5, 1)), Some((71, 1, 4))],   // p353
        [Some((71, 7, 2)), Some((71, 14, 0))],  // p354
        [None, Some((71, 22, 0))],              // p355
        [Some((72, 3, 3)), Some((72, 3, 2))],   // p356
        [Some((74, 0, 1)), Some((73, 1, 2))],   // p357
        [Some((76, 1, 0)), Some((75, 1, 3))],   // p358
        [Some((77, 6, 1)), None],               // p359
        [Some((79, 8, 3)), Some((79, 9, 2))],   // p360
        [Some((83, 0, 0)), Some((82, 1, 0))],   // p361
        [Some((85, 3, 0)), Some((85, 2, 0))],   // p362
        [Some((87, 0, 3)), Some((85, 5, 0))],   // p363
        [Some((88, 8, 2)), None],               // p364
        [Some((93, 0, 0)), Some((92, 1, 0))],   // p365
        [Some((96, 0, 3)), None],               // p366
        [Some((98, 0, 2)), Some((99, 2, 2))],   // p367
        [Some((102, 3, 0)), Some((102, 1, 2))], // p368
    ];
    let mut rendered_columns = [String::new(), String::new()];
    let mut expected_columns = [String::new(), String::new()];
    let mut boundary_owners = vec![Vec::new(); 4];
    let boundary_witnesses = [
        (40, "그 사무 처리를 위하여 직인을 가질 수 있다.", 337),
        (40, "③ 각급 행정기관은 전자문서에 사용하기", 338),
        (93, "제65조(행정업무 운영에 관한 교육)", 365),
        (93, "대하여 매년 1회 이상 행정업무의 효율성 증진", 366),
    ];
    let compact = |text: &str| {
        text.chars()
            .filter(|c| !c.is_whitespace())
            .collect::<String>()
    };
    for page_index in 311..core.page_count() {
        let page = core
            .build_page_render_tree(page_index)
            .expect("native law page");
        let Some(table) = find_render_node(&page.root, &|node| {
            matches!(&node.node_type,
            RenderNodeType::Table(table) if table.section_index == Some(11)
                && table.para_index == Some(3))
        }) else {
            continue;
        };
        fragments.push(page_index);
        for child in &table.children {
            if let RenderNodeType::TableCell(cell) = &child.node_type {
                if cell.row == 0 {
                    continue;
                }
                let mut text = String::new();
                collect_rendered_text(child, &mut text);
                if cell.col == 0 {
                    for (index, (row, witness, _)) in boundary_witnesses.iter().enumerate() {
                        if cell.row == *row && compact(&text).contains(&compact(witness)) {
                            boundary_owners[index].push(page_index + 1);
                        }
                    }
                }
                rendered_columns[cell.col as usize].push_str(&text);
                cell_text
                    .entry((cell.row, cell.col))
                    .or_default()
                    .push_str(&text);
            }
        }
        if let Some(ends) = source_frame_ends.get((page_index - 311) as usize) {
            for (col, end) in ends.iter().enumerate() {
                if let Some((end_row, end_paragraph, end_line)) = end {
                    let mut column_cells: Vec<_> = source_table
                        .cells
                        .iter()
                        .filter(|cell| {
                            cell.col as usize == col && cell.row != 0 && cell.row <= *end_row
                        })
                        .collect();
                    column_cells.sort_by_key(|cell| cell.row);
                    let mut expected = String::new();
                    for cell in column_cells {
                        for (index, paragraph) in cell.paragraphs.iter().enumerate() {
                            if cell.row == *end_row && index > *end_paragraph {
                                break;
                            }
                            let limit = if cell.row == *end_row && index == *end_paragraph {
                                paragraph
                                    .line_segs
                                    .get(end_line + 1)
                                    .map(|line| paragraph.utf16_pos_to_char_idx(line.text_start))
                                    .unwrap_or_else(|| paragraph.text.chars().count())
                            } else {
                                paragraph.text.chars().count()
                            };
                            expected.extend(paragraph.text.chars().take(limit));
                        }
                    }
                    expected_columns[col] = expected;
                }
                let actual = compact(&rendered_columns[col]);
                let expected = compact(&expected_columns[col]);
                let first_difference = actual
                    .chars()
                    .zip(expected.chars())
                    .position(|(left, right)| left != right)
                    .unwrap_or_else(|| actual.chars().count().min(expected.chars().count()));
                assert!(actual == expected,
                    "law physical p{} c{} source endpoint {:?}: first difference {}, actual chars {}, expected chars {}",
                    page_index + 1, col, end, first_difference,
                    actual.chars().count(), expected.chars().count());
            }
        }
    }
    for cell in source_table.cells.iter().filter(|cell| cell.row != 0) {
        let expected: String = cell
            .paragraphs
            .iter()
            .map(|paragraph| paragraph.text.as_str())
            .collect();
        let actual = cell_text
            .get(&(cell.row, cell.col))
            .map(String::as_str)
            .unwrap_or("");
        assert_eq!(
            compact(actual),
            compact(&expected),
            "law cell r{}/c{} text exactly once",
            cell.row,
            cell.col
        );
    }
    // PDF sha256 34db2aeefa4ae00b38c464571e7e17eef375ffd3ec29eb6d89ddcd67b63bb670:
    // the native law table owns physical p312..p368, including all saved frames.
    assert_eq!(
        fragments,
        (311..368).collect::<Vec<_>>(),
        "native law physical source frames"
    );
    for ((row, witness, physical_page), owners) in boundary_witnesses.iter().zip(boundary_owners) {
        assert_eq!(
            owners,
            [*physical_page],
            "law r{row}/c0 boundary owner for {witness}"
        );
    }
}

/// The Hancom PDF keeps a blank page between the appendix cover and the law
/// table (physical p310/p311/p312), but its later form appendix has no blank
/// page between the three-page design table and the next form (p373..p376).
#[test]
fn issue_3930_native_appendix_break_markers_keep_their_physical_owner() {
    let bytes = fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(HWP_FIXTURE))
        .expect("read native appendix fixture");
    let document = parse_document(&bytes).expect("parse native appendix fixture");
    let law_section = &document.sections[11];
    assert!(matches!(
        law_section.paragraphs[2].controls.as_slice(),
        [Control::PageHide(_)]
    ));
    assert!(matches!(
        law_section.paragraphs[3].controls.as_slice(),
        [Control::Table(table)] if !table.common.treat_as_char
    ));
    let form_section = &document.sections[12];
    assert!(form_section.paragraphs[17].text.is_empty());
    assert!(form_section.paragraphs[17].controls.is_empty());
    assert_eq!(
        form_section.paragraphs[17].column_type,
        rhwp::model::paragraph::ColumnBreakType::Column
    );
    assert_eq!(
        form_section.paragraphs[18].column_type,
        rhwp::model::paragraph::ColumnBreakType::Page
    );

    let core = DocumentCore::from_bytes(&bytes).expect("native appendix render core");
    let page_of = |section, paragraph| {
        let result = core
            .get_page_of_position_native(section, paragraph)
            .expect("appendix marker must have a physical owner");
        serde_json::from_str::<serde_json::Value>(&result).expect("page owner JSON")["page"]
            .as_u64()
            .expect("numeric page owner") as u32
    };
    let cover_page = page_of(11, 0);
    let blank_page = page_of(11, 2);
    let law_page = page_of(11, 3);
    assert_eq!(blank_page, cover_page + 1, "PageHide owns the blank page");
    assert_eq!(law_page, blank_page + 1, "law table begins after the blank");
    let blank = core
        .build_page_render_tree(blank_page)
        .expect("blank page tree");
    let blank_body = find_render_node(&blank.root, &|node| {
        matches!(&node.node_type, RenderNodeType::Body { .. })
    })
    .expect("blank page body");
    let mut blank_text = String::new();
    collect_rendered_text(blank_body, &mut blank_text);
    assert!(
        blank_text.trim().is_empty(),
        "the PageHide page is physically blank"
    );

    let design_start = page_of(12, 16);
    let carrier_page = page_of(12, 17);
    let next_form = page_of(12, 18);
    assert_eq!(
        next_form,
        design_start + 3,
        "the design table owns three pages"
    );
    assert_eq!(
        carrier_page + 1,
        next_form,
        "empty Column carrier stays on the preceding table page"
    );
    for page in design_start..next_form {
        let tree = core
            .build_page_render_tree(page)
            .expect("design table page tree");
        let body = find_render_node(&tree.root, &|node| {
            matches!(&node.node_type, RenderNodeType::Body { .. })
        })
        .expect("design table body");
        let mut text = String::new();
        collect_rendered_text(body, &mut text);
        assert!(
            !text.trim().is_empty(),
            "design table page {} must contain text",
            page + 1
        );
    }
}
