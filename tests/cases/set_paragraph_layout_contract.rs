//! Paragraph and cell replacement must preserve stored layout and style runs.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::paragraph::{CharShapeRef, LineSeg, Paragraph};
use rhwp::model::table::{Cell, Table};

const TITLE: &str = "앞😀제목 뒤";
const CHANGED_TITLE: &str = "앞😀표목 뒤";
const STORED_TABLE_GAP: i32 = 66_096;

fn text_paragraph(text: &str, vertical_pos: i32) -> Paragraph {
    let mut paragraph = Paragraph::new_empty();
    paragraph.text = text.to_string();
    paragraph.char_count = text.encode_utf16().count() as u32;
    let mut offset = 0;
    paragraph.char_offsets = text
        .chars()
        .map(|ch| {
            let start = offset;
            offset += ch.len_utf16() as u32;
            start
        })
        .collect();
    paragraph.char_shapes = vec![CharShapeRef {
        start_pos: 0,
        char_shape_id: 0,
    }];
    paragraph.line_segs = vec![LineSeg {
        text_start: 0,
        vertical_pos,
        line_height: 1_000,
        text_height: 1_000,
        baseline_distance: 850,
        line_spacing: 600,
        column_start: 0,
        segment_width: 48_000,
        tag: LineSeg::TAG_SINGLE_SEGMENT_LINE,
    }];
    paragraph
}

fn core_with_stored_table_gap() -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    let mut document = core.document().clone();
    document.doc_info.raw_stream_dirty = true;
    document.sections[0].raw_stream = None;
    document.doc_info.char_shapes[0].raw_data = None;
    document.doc_info.char_shapes[0].base_size = 1_000;
    let second_shape = document.doc_info.char_shapes[0].clone();
    document.doc_info.char_shapes.push(second_shape);

    let mut title = text_paragraph(TITLE, 1_000);
    title.char_shapes.push(CharShapeRef {
        start_pos: 3, // The preceding emoji takes two UTF-16 code units.
        char_shape_id: 1,
    });
    let following_start = flow_end(&title) + STORED_TABLE_GAP;
    let mut cell_paragraph = text_paragraph(TITLE, 0);
    cell_paragraph.char_shapes = title.char_shapes.clone();
    let mut table = Table {
        row_count: 1,
        col_count: 1,
        row_sizes: vec![1],
        cells: vec![Cell {
            col_span: 1,
            row_span: 1,
            width: 48_000,
            height: 65_000,
            paragraphs: vec![cell_paragraph],
            ..Default::default()
        }],
        ..Default::default()
    };
    table.common.width = 48_000;
    table.common.height = 65_000;
    table.rebuild_grid();
    let mut table_anchor = Paragraph::new_empty();
    table_anchor.line_segs.clear();
    table_anchor.controls.push(Control::Table(Box::new(table)));
    document.sections[0].paragraphs = vec![
        title,
        table_anchor,
        text_paragraph("다음 글", following_start),
        text_paragraph("이어지는 글", following_start + 1_600),
        text_paragraph("새 쪽 글", 0),
    ];
    let page = &mut document.sections[0].section_def.page_def;
    page.width = 50_000;
    page.height = 100_000;
    page.margin_left = 1_000;
    page.margin_right = 1_000;
    page.margin_top = 1_000;
    page.margin_bottom = 1_000;
    core.set_document(document);
    core
}

fn flow_end(paragraph: &Paragraph) -> i32 {
    let segment = paragraph.line_segs.last().unwrap();
    segment.vertical_pos + segment.line_height.min(segment.text_height) + segment.line_spacing
}

fn starts(core: &DocumentCore) -> Vec<Option<i32>> {
    core.document().sections[0]
        .paragraphs
        .iter()
        .map(|paragraph| paragraph.line_segs.first().map(|seg| seg.vertical_pos))
        .collect()
}

fn shapes(core: &DocumentCore) -> Vec<(u32, u32)> {
    core.document().sections[0].paragraphs[0]
        .char_shapes
        .iter()
        .map(|shape| (shape.start_pos, shape.char_shape_id))
        .collect()
}

#[test]
fn equal_width_edit_keeps_table_gap_style_boundary_and_page_reset() {
    let mut core = core_with_stored_table_gap();
    let before = starts(&core);
    let before_shapes = shapes(&core);
    let before_end = flow_end(&core.document().sections[0].paragraphs[0]);
    core.replace_body_text_native(0, 0, 2, 1, "표").unwrap();
    let after_end = flow_end(&core.document().sections[0].paragraphs[0]);
    let after = starts(&core);
    assert_eq!(
        core.document().sections[0].paragraphs[0].text,
        CHANGED_TITLE
    );
    assert_eq!(shapes(&core), before_shapes);
    assert_eq!(after[2].unwrap() - after_end, STORED_TABLE_GAP);
    assert_eq!(
        after[3].unwrap() - before[3].unwrap(),
        after_end - before_end
    );
    assert_eq!(after[4], before[4], "stored page reset must remain fixed");
    assert!(core.document().sections[0].paragraphs[1]
        .line_segs
        .is_empty());
}

#[test]
fn growing_title_carries_only_actual_height_change_across_table() {
    let mut core = core_with_stored_table_gap();
    let before = starts(&core);
    let before_end = flow_end(&core.document().sections[0].paragraphs[0]);
    core.replace_body_text_native(0, 0, 2, 1, &"글".repeat(200))
        .unwrap();
    let after_end = flow_end(&core.document().sections[0].paragraphs[0]);
    let after = starts(&core);
    assert!(after_end > before_end, "the edited title must gain lines");
    assert_eq!(after[2].unwrap() - after_end, STORED_TABLE_GAP);
    assert_eq!(
        after[3].unwrap() - before[3].unwrap(),
        after_end - before_end
    );
    assert_eq!(after[4], before[4]);
}

#[test]
fn editing_immediately_after_table_keeps_its_stored_start() {
    let mut core = core_with_stored_table_gap();
    let before = starts(&core);
    core.replace_body_text_native(0, 2, 0, 1, "뒤").unwrap();
    assert_eq!(starts(&core)[2], before[2]);
}

struct CliFiles {
    source: PathBuf,
    output: PathBuf,
}

impl CliFiles {
    fn new() -> Self {
        let tag = format!(
            "rhwp-paragraph-layout-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        Self {
            source: std::env::temp_dir().join(format!("{tag}-source.hwp")),
            output: std::env::temp_dir().join(format!("{tag}-output.hwp")),
        }
    }
}

impl Drop for CliFiles {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.source);
        let _ = std::fs::remove_file(&self.output);
    }
}

fn set_title(source: &Path, output: &Path, text: &str) {
    let binary = std::env::var("CARGO_BIN_EXE_rhwp")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string());
    let result = Command::new(binary)
        .args([
            "set-paragraph",
            source.to_str().unwrap(),
            "--section",
            "0",
            "--para",
            "0",
            "--text",
            text,
            "-o",
            output.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(result.status.success(), "{result:?}");
}

fn set_cell(source: &Path, output: &Path, text: &str, by_position: bool) {
    let binary = std::env::var("CARGO_BIN_EXE_rhwp")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string());
    let mut command = Command::new(binary);
    command.args([
        "set-cell-text",
        source.to_str().unwrap(),
        "--para",
        "1",
        "--ctrl",
        "0",
        "--cell-para",
        "0",
        "--text",
        text,
        "-o",
        output.to_str().unwrap(),
    ]);
    if by_position {
        command.args(["--row", "0", "--col", "0"]);
    } else {
        command.args(["--cell", "0"]);
    }
    let result = command.output().unwrap();
    assert!(result.status.success(), "{result:?}");
}

fn cell_paragraph(core: &DocumentCore) -> &Paragraph {
    let Control::Table(table) = &core.document().sections[0].paragraphs[1].controls[0] else {
        panic!("fixture must contain a table");
    };
    &table.cells[0].paragraphs[0]
}

#[test]
fn cli_cell_same_text_preserves_the_entire_hwp_byte_stream() {
    let mut core = core_with_stored_table_gap();
    let source = core.export_hwp_native().unwrap();
    let parsed = DocumentCore::from_bytes(&source).unwrap();
    let text = &cell_paragraph(&parsed).text;
    for by_position in [false, true] {
        let files = CliFiles::new();
        std::fs::write(&files.source, &source).unwrap();
        set_cell(&files.source, &files.output, text, by_position);
        assert!(std::fs::read(&files.output).unwrap() == source);
    }
}

#[test]
fn cli_cell_unicode_replacement_preserves_styles_and_body_layout() {
    let mut core = core_with_stored_table_gap();
    let source = core.export_hwp_native().unwrap();
    let before = DocumentCore::from_bytes(&source).unwrap();
    let before_shapes: Vec<_> = cell_paragraph(&before)
        .char_shapes
        .iter()
        .map(|shape| (shape.start_pos, shape.char_shape_id))
        .collect();
    for by_position in [false, true] {
        let files = CliFiles::new();
        std::fs::write(&files.source, &source).unwrap();
        set_cell(&files.source, &files.output, CHANGED_TITLE, by_position);
        let after = DocumentCore::from_bytes(&std::fs::read(&files.output).unwrap()).unwrap();
        assert_eq!(cell_paragraph(&after).text, CHANGED_TITLE);
        let after_shapes: Vec<_> = cell_paragraph(&after)
            .char_shapes
            .iter()
            .map(|shape| (shape.start_pos, shape.char_shape_id))
            .collect();
        assert_eq!(after_shapes, before_shapes);
        assert_eq!(starts(&after), starts(&before));
        assert_eq!(after.page_count(), before.page_count());
        for (after_para, before_para) in after.document().sections[0]
            .paragraphs
            .iter()
            .zip(&before.document().sections[0].paragraphs)
        {
            assert_eq!(after_para.text, before_para.text);
        }
    }
}

#[test]
fn cli_same_text_preserves_the_entire_hwp_byte_stream() {
    let mut core = core_with_stored_table_gap();
    let source = core.export_hwp_native().unwrap();
    let parsed = DocumentCore::from_bytes(&source).unwrap();
    let text = &parsed.document().sections[0].paragraphs[0].text;
    let files = CliFiles::new();
    std::fs::write(&files.source, &source).unwrap();
    set_title(&files.source, &files.output, text);
    assert!(std::fs::read(&files.output).unwrap() == source);
}

#[test]
fn cli_unicode_replacement_preserves_styles_table_gap_and_other_paragraphs() {
    let mut core = core_with_stored_table_gap();
    let source = core.export_hwp_native().unwrap();
    let before = DocumentCore::from_bytes(&source).unwrap();
    let files = CliFiles::new();
    std::fs::write(&files.source, &source).unwrap();
    set_title(&files.source, &files.output, CHANGED_TITLE);
    let after = DocumentCore::from_bytes(&std::fs::read(&files.output).unwrap()).unwrap();
    assert_eq!(
        after.document().sections[0].paragraphs[0].text,
        CHANGED_TITLE
    );
    assert_eq!(shapes(&after), shapes(&before));
    assert_eq!(starts(&after), starts(&before));
    assert_eq!(after.page_count(), before.page_count());
    for index in 1..before.document().sections[0].paragraphs.len() {
        assert_eq!(
            after.document().sections[0].paragraphs[index].text,
            before.document().sections[0].paragraphs[index].text
        );
    }
}
