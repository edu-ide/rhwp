use super::*;

#[test]
fn paragraph_frame_padding_keeps_all_zero_table_boundary() {
    let cell = Cell {
        padding: Padding {
            left: 141,
            right: 141,
            top: 141,
            bottom: 141,
        },
        apply_inner_margin: false,
        ..Default::default()
    };
    let table_padding = Padding::default();

    let frame = cell.paragraph_frame_padding(&table_padding);
    assert_eq!(
        (frame.left, frame.right, frame.top, frame.bottom),
        (0, 0, 0, 0)
    );
    // 전축0 미지정 폴백은 수직 전용 — 수평은 한글이 진짜 0 으로 쓴다
    // (exam_social p2 한글 2020/2022 인쇄 PDF 실측 + 저장 sw 52/52,
    // mydocs/plans/cell_width_authority.md).
    let paint = cell.effective_padding(&table_padding);
    assert_eq!(
        (paint.left, paint.right, paint.top, paint.bottom),
        (0, 0, 141, 141)
    );
}

#[test]
fn paragraph_frame_owner_width_resolves_a_short_repeated_row_on_the_table_grid() {
    let cell = |row, col, width| Cell {
        row,
        col,
        row_span: 1,
        col_span: 1,
        width,
        ..Default::default()
    };
    let mut table = Table {
        row_count: 3,
        col_count: 2,
        cells: vec![
            cell(0, 0, 22_393),
            cell(0, 1, 22_396),
            cell(1, 0, 22_393),
            cell(1, 1, 22_393),
            cell(2, 0, 22_393),
            cell(2, 1, 22_393),
        ],
        ..Default::default()
    };
    table.common.width = 44_789;

    assert_eq!(table.paragraph_frame_owner_widths()[2..4], [22_393, 22_396]);

    table.local_resize_rows.push(1);
    table.local_resize_cell_widths.push((3, 22_400));
    assert_eq!(table.paragraph_frame_owner_widths()[3], 22_400);
}

#[test]
fn paragraph_frame_owner_widths_handles_the_large_document_table_shape_in_one_pass() {
    const ROWS: u16 = 5_277;
    const COLS: u16 = 10;
    let mut cells = Vec::with_capacity(usize::from(ROWS) * usize::from(COLS));
    for row in 0..ROWS {
        for col in 0..COLS {
            cells.push(Cell {
                row,
                col,
                row_span: 1,
                col_span: 1,
                width: 1_000,
                ..Default::default()
            });
        }
    }
    let mut table = Table {
        row_count: ROWS,
        col_count: COLS,
        cells,
        ..Default::default()
    };
    table.common.width = 10_000;

    let owners = table.paragraph_frame_owner_widths();
    assert_eq!(owners.len(), 52_770);
    assert!(owners.iter().all(|width| *width == 1_000));
}
