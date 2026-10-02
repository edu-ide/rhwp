use super::*;
use crate::model::table::Table;

#[test]
fn test_control_utf16_positions_reconstructs_a_shared_control_gap() {
    // Two controls share the visible-text boundary before B. The raw stream
    // has their distinct 8-unit control slots at 1 and 9 before B at 17.
    let para = Paragraph {
        text: "AB".to_string(),
        char_offsets: vec![0, 17],
        controls: vec![
            Control::Table(Box::<Table>::default()),
            Control::Table(Box::<Table>::default()),
        ],
        ..Default::default()
    };

    assert_eq!(para.control_text_positions(), vec![1, 1]);
    assert_eq!(para.control_utf16_positions(), vec![1, 9]);
}

#[test]
fn shift_for_inline_control_insert_moves_line_starts_too() {
    // [#4347] 줄 시작(`text_start`)도 char_offsets 와 같은 UTF-16 좌표계다. 함께 밀지 않으면
    // 저장된 줄 나눔만 8 만큼 어긋난 채 남고, 그 문단을 다시 조판하는 순간 값이 튄다 —
    // 원인이 삽입이 아닌 곳(그림 배치 토글)에서 찾아진다.
    let mut para = Paragraph {
        text: "0123456789".to_string(),
        char_offsets: (0..10).collect(),
        char_count: 10,
        line_segs: vec![
            LineSeg {
                text_start: 0,
                ..Default::default()
            },
            LineSeg {
                text_start: 4,
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    para.shift_for_inline_control_insert(0);

    // 첫 줄은 문단 시작에 고정한다 — 넣은 컨트롤이 그 줄에 든다.
    assert_eq!(para.line_segs[0].text_start, 0);
    // 뒤 줄은 char_offsets 와 같은 만큼 밀린다.
    assert_eq!(para.line_segs[1].text_start, 12);
    assert_eq!(para.char_offsets[4], 12);
}

#[test]
fn shift_for_inline_control_insert_leaves_earlier_lines_alone() {
    // 삽입 지점 **앞** 줄은 그대로다. 뒤 줄만 밀린다.
    let mut para = Paragraph {
        text: "0123456789".to_string(),
        char_offsets: (0..10).collect(),
        char_count: 10,
        line_segs: vec![
            LineSeg {
                text_start: 0,
                ..Default::default()
            },
            LineSeg {
                text_start: 4,
                ..Default::default()
            },
            LineSeg {
                text_start: 8,
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    para.shift_for_inline_control_insert(6);

    assert_eq!(para.line_segs[0].text_start, 0);
    assert_eq!(para.line_segs[1].text_start, 4);
    assert_eq!(para.line_segs[2].text_start, 16);
}
