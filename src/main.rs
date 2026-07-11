use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("--help") | Some("-h") => print_help(),
        Some("--version") | Some("-V") => println!("rhwp v{}", rhwp::version()),
        Some("export-svg") => export_svg(&args[2..]),
        Some("export-render-tree") => export_render_tree(&args[2..]),
        Some("export-png") => export_png(&args[2..]),
        Some("export-pdf") => export_pdf(&args[2..]),
        Some("export-text") => export_text(&args[2..]),
        Some("export-markdown") => export_markdown(&args[2..]),
        Some("info") => show_info(&args[2..]),
        Some("dump") => dump_controls(&args[2..]),
        Some("dump-note-shape") => dump_note_shape(&args[2..]),
        Some("dump-endnote-lines") => dump_endnote_lines(&args[2..]),
        Some("dump-pages") => dump_pages(&args[2..]),
        Some("create-hwp") => create_hwp(&args[2..]),
        Some("replace-text") => replace_text_cli(&args[2..]),
        Some("list-fields") => list_fields_cli(&args[2..]),
        Some("insert-clickhere-field") => insert_clickhere_field_cli(&args[2..]),
        Some("get-field-info") => get_field_info_cli(&args[2..]),
        Some("remove-field") => remove_field_cli(&args[2..]),
        Some("set-field") => set_field_cli(&args[2..]),
        Some("list-forms") => list_forms_cli(&args[2..]),
        Some("list-objects") => list_objects_cli(&args[2..]),
        Some("create-form") => create_form_cli(&args[2..]),
        Some("get-form") => get_form_cli(&args[2..]),
        Some("set-form") => set_form_cli(&args[2..]),
        Some("delete-form") => delete_form_cli(&args[2..]),
        Some("extract-structure") => extract_structure_cli(&args[2..]),
        Some("insert-text") => text_edit_cli(&args[2..], false),
        Some("delete-text") => text_edit_cli(&args[2..], true),
        Some("set-paragraph") => set_paragraph_cli(&args[2..]),
        Some("insert-paragraph") => paragraph_insert_cli(&args[2..]),
        Some("copy-paragraph") => paragraph_copy_cli(&args[2..]),
        Some("copy-paragraph-range") => paragraph_range_copy_cli(&args[2..]),
        Some("split-paragraph") => paragraph_split_cli(&args[2..]),
        Some("merge-paragraph") => paragraph_merge_delete_cli(&args[2..], "merge"),
        Some("delete-paragraph") => paragraph_merge_delete_cli(&args[2..], "delete"),
        Some("insert-page-break") => layout_break_cli(&args[2..], "page"),
        Some("insert-column-break") => layout_break_cli(&args[2..], "column"),
        Some("set-column-def") => set_column_def_cli(&args[2..]),
        Some("insert-new-number") => new_number_cli(&args[2..]),
        Some("get-page-hide") => get_page_hide_cli(&args[2..]),
        Some("set-page-hide") => set_page_hide_cli(&args[2..]),
        Some("list-bookmarks") => list_bookmarks_cli(&args[2..]),
        Some("add-bookmark") => add_bookmark_cli(&args[2..]),
        Some("rename-bookmark") => rename_bookmark_cli(&args[2..]),
        Some("delete-bookmark") => delete_bookmark_cli(&args[2..]),
        Some("create-footnote") => note_create_cli(&args[2..], false),
        Some("create-endnote") => note_create_cli(&args[2..], true),
        Some("get-footnote") => note_get_cli(&args[2..]),
        Some("insert-footnote-text") => note_text_cli(&args[2..], "insert"),
        Some("delete-footnote-text") => note_text_cli(&args[2..], "delete"),
        Some("split-footnote-paragraph") => note_paragraph_cli(&args[2..], "split"),
        Some("merge-footnote-paragraph") => note_paragraph_cli(&args[2..], "merge"),
        Some("delete-footnote") => note_delete_cli(&args[2..]),
        Some("create-table") => create_table_cli(&args[2..]),
        Some("copy-table") => table_structure_cli(&args[2..], "copy-table"),
        Some("delete-table") => table_structure_cli(&args[2..], "delete-table"),
        Some("set-cell-text") => set_cell_text_cli(&args[2..]),
        Some("insert-cell-text") => cell_text_edit_cli(&args[2..], false),
        Some("delete-cell-text") => cell_text_edit_cli(&args[2..], true),
        Some("insert-cell-paragraph") => cell_paragraph_edit_cli(&args[2..], false),
        Some("delete-cell-paragraph") => cell_paragraph_edit_cli(&args[2..], true),
        Some("split-cell-paragraph") => cell_paragraph_cli(&args[2..], false),
        Some("merge-cell-paragraph") => cell_paragraph_cli(&args[2..], true),
        Some("set-cell-field") => cell_field_cli(&args[2..], false),
        Some("clear-cell-field") => cell_field_cli(&args[2..], true),
        Some("insert-table-row") => table_structure_cli(&args[2..], "insert-table-row"),
        Some("copy-table-row") => table_structure_cli(&args[2..], "copy-table-row"),
        Some("delete-table-row") => table_structure_cli(&args[2..], "delete-table-row"),
        Some("insert-table-column") => table_structure_cli(&args[2..], "insert-table-column"),
        Some("copy-table-column") => table_structure_cli(&args[2..], "copy-table-column"),
        Some("delete-table-column") => table_structure_cli(&args[2..], "delete-table-column"),
        Some("merge-table-cells") => table_structure_cli(&args[2..], "merge-table-cells"),
        Some("split-table-cell") => table_structure_cli(&args[2..], "split-table-cell"),
        Some("get-cell-properties") => get_table_properties_cli(&args[2..], true),
        Some("set-cell-properties") => set_table_properties_cli(&args[2..], true),
        Some("get-table-properties") => get_table_properties_cli(&args[2..], false),
        Some("set-table-properties") => set_table_properties_cli(&args[2..], false),
        Some("resize-table-cells") => resize_table_cells_cli(&args[2..]),
        Some("get-char-properties") => get_format_properties_cli(&args[2..], "char"),
        Some("set-char-format") => set_format_cli(&args[2..], "char"),
        Some("get-para-properties") => get_format_properties_cli(&args[2..], "para"),
        Some("set-para-format") => set_format_cli(&args[2..], "para"),
        Some("list-styles") => list_styles_cli(&args[2..]),
        Some("apply-style") => apply_style_cli(&args[2..], false),
        Some("apply-cell-style") => apply_style_cli(&args[2..], true),
        Some("get-cell-char-properties") => get_format_properties_cli(&args[2..], "cell-char"),
        Some("set-cell-char-format") => set_format_cli(&args[2..], "cell-char"),
        Some("get-cell-para-properties") => get_format_properties_cli(&args[2..], "cell-para"),
        Some("set-cell-para-format") => set_format_cli(&args[2..], "cell-para"),
        Some("get-page-def") => get_page_settings_cli(&args[2..], "page-def"),
        Some("set-page-def") => set_page_settings_cli(&args[2..], "page-def"),
        Some("get-section-def") => get_page_settings_cli(&args[2..], "section-def"),
        Some("set-section-def") => set_page_settings_cli(&args[2..], "section-def"),
        Some("get-page-border-fill") => get_page_settings_cli(&args[2..], "page-border-fill"),
        Some("set-page-border-fill") => set_page_settings_cli(&args[2..], "page-border-fill"),
        Some("insert-picture") => insert_picture_cli(&args[2..]),
        Some("get-picture-properties") => get_object_properties_cli(&args[2..], "picture"),
        Some("set-picture-properties") => set_object_properties_cli(&args[2..], "picture"),
        Some("delete-picture") => delete_object_cli(&args[2..], "picture"),
        Some("create-shape") => create_shape_cli(&args[2..]),
        Some("set-cell-shape-text") => set_cell_shape_text_cli(&args[2..]),
        Some("set-cell-shape-char-format") => set_cell_shape_format_cli(&args[2..], true),
        Some("set-cell-shape-para-format") => set_cell_shape_format_cli(&args[2..], false),
        Some("get-shape-properties") => get_object_properties_cli(&args[2..], "shape"),
        Some("set-shape-properties") => set_object_properties_cli(&args[2..], "shape"),
        Some("delete-shape") => delete_object_cli(&args[2..], "shape"),
        Some("change-shape-z-order") => change_shape_z_order_cli(&args[2..]),
        Some("group-shapes") => group_shapes_cli(&args[2..]),
        Some("ungroup-shape") => ungroup_shape_cli(&args[2..]),
        Some("get-header-footer") => get_header_footer_cli(&args[2..]),
        Some("list-header-footer") => list_header_footer_cli(&args[2..]),
        Some("create-header-footer") => header_footer_simple_edit_cli(&args[2..], "create"),
        Some("delete-header-footer") => header_footer_simple_edit_cli(&args[2..], "delete"),
        Some("insert-header-footer-text") => header_footer_text_edit_cli(&args[2..], "insert"),
        Some("delete-header-footer-text") => header_footer_text_edit_cli(&args[2..], "delete"),
        Some("split-header-footer-paragraph") => {
            header_footer_paragraph_edit_cli(&args[2..], "split")
        }
        Some("merge-header-footer-paragraph") => {
            header_footer_paragraph_edit_cli(&args[2..], "merge")
        }
        Some("get-header-footer-para-info") => get_header_footer_para_info_cli(&args[2..]),
        Some("get-header-footer-para-properties") => {
            get_header_footer_para_properties_cli(&args[2..])
        }
        Some("set-header-footer-para-format") => set_header_footer_para_format_cli(&args[2..]),
        Some("insert-header-footer-field") => insert_header_footer_field_cli(&args[2..]),
        Some("apply-header-footer-template") => apply_header_footer_template_cli(&args[2..]),
        Some("list-master-pages") => list_master_pages_cli(&args[2..]),
        Some("create-master-page") => create_master_page_cli(&args[2..]),
        Some("set-master-page-text") => set_master_page_text_cli(&args[2..]),
        Some("delete-master-page") => delete_master_page_cli(&args[2..]),
        Some("diag") => diag_document(&args[2..]),
        Some("convert") => convert_hwp(&args[2..]),
        Some("build-from-ingest") => build_from_ingest(&args[2..]),
        Some("hwp5-inventory") => rhwp::diagnostics::hwp5_inventory::run(&args[2..]),
        Some("hwp5-inventory-diff") => rhwp::diagnostics::hwp5_inventory_diff::run(&args[2..]),
        Some("hwp5-contract-analyze") => rhwp::diagnostics::hwp5_contract_analyze::run(&args[2..]),
        Some("hwp5-ctrl-data-trace") => rhwp::diagnostics::hwp5_ctrl_data_trace::run(&args[2..]),
        Some("hwp5-contract-probe") => rhwp::diagnostics::hwp5_contract_probe::run(&args[2..]),
        Some("hwp5-table-probe") => rhwp::diagnostics::hwp5_table_probe::run(&args[2..]),
        Some("hwp5-mel-personnel-probe") => {
            rhwp::diagnostics::hwp5_mel_personnel_probe::run(&args[2..])
        }
        Some("hwp5-borderfill-diagonal-probe") => {
            rhwp::diagnostics::hwp5_borderfill_diagonal_probe::run(&args[2..])
        }
        Some("hwp5-first-para-control-probe") => {
            rhwp::diagnostics::hwp5_first_para_control_probe::run(&args[2..])
        }
        Some("hwp5-anchor-trace") => rhwp::diagnostics::hwp5_anchor_trace::run(&args[2..]),
        Some("hwp5-cell-header-probe") => {
            rhwp::diagnostics::hwp5_cell_header_probe::run(&args[2..])
        }
        Some("dump-records") => dump_raw_records(&args[2..]),
        Some("test-shape") => test_shape_roundtrip(&args[2..]),
        Some("test-caption") => test_caption(&args[2..]),
        Some("gen-table") => gen_table(&args[2..]),
        Some("gen-pua") => gen_pua_test(&args[2..]),
        Some("test-field") => test_field_roundtrip(&args[2..]),
        Some("ir-diff") => ir_diff(&args[2..]),
        Some("hwpx-roundtrip") => rhwp::diagnostics::hwpx_roundtrip_batch::run(&args[2..]),
        Some("thumbnail") => extract_thumbnail(&args[2..]),
        _ => {
            println!("rhwp v{}", rhwp::version());
            println!("사용법: rhwp <명령> [옵션]");
            println!("'rhwp --help'로 자세한 사용법을 확인하세요.");
        }
    }
}

fn print_help() {
    println!("rhwp v{} - HWP 파일 뷰어", rhwp::version());
    println!();
    println!("사용법: rhwp <명령> [옵션]");
    println!();
    println!("명령:");
    println!("  export-svg <파일.hwp> [옵션]");
    println!("      HWP 파일을 SVG로 내보내기");
    println!();
    println!("      -o, --output <폴더>     출력 폴더 (기본: output/)");
    println!("      -p, --page <번호>       특정 페이지만 내보내기 (0부터 시작)");
    println!("      --show-para-marks       문단부호(↵/↓) 표시");
    println!("      --show-control-codes    조판부호 보이기 (문단부호 + 개체 마커 등)");
    println!("      --debug-overlay         디버그 오버레이 (문단/표 경계 + 인덱스 라벨)");
    println!("      --respect-vpos-reset    LINE_SEG vpos=0 리셋을 단/페이지 강제 경계로 처리");
    println!("      --show-grid[=Nmm]       격자 오버레이 (기본: 1mm, 예: --show-grid=3mm)");
    println!("      --grid-origin=X,Y|auto  격자 종이 기준 위치 (예: --grid-origin=15mm,20mm)");
    println!("      --font-style            @font-face local() 참조 삽입 (폰트 데이터 미포함)");
    println!("      --embed-fonts           폰트 서브셋 임베딩 (사용 글자만 base64)");
    println!("      --embed-fonts=full      폰트 전체 임베딩 (base64)");
    println!("      --font-path <경로>      폰트 파일 탐색 경로 (여러 번 지정 가능)");
    println!();
    println!("  export-render-tree <파일.hwp> [옵션]");
    println!("      페이지별 render tree bbox JSON을 내보내기 (레이아웃 시각 분석용)");
    println!();
    println!("      -o, --output <폴더>     출력 폴더 (기본: output/)");
    println!("      -p, --page <번호>       특정 페이지만 내보내기 (0부터 시작)");
    println!("      --show-para-marks       문단부호(↵/↓) 표시 상태의 트리 생성");
    println!("      --show-control-codes    조판부호 보이기 상태의 트리 생성");
    println!("      --respect-vpos-reset    LINE_SEG vpos=0 리셋을 단/페이지 강제 경계로 처리");
    println!();
    println!("  export-png <파일.hwp> [옵션]   (native-skia feature 필요)");
    println!("      HWP 파일을 PNG로 내보내기 (Skia raster backend, AI 파이프라인 + VLM 연동)");
    println!();
    println!("      -o, --output <폴더>     출력 폴더 (기본: output/)");
    println!("      -p, --page <번호>       특정 페이지만 내보내기 (0부터 시작)");
    println!("      --font-path <경로>      폰트 파일 탐색 경로 (여러 번 지정 가능)");
    println!("                              한컴 전용 폰트 (HY견명조 등) 가 시스템에 없을 때 ttfs 디렉토리 지정");
    println!("      --scale <배율>          렌더링 배율 (기본: 1.0)");
    println!("      --max-dimension <픽셀>  한 변 최대 픽셀 (longest edge). VLM 입력 한도용.");
    println!(
        "                              명시 --scale 이 없으면 자동 scale 계산 (페이지 → 한도 안)"
    );
    println!("      --dpi <값>              DPI 메타데이터 (PNG pHYs chunk). 실제 픽셀 수 무관.");
    println!("                              --scale 미지정 시 scale = dpi/96 자동 계산");
    println!("      --vlm-target <프리셋>   VLM 입력 프리셋 (하이픈/밑줄 모두 허용):");
    println!("                              claude:     1568 px / 1.15 MP (Claude Vision)");
    println!("                              gpt4v-low:  512 px (GPT-4V low detail)");
    println!(
        "                              gpt4v-high: 2000 px / 1.54 MP (GPT-4V high, 별칭: gpt4v)"
    );
    println!("                              gemini:     3072 px (Google Gemini)");
    println!("                              qwen-vl:    2240 px (Qwen-VL, 별칭: qwen)");
    println!("                              llava:      672 px (LLaVA / OSS CLIP)");
    println!();
    println!("  export-text <파일.hwp> [옵션]");
    println!("      페이지별 텍스트를 TXT로 내보내기");
    println!();
    println!("      -o, --output <폴더>     출력 폴더 (기본: output/)");
    println!("      -p, --page <번호>       특정 페이지만 내보내기 (0부터 시작)");
    println!();
    println!("  export-markdown <파일.hwp> [옵션]");
    println!("      페이지별 텍스트를 Markdown(.md)으로 내보내기");
    println!();
    println!("      -o, --output <폴더>     출력 폴더 (기본: output/)");
    println!("      -p, --page <번호>       특정 페이지만 내보내기 (0부터 시작)");
    println!();
    println!("  export-pdf <파일.hwp> [-o 출력.pdf] [-p 페이지]");
    println!("      HWP 파일을 PDF로 내보내기 (svg2pdf + pdf-writer)");
    println!();
    println!("  info <파일.hwp>");
    println!("      HWP 파일 정보 표시");
    println!();
    println!("  dump <파일.hwp> [--section <번호>] [--para <번호>]");
    println!("      문서 조판부호 구조 덤프 (디버깅용)");
    println!();
    println!("  dump-note-shape <파일.hwp|파일.hwpx>");
    println!("      구역별 각주/미주 모양 raw 값과 한컴 UI 의미값을 JSON으로 덤프");
    println!();
    println!("  dump-endnote-lines <파일.hwp> <section> <para> <control> [note-para]");
    println!("      특정 미주 원본 문단의 line_seg, TextRun, TAC 수식 위치를 함께 덤프");
    println!();
    println!("  dump-pages <파일.hwp> [-p <번호>] [--respect-vpos-reset]");
    println!("      페이지네이션 결과 덤프 (페이지별 문단/표 배치 목록)");
    println!();
    println!("  create-hwp --text <텍스트>|--text-file <파일> -o <출력.hwp> [--template <파일.hwp|파일.hwpx>]");
    println!("      새 binary HWP 생성 (빈 HWP 또는 템플릿 기반)");
    println!();
    println!("  replace-text <파일.hwp> --old <검색어> --new <대체문구> -o <출력.hwp> [--all] [--case-sensitive]");
    println!("      binary HWP 본문/표/글상자 텍스트 치환 후 저장");
    println!();
    println!("  list-fields <파일.hwp>");
    println!("      HWP 누름틀/셀 필드 목록을 JSON으로 출력");
    println!();
    println!("  insert-clickhere-field <파일.hwp> --section N --para N [--ctrl N (--cell N|--textbox) [--cell-para N|--textbox-para N] | --cell-path JSON --ctrl N [--textbox-para N]] --offset N --name <필드명> [--guide <안내문>] [--memo <메모>] [--value <초기값>] -o <출력.hwp>");
    println!("      본문/표 셀/글상자 지정 위치에 이름 있는 HWP 누름틀 필드를 생성 후 저장");
    println!();
    println!("  get-field-info <파일.hwp> --section N --para N [--ctrl N (--cell N|--textbox) [--cell-para N|--textbox-para N]] --offset N");
    println!("      본문/표 셀/글상자 지정 위치의 HWP 누름틀 필드 정보를 JSON으로 출력");
    println!();
    println!("  remove-field <파일.hwp> --section N --para N [--ctrl N (--cell N|--textbox) [--cell-para N|--textbox-para N]] --offset N -o <출력.hwp>");
    println!("      본문/표 셀/글상자 지정 위치의 HWP 누름틀 필드를 제거하고 텍스트는 유지");
    println!();
    println!("  set-field <파일.hwp> --name <필드명> --value <값> -o <출력.hwp>");
    println!("      이름이 있는 HWP 필드 값을 설정 후 저장");
    println!();
    println!("  list-forms <파일.hwp>");
    println!("      HWP 양식 개체 목록을 이름/위치 JSON으로 출력");
    println!();
    println!("  create-form <파일.hwp> --section N --para N [--cell-path <JSON>|--table-ctrl N --row R --col C [--cell-para N]] --offset N --form-type checkbox|radio|edit|button|combo --name <이름> [--caption <캡션>] [--text <텍스트>] [--value N] [--width N] [--height N] -o <출력.hwp>");
    println!("      본문 또는 표 셀/글상자 내부 지정 위치에 HWP 양식 개체를 생성 후 저장");
    println!();
    println!("  get-form <파일.hwp> --section N --para N [--cell-path <JSON>|--table-ctrl N --row R --col C [--cell-para N]] --ctrl N");
    println!("      HWP 양식 개체 정보를 JSON으로 출력");
    println!();
    println!("  set-form <파일.hwp> --section N --para N [--cell-path <JSON>|--table-ctrl N --row R --col C [--cell-para N]] --ctrl N --json <값JSON> -o <출력.hwp>");
    println!("      HWP 양식 개체의 value/text/caption 값을 설정 후 저장");
    println!();
    println!("  delete-form <파일.hwp> --section N --para N [--cell-path <JSON>|--table-ctrl N --row R --col C [--cell-para N]] --ctrl N -o <출력.hwp>");
    println!("      HWP 양식 개체를 삭제 후 저장");
    println!();
    println!("  extract-structure <파일.hwp>");
    println!("      문단/표/셀 위치를 JSON으로 추출");
    println!();
    println!(
        "  insert-text <파일.hwp> --section N --para N --offset N --text <텍스트> -o <출력.hwp>"
    );
    println!("      본문 문단의 지정 문자 오프셋에 텍스트를 삽입");
    println!();
    println!("  delete-text <파일.hwp> --section N --para N --offset N --count N -o <출력.hwp>");
    println!("      본문 문단의 지정 문자 범위를 삭제");
    println!();
    println!("  set-paragraph <파일.hwp> --section N --para N --text <텍스트> -o <출력.hwp>");
    println!("      지정 문단 텍스트를 직접 교체");
    println!();
    println!("  insert-paragraph <파일.hwp> --section N --para N [--text <텍스트>] -o <출력.hwp>");
    println!("      지정 위치에 본문 문단을 삽입");
    println!();
    println!("  copy-paragraph <파일.hwp> --section N --para N [--before|--after] -o <출력.hwp>");
    println!("      지정 본문 문단을 텍스트와 서식까지 복제");
    println!();
    println!("  copy-paragraph-range <파일.hwp> --section N --start N --end N [--before|--after] [--replace OLD NEW]... -o <출력.hwp>");
    println!("      지정 본문 문단 범위를 같은 순서로 복제");
    println!();
    println!("  split-paragraph <파일.hwp> --section N --para N --offset N -o <출력.hwp>");
    println!("      본문 문단을 문자 오프셋에서 분할");
    println!();
    println!("  merge-paragraph <파일.hwp> --section N --para N -o <출력.hwp>");
    println!("      지정 본문 문단을 이전 문단에 병합");
    println!();
    println!("  delete-paragraph <파일.hwp> --section N --para N -o <출력.hwp>");
    println!("      본문 문단을 삭제");
    println!();
    println!("  insert-page-break <파일.hwp> --section N --para N --offset N -o <출력.hwp>");
    println!("      지정 위치에 쪽 나누기를 삽입");
    println!();
    println!("  insert-column-break <파일.hwp> --section N --para N --offset N -o <출력.hwp>");
    println!("      지정 위치에 단 나누기를 삽입");
    println!();
    println!("  set-column-def <파일.hwp> --section N --count N [--type normal|distribute|parallel] [--spacing HWPUNIT] [--same-width|--variable-width] -o <출력.hwp>");
    println!("      구역 다단 설정을 변경");
    println!();
    println!(
        "  insert-new-number <파일.hwp> --section N --para N --offset N --start N -o <출력.hwp>"
    );
    println!("      지정 문단 위치부터 쪽 번호를 새 번호로 시작");
    println!();
    println!("  get-page-hide <파일.hwp> --section N --para N");
    println!("      문단의 쪽 감추기(PageHide) 상태를 JSON으로 조회");
    println!();
    println!("  set-page-hide <파일.hwp> --section N --para N [--hide-header] [--hide-footer] [--hide-master-page] [--hide-border] [--hide-fill] [--hide-page-num] -o <출력.hwp>");
    println!("      문단의 쪽 감추기(PageHide) 플래그를 설정. 플래그가 없으면 기존 PageHide 제거");
    println!();
    println!("  list-bookmarks <파일.hwp>");
    println!("      문서 내 책갈피 목록을 JSON으로 조회");
    println!();
    println!(
        "  add-bookmark <파일.hwp> --section N --para N --offset N --name <이름> -o <출력.hwp>"
    );
    println!("      본문 위치에 책갈피를 추가");
    println!();
    println!(
        "  rename-bookmark <파일.hwp> --section N --para N --ctrl N --name <새이름> -o <출력.hwp>"
    );
    println!("      책갈피 이름을 변경");
    println!();
    println!("  delete-bookmark <파일.hwp> --section N --para N --ctrl N -o <출력.hwp>");
    println!("      책갈피 컨트롤을 삭제");
    println!();
    println!("  create-footnote <파일.hwp> --section N --para N --offset N [--text <텍스트>|--text-file <파일>] -o <출력.hwp>");
    println!("      지정 위치에 각주를 생성하고 선택적으로 내용을 입력");
    println!();
    println!("  create-endnote <파일.hwp> --section N --para N --offset N [--text <텍스트>|--text-file <파일>] -o <출력.hwp>");
    println!("      지정 위치에 미주를 생성하고 선택적으로 내용을 입력");
    println!();
    println!("  get-footnote <파일.hwp> --section N --para N --ctrl N");
    println!("      각주/미주 본문 정보를 JSON으로 조회");
    println!();
    println!("  insert-footnote-text <파일.hwp> --section N --para N --ctrl N --note-para N --offset N --text <텍스트> -o <출력.hwp>");
    println!("      각주/미주 문단에 텍스트를 삽입");
    println!();
    println!("  delete-footnote-text <파일.hwp> --section N --para N --ctrl N --note-para N --offset N --count N -o <출력.hwp>");
    println!("      각주/미주 문단 텍스트 일부를 삭제");
    println!();
    println!("  split-footnote-paragraph <파일.hwp> --section N --para N --ctrl N --note-para N --offset N -o <출력.hwp>");
    println!("      각주/미주 문단을 분할");
    println!();
    println!("  merge-footnote-paragraph <파일.hwp> --section N --para N --ctrl N --note-para N -o <출력.hwp>");
    println!("      각주/미주 문단을 이전 문단과 병합");
    println!();
    println!("  delete-footnote <파일.hwp> --section N --para N --ctrl N -o <출력.hwp>");
    println!("      본문 각주와 각주 본문을 삭제");
    println!();
    println!(
        "  create-table <파일.hwp> --section N --para N --offset N --rows N --cols N -o <출력.hwp>"
    );
    println!("      지정 위치에 HWP 표를 생성");
    println!();
    println!(
        "  copy-table <파일.hwp> --section N --para N --ctrl N [--before|--after] [--replace OLD NEW]... -o <출력.hwp>"
    );
    println!("      표 전체를 내용과 서식까지 복제하여 삽입");
    println!();
    println!("  delete-table <파일.hwp> --section N --para N --ctrl N -o <출력.hwp>");
    println!("      독립 표 문단과 바로 뒤 빈 문단을 삭제");
    println!();
    println!("  set-cell-text <파일.hwp> --para N --ctrl N (--cell N|--row N --col N) [--cell-para N] --text <텍스트> -o <출력.hwp>");
    println!("      표 셀 문단 텍스트를 직접 교체");
    println!();
    println!("  insert-cell-text <파일.hwp> --para N --ctrl N (--cell N|--row N --col N) [--cell-para N] --offset N --text <텍스트> -o <출력.hwp>");
    println!("      표 셀 문단의 지정 문자 오프셋에 텍스트를 삽입");
    println!();
    println!("  delete-cell-text <파일.hwp> --para N --ctrl N (--cell N|--row N --col N) [--cell-para N] --offset N --count N -o <출력.hwp>");
    println!("      표 셀 문단의 지정 문자 범위를 삭제");
    println!();
    println!("  insert-cell-paragraph <파일.hwp> --para N --ctrl N (--cell N|--row N --col N) --cell-para N [--text <텍스트>] -o <출력.hwp>");
    println!("      표 셀 내부의 지정 위치에 새 문단을 삽입");
    println!();
    println!("  delete-cell-paragraph <파일.hwp> --para N --ctrl N (--cell N|--row N --col N) --cell-para N -o <출력.hwp>");
    println!("      표 셀 내부 문단을 삭제");
    println!();
    println!("  split-cell-paragraph <파일.hwp> --para N --ctrl N (--cell N|--row N --col N) [--cell-para N] --offset N -o <출력.hwp>");
    println!("      표 셀 내부 문단을 문자 오프셋에서 분할");
    println!();
    println!("  merge-cell-paragraph <파일.hwp> --para N --ctrl N (--cell N|--row N --col N) --cell-para N -o <출력.hwp>");
    println!("      표 셀 내부 문단을 이전 셀 문단에 병합");
    println!();
    println!("  set-cell-field <파일.hwp> --para N --ctrl N (--cell N|--row N --col N) --name <필드명> -o <출력.hwp>");
    println!("      표 셀을 이름 있는 HWP 셀 필드로 지정");
    println!();
    println!(
        "  clear-cell-field <파일.hwp> --para N --ctrl N (--cell N|--row N --col N) -o <출력.hwp>"
    );
    println!("      표 셀의 HWP 셀 필드 이름을 제거하고 텍스트는 유지");
    println!();
    println!("  insert-table-row <파일.hwp> --section N --para N --ctrl N --row N [--above|--below] -o <출력.hwp>");
    println!("      표 행을 삽입");
    println!();
    println!("  copy-table-row <파일.hwp> --section N --para N --ctrl N --row N [--above|--below] [--replace OLD NEW]... -o <출력.hwp>");
    println!("      표 행을 내용과 서식까지 복제하여 삽입");
    println!();
    println!("  delete-table-row <파일.hwp> --section N --para N --ctrl N --row N -o <출력.hwp>");
    println!("      표 행을 삭제");
    println!();
    println!("  insert-table-column <파일.hwp> --section N --para N --ctrl N --col N [--left|--right] -o <출력.hwp>");
    println!("      표 열을 삽입");
    println!();
    println!("  copy-table-column <파일.hwp> --section N --para N --ctrl N --col N [--left|--right] [--replace OLD NEW]... -o <출력.hwp>");
    println!("      표 열을 내용과 서식까지 복제하여 삽입");
    println!();
    println!(
        "  delete-table-column <파일.hwp> --section N --para N --ctrl N --col N -o <출력.hwp>"
    );
    println!("      표 열을 삭제");
    println!();
    println!("  merge-table-cells <파일.hwp> --section N --para N --ctrl N --start-row N --start-col N --end-row N --end-col N -o <출력.hwp>");
    println!("      표 셀 범위를 병합");
    println!();
    println!(
        "  split-table-cell <파일.hwp> --section N --para N --ctrl N --row N --col N -o <출력.hwp>"
    );
    println!("      병합된 표 셀을 분할");
    println!();
    println!("  get-cell-properties <파일.hwp> --section N --para N --ctrl N --cell N");
    println!("      표 셀 속성을 JSON으로 조회");
    println!();
    println!("  set-cell-properties <파일.hwp> --section N --para N --ctrl N --cell N --json <속성JSON> -o <출력.hwp>");
    println!("      표 셀 폭/높이/패딩/정렬/보호/테두리/채우기 속성을 직접 수정");
    println!();
    println!("  get-table-properties <파일.hwp> --section N --para N --ctrl N");
    println!("      표 속성을 JSON으로 조회");
    println!();
    println!("  set-table-properties <파일.hwp> --section N --para N --ctrl N --json <속성JSON> -o <출력.hwp>");
    println!("      표 패딩/반복 머리/배치/테두리/채우기 속성을 직접 수정");
    println!();
    println!("  resize-table-cells <파일.hwp> --section N --para N --ctrl N --json <변경배열JSON> -o <출력.hwp>");
    println!("      여러 표 셀의 폭/높이를 델타 값으로 조절");
    println!();
    println!("  get-char-properties <파일.hwp> --section N --para N --offset N");
    println!("      본문 글자 속성을 JSON으로 조회");
    println!();
    println!("  set-char-format <파일.hwp> --section N --para N --start N --end N --json <서식JSON> -o <출력.hwp>");
    println!("      본문 글자 범위 서식을 직접 수정");
    println!();
    println!("  get-para-properties <파일.hwp> --section N --para N");
    println!("      본문 문단 속성을 JSON으로 조회");
    println!();
    println!("  set-para-format <파일.hwp> --section N --para N --json <서식JSON> -o <출력.hwp>");
    println!("      본문 문단 서식을 직접 수정");
    println!();
    println!("  list-styles <파일.hwp>");
    println!("      문서 스타일 목록을 JSON으로 조회");
    println!();
    println!("  apply-style <파일.hwp> --section N --para N (--style-id N|--style-name <이름>) -o <출력.hwp>");
    println!("      본문 문단에 문서 스타일을 적용");
    println!();
    println!("  apply-cell-style <파일.hwp> --section N --para N --ctrl N (--cell N|--row R --col C) [--cell-para N] (--style-id N|--style-name <이름>) -o <출력.hwp>");
    println!("      표 셀 내부 문단에 문서 스타일을 적용");
    println!();
    println!("  get-cell-char-properties <파일.hwp> --section N --para N --ctrl N (--cell N|--row R --col C) [--cell-para N] --offset N");
    println!("      셀 내부 글자 속성을 JSON으로 조회");
    println!();
    println!("  set-cell-char-format <파일.hwp> --section N --para N --ctrl N (--cell N|--row R --col C) [--cell-para N] --start N --end N --json <서식JSON> -o <출력.hwp>");
    println!("      셀 내부 글자 범위 서식을 직접 수정");
    println!();
    println!("  get-cell-para-properties <파일.hwp> --section N --para N --ctrl N (--cell N|--row R --col C) [--cell-para N]");
    println!("      셀 내부 문단 속성을 JSON으로 조회");
    println!();
    println!("  set-cell-para-format <파일.hwp> --section N --para N --ctrl N (--cell N|--row R --col C) [--cell-para N] --json <서식JSON> -o <출력.hwp>");
    println!("      셀 내부 문단 서식을 직접 수정");
    println!();
    println!("  get-page-def <파일.hwp> --section N");
    println!("      용지 크기/여백 설정을 JSON으로 조회");
    println!();
    println!("  set-page-def <파일.hwp> --section N --json <설정JSON> -o <출력.hwp>");
    println!("      용지 크기/여백 설정을 직접 수정");
    println!();
    println!("  get-section-def <파일.hwp> --section N");
    println!("      구역 번호/숨김/탭 설정을 JSON으로 조회");
    println!();
    println!("  set-section-def <파일.hwp> --section N --json <설정JSON> -o <출력.hwp>");
    println!("      구역 번호/숨김/탭 설정을 직접 수정");
    println!();
    println!("  get-page-border-fill <파일.hwp> --section N");
    println!("      쪽 테두리/배경 설정을 JSON으로 조회");
    println!();
    println!("  set-page-border-fill <파일.hwp> --section N --json <설정JSON> -o <출력.hwp>");
    println!("      쪽 테두리/배경 설정을 직접 수정");
    println!();
    println!("  insert-picture <파일.hwp> --section N --para N --offset N --image <이미지> --width N --height N [--cell-path <JSON>|--table-ctrl N --row R --col C [--cell-para N]] -o <출력.hwp>");
    println!("      그림 컨트롤을 삽입");
    println!();
    println!("  list-objects <파일.hwp>");
    println!("      본문/표 셀 내부 그림·도형·글상자 객체 목록을 JSON으로 조회");
    println!();
    println!("  get-picture-properties <파일.hwp> --section N --para N [--cell-path <JSON>|--table-ctrl N --row R --col C [--cell-para N]] --ctrl N");
    println!("      본문/표 셀 내부 그림 속성을 JSON으로 조회");
    println!();
    println!("  set-picture-properties <파일.hwp> --section N --para N [--cell-path <JSON>|--table-ctrl N --row R --col C [--cell-para N]] --ctrl N --json <속성JSON> -o <출력.hwp>");
    println!("      본문/표 셀 내부 그림 크기/위치/효과/자르기/캡션 속성을 직접 수정");
    println!();
    println!("  delete-picture <파일.hwp> --section N --para N [--cell-path <JSON>|--table-ctrl N --row R --col C [--cell-para N]] --ctrl N -o <출력.hwp>");
    println!("      본문/표 셀 내부 그림 컨트롤 삭제");
    println!();
    println!("  create-shape <파일.hwp> --section N --para N [--cell-path <JSON>|--table-ctrl N --row R --col C [--cell-para N]] --offset N --width N --height N [--shape-type rectangle|textbox|line|ellipse|polygon|arc] -o <출력.hwp>");
    println!("      본문/표 셀 내부 도형·글상자 컨트롤을 삽입");
    println!();
    println!("  set-cell-shape-text <파일.hwp> --section N --para N [--cell-path <JSON>|--table-ctrl N --row R --col C [--cell-para N]] --ctrl N [--textbox-para N] --text <텍스트> -o <출력.hwp>");
    println!("      표 셀 내부 글상자 텍스트를 직접 수정");
    println!();
    println!("  set-cell-shape-char-format <파일.hwp> --section N --para N [--cell-path <JSON>|--table-ctrl N --row R --col C [--cell-para N]] --ctrl N --textbox-para N --start N --end N --json <서식JSON> -o <출력.hwp>");
    println!("      표 셀 내부 글상자 문자 범위 서식을 직접 수정");
    println!();
    println!("  set-cell-shape-para-format <파일.hwp> --section N --para N [--cell-path <JSON>|--table-ctrl N --row R --col C [--cell-para N]] --ctrl N --textbox-para N --json <서식JSON> -o <출력.hwp>");
    println!("      표 셀 내부 글상자 문단 서식을 직접 수정");
    println!();
    println!("  get-shape-properties <파일.hwp> --section N --para N --ctrl N");
    println!("      도형 속성을 JSON으로 조회");
    println!();
    println!("  set-shape-properties <파일.hwp> --section N --para N --ctrl N --json <속성JSON> -o <출력.hwp>");
    println!("      도형 크기/위치/선/채우기/글상자 속성을 직접 수정");
    println!();
    println!("  delete-shape <파일.hwp> --section N --para N --ctrl N -o <출력.hwp>");
    println!("      도형 컨트롤 삭제");
    println!();
    println!("  change-shape-z-order <파일.hwp> --section N --para N --ctrl N --operation front|back|forward|backward -o <출력.hwp>");
    println!("      도형의 앞/뒤 배치 순서를 변경");
    println!();
    println!("  group-shapes <파일.hwp> --section N --targets <대상JSON> -o <출력.hwp>");
    println!("      여러 그림/도형을 하나의 그룹으로 묶기");
    println!();
    println!("  ungroup-shape <파일.hwp> --section N --para N --ctrl N -o <출력.hwp>");
    println!("      그룹 도형을 한 단계 풀기");
    println!();
    println!(
        "  get-header-footer <파일.hwp> --section N --kind header|footer --apply-to both|even|odd"
    );
    println!("      머리말/꼬리말 내용과 위치를 JSON으로 조회");
    println!();
    println!("  list-header-footer <파일.hwp> [--section N --kind header|footer --apply-to both|even|odd]");
    println!("      문서의 머리말/꼬리말 목록을 JSON으로 조회");
    println!();
    println!("  create-header-footer <파일.hwp> --section N --kind header|footer --apply-to both|even|odd -o <출력.hwp>");
    println!("      빈 머리말/꼬리말을 생성");
    println!();
    println!("  insert-header-footer-text <파일.hwp> --section N --kind header|footer --apply-to both|even|odd --hf-para N --offset N --text <텍스트> -o <출력.hwp>");
    println!("      머리말/꼬리말 문단에 텍스트를 삽입");
    println!();
    println!("  delete-header-footer-text <파일.hwp> --section N --kind header|footer --apply-to both|even|odd --hf-para N --offset N --count N -o <출력.hwp>");
    println!("      머리말/꼬리말 문단에서 텍스트를 삭제");
    println!();
    println!("  split-header-footer-paragraph <파일.hwp> --section N --kind header|footer --apply-to both|even|odd --hf-para N --offset N -o <출력.hwp>");
    println!("      머리말/꼬리말 문단을 분할");
    println!();
    println!("  merge-header-footer-paragraph <파일.hwp> --section N --kind header|footer --apply-to both|even|odd --hf-para N -o <출력.hwp>");
    println!("      머리말/꼬리말 문단을 이전 문단과 병합");
    println!();
    println!("  get-header-footer-para-properties <파일.hwp> --section N --kind header|footer --apply-to both|even|odd --hf-para N");
    println!("      머리말/꼬리말 문단 서식을 JSON으로 조회");
    println!();
    println!("  set-header-footer-para-format <파일.hwp> --section N --kind header|footer --apply-to both|even|odd --hf-para N --json <서식JSON> -o <출력.hwp>");
    println!("      머리말/꼬리말 문단 서식을 직접 수정");
    println!();
    println!("  insert-header-footer-field <파일.hwp> --section N --kind header|footer --apply-to both|even|odd --hf-para N --offset N --field page-number|total-pages|filename -o <출력.hwp>");
    println!("      머리말/꼬리말에 쪽번호/총쪽수/파일명 필드를 삽입");
    println!();
    println!("  apply-header-footer-template <파일.hwp> --section N --kind header|footer --apply-to both|even|odd --template N -o <출력.hwp>");
    println!("      기본 머리말/꼬리말 템플릿(0~10)을 적용");
    println!();
    println!("  delete-header-footer <파일.hwp> --section N --kind header|footer --apply-to both|even|odd -o <출력.hwp>");
    println!("      머리말/꼬리말 컨트롤 삭제");
    println!();
    println!("  list-master-pages <파일.hwp> --section N");
    println!("      바탕쪽 목록과 텍스트를 JSON으로 조회");
    println!();
    println!("  create-master-page <파일.hwp> --section N --apply-to both|even|odd [--text <텍스트>] [--extension] [--overlap] -o <출력.hwp>");
    println!("      바탕쪽을 생성");
    println!();
    println!("  set-master-page-text <파일.hwp> --section N --master N --para N --text <텍스트> -o <출력.hwp>");
    println!("      바탕쪽 문단 텍스트를 수정");
    println!();
    println!("  delete-master-page <파일.hwp> --section N --master N -o <출력.hwp>");
    println!("      바탕쪽을 삭제");
    println!();
    println!("  dump-records <파일.hwp>");
    println!("      HWP5 raw record 덤프 (DocInfo/BodyText 레코드 트리)");
    println!();
    println!("  diag <파일.hwp>");
    println!("      문서 구조 진단 (번호/글머리표/개요 분석)");
    println!();
    println!("  hwp5-inventory <파일.hwp> [--format jsonl|md] [--section N] [--out <path>]");
    println!("      HWP5 DocInfo/BodyText record inventory 생성 (HWPX→HWP contract 분석용)");
    println!();
    println!("  hwp5-inventory-diff <oracle.hwp> <generated.hwp> [--align index|lcs] [--report diff|hints|bundles|table-fields|table-probe-plan] [--focus all|table|shape|ctrl|missing|docinfo] [--window N] [--format jsonl|md] [--section N] [--out <path>]");
    println!("      HWP5 inventory 비교 결과, contract 후보 힌트, 후보 주변 bundle 생성");
    println!();
    println!("  hwp5-contract-analyze <source.hwpx> <oracle.hwp> <generated.hwp> --out-dir <폴더>");
    println!("      HWPX/HWP oracle/generated record-control contract graph 분석 보고서 생성");
    println!();
    println!("  hwp5-ctrl-data-trace <oracle.hwp> <generated.hwp> --out <path> [--section N] [--record-index N]");
    println!("      oracle/generated CTRL_DATA ParameterSet 구조 추적 보고서 생성");
    println!();
    println!("  hwp5-contract-probe <oracle.hwp> <generated.hwp> --out-dir <폴더>");
    println!("      DocInfo MEMO_SHAPE/ID_MAPPINGS와 누락 CTRL_DATA 축별 판정용 HWP probe 생성");
    println!();
    println!("  hwp5-table-probe <oracle.hwp> <generated.hwp> --out-dir <폴더>");
    println!("      TABLE/CTRL_HEADER(Table) field 축별 판정용 HWP probe 생성");
    println!();
    println!("  hwp5-mel-personnel-probe <oracle.hwp> <generated.hwp> --out-dir <폴더>");
    println!("      mel-001 인원현황 표 TABLE/LIST_HEADER/PARA_HEADER 축별 판정용 HWP probe 생성");
    println!();
    println!("  hwp5-borderfill-diagonal-probe <oracle.hwp> <generated.hwp> --out-dir <폴더>");
    println!("      DocInfo BORDER_FILL 대각선 attr/payload 축별 판정용 HWP probe 생성");
    println!();
    println!("  hwp5-first-para-control-probe <oracle.hwp> <generated.hwp> --out-dir <폴더>");
    println!("      첫 문단 control/PARA_TEXT/PARA_CHAR_SHAPE 계약 축별 판정용 HWP probe 생성");
    println!();
    println!("  hwp5-anchor-trace <파일.hwp> --needle <텍스트> [--section N] [--window N] [--out <path>]");
    println!("      특정 텍스트를 포함한 PARA_TEXT 주변의 raw HWP5 record를 추적");
    println!();
    println!("  hwp5-cell-header-probe <oracle.hwp> <generated.hwp> --out-dir <폴더>");
    println!("      표 셀 LIST_HEADER/PARA_HEADER 계약 축별 판정용 HWP probe 생성");
    println!();
    println!("  convert <입력.hwp|입력.hwpx> <출력.hwp>");
    println!("      배포용(읽기전용) HWP를 편집 가능한 HWP로 변환");
    println!();
    println!("  build-from-ingest <ingest.json> [--media-dir <dir>] -o <out.hwpx>");
    println!("      ingest JSON(시험문제 등)을 HWPX로 생성 (rhwp-exam-ingest 파이프라인)");
    println!();
    println!("  ir-diff <파일A.hwpx> <파일B.hwp> [-s <구역>] [-p <문단>]");
    println!("      두 파일의 IR(중간표현) 비교 (HWPX↔HWP 불일치 검출)");
    println!("      비교 항목: text, char_count, char_offsets, char_shapes, line_segs,");
    println!("                 controls(타입+속성), tab_extended, ParaShape, TabDef");
    println!("      표: page_break, outer_margin, treat_as_char, wrap, size, v_offset/h_offset");
    println!("      그림/도형: treat_as_char, wrap, size, v_offset/h_offset, vert_rel/horz_rel");
    println!();
    println!("  hwpx-roundtrip <파일.hwpx | --batch 폴더> [-o <출력폴더>] [--lineseg-report]");
    println!("      HWPX → IR → HWPX roundtrip 검증 (Task #1315 baseline)");
    println!("      재조립 .hwpx와 inventory.tsv를 출력 폴더(기본 output/poc/task1315)에 생성");
    println!("      --lineseg-report: 문단별 lineseg diff를 lineseg_diff.tsv로 산출 (#1380 측정)");
    println!();
    println!("  thumbnail <파일.hwp> [옵션]");
    println!("      HWP 파일에서 썸네일(PrvImage) 추출");
    println!();
    println!("      -o, --output <파일>       출력 파일 경로 (기본: 입력명_thumb.png)");
    println!("      --base64                  base64 문자열을 stdout에 출력");
    println!("      --data-uri                data:image/... URI 형식으로 stdout에 출력");
    println!();
    println!("내부 개발·회귀 도구 (일반 사용자 대상 아님):");
    println!("  test-caption <파일.hwp>             캡션 라운드트립 검증");
    println!("  test-field <파일.hwp>               필드 라운드트립 검증");
    println!("  test-shape <입력.hwp> <출력.hwp>    도형 라운드트립 검증");
    println!("  gen-table                           표 테스트 HWP 생성");
    println!("  gen-pua                             PUA 문자 테스트 HWP 생성");
    println!();
    println!("옵션:");
    println!("  -h, --help      도움말 표시");
    println!("  -V, --version   버전 표시");
}

struct HwpCreateCliResult {
    bytes: Vec<u8>,
    paragraph_count: usize,
    page_count_before: u32,
    page_count_after: u32,
}

struct HwpReplaceCliResult {
    bytes: Vec<u8>,
    count: usize,
    details: serde_json::Value,
    page_count_before: u32,
    page_count_after: u32,
}

struct HwpFieldCliResult {
    bytes: Vec<u8>,
    details: serde_json::Value,
    page_count_before: u32,
    page_count_after: u32,
}

struct HwpEditCliResult {
    bytes: Vec<u8>,
    details: serde_json::Value,
    page_count_before: u32,
    page_count_after: u32,
}

struct HwpTableCliResult {
    bytes: Vec<u8>,
    para_idx: usize,
    control_idx: usize,
    details: serde_json::Value,
    page_count_before: u32,
    page_count_after: u32,
}

fn parse_json_value(s: &str) -> serde_json::Value {
    serde_json::from_str(s).unwrap_or_else(|_| serde_json::json!({"raw": s}))
}

fn serialize_hwp_verified_for_cli(
    core: &mut rhwp::document_core::DocumentCore,
) -> Result<(Vec<u8>, u32, u32), String> {
    let verification = core
        .serialize_hwp_with_verify()
        .map_err(|e| format!("HWP 직렬화/재로드 검증 실패: {}", e))?;
    if !verification.recovered {
        return Err(format!(
            "HWP 재로드 검증 실패: page_count_before={}, page_count_after={}",
            verification.page_count_before, verification.page_count_after
        ));
    }
    Ok((
        verification.bytes,
        verification.page_count_before,
        verification.page_count_after,
    ))
}

fn load_hwp_core_for_cli(
    template_path: Option<&str>,
) -> Result<rhwp::document_core::DocumentCore, String> {
    if let Some(path) = template_path {
        let data =
            fs::read(path).map_err(|e| format!("템플릿 파일 읽기 실패 - {}: {}", path, e))?;
        let mut core = rhwp::document_core::DocumentCore::from_bytes(&data)
            .map_err(|e| format!("템플릿 파싱 실패 - {}: {}", path, e))?;
        core.convert_to_editable_native()
            .map_err(|e| format!("템플릿 편집 가능 변환 실패: {}", e))?;
        Ok(core)
    } else {
        let mut core = rhwp::document_core::DocumentCore::new_empty();
        core.create_blank_document_native()
            .map_err(|e| format!("빈 HWP 생성 실패: {}", e))?;
        Ok(core)
    }
}

fn insert_plain_text_lines_for_cli(
    core: &mut rhwp::document_core::DocumentCore,
    text: &str,
    append: bool,
) -> Result<usize, String> {
    let lines: Vec<&str> = text
        .lines()
        .map(str::trim_end)
        .filter(|line| !line.trim().is_empty())
        .collect();
    if lines.is_empty() {
        return Err("텍스트에 비어 있지 않은 줄이 없습니다.".to_string());
    }

    core.begin_batch_native()
        .map_err(|e| format!("배치 편집 시작 실패: {}", e))?;
    let edit_result = (|| -> Result<(), String> {
        let start_para = if append {
            core.get_paragraph_count_native(0)
                .map_err(|e| format!("문단 수 조회 실패: {}", e))?
        } else {
            let first_len = core
                .get_paragraph_length_native(0, 0)
                .map_err(|e| format!("첫 문단 길이 조회 실패: {}", e))?;
            if first_len > 0 {
                core.delete_text_native(0, 0, 0, first_len)
                    .map_err(|e| format!("첫 문단 초기화 실패: {}", e))?;
            }
            0
        };

        for (i, line) in lines.iter().enumerate() {
            let para_idx = start_para + i;
            if append || i > 0 {
                core.insert_paragraph_native(0, para_idx)
                    .map_err(|e| format!("문단 추가 실패: {}", e))?;
            }
            core.insert_text_native(0, para_idx, 0, line)
                .map_err(|e| format!("텍스트 삽입 실패: {}", e))?;
        }
        Ok(())
    })();
    let end_result = core
        .end_batch_native()
        .map_err(|e| format!("배치 편집 종료 실패: {}", e));
    edit_result?;
    end_result?;
    Ok(lines.len())
}

fn create_hwp_bytes_from_text_for_cli(
    text: &str,
    template_path: Option<&str>,
) -> Result<HwpCreateCliResult, String> {
    let mut core = load_hwp_core_for_cli(template_path)?;
    let paragraph_count =
        insert_plain_text_lines_for_cli(&mut core, text, template_path.is_some())?;
    let verification = core
        .serialize_hwp_with_verify()
        .map_err(|e| format!("HWP 직렬화/재로드 검증 실패: {}", e))?;
    if !verification.recovered {
        return Err(format!(
            "HWP 재로드 검증 실패: page_count_before={}, page_count_after={}",
            verification.page_count_before, verification.page_count_after
        ));
    }
    Ok(HwpCreateCliResult {
        bytes: verification.bytes,
        paragraph_count,
        page_count_before: verification.page_count_before,
        page_count_after: verification.page_count_after,
    })
}

fn replace_hwp_text_bytes_for_cli(
    data: &[u8],
    old: &str,
    new: &str,
    replace_all: bool,
    case_sensitive: bool,
) -> Result<HwpReplaceCliResult, String> {
    if old.is_empty() {
        return Err("검색어는 비어 있을 수 없습니다.".to_string());
    }
    let mut core = rhwp::document_core::DocumentCore::from_bytes(data)
        .map_err(|e| format!("HWP 파싱 실패: {}", e))?;
    core.convert_to_editable_native()
        .map_err(|e| format!("편집 가능 변환 실패: {}", e))?;
    let details_json = if replace_all {
        core.replace_all_native(old, new, case_sensitive)
            .map_err(|e| format!("텍스트 전체 치환 실패: {}", e))?
    } else {
        core.replace_one_native(old, new, case_sensitive)
            .map_err(|e| format!("텍스트 단일 치환 실패: {}", e))?
    };
    let details = parse_json_value(&details_json);
    let count = if replace_all {
        details.get("count").and_then(|v| v.as_u64()).unwrap_or(0) as usize
    } else if details.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) {
        1
    } else {
        0
    };
    let verification = core
        .serialize_hwp_with_verify()
        .map_err(|e| format!("HWP 직렬화/재로드 검증 실패: {}", e))?;
    if !verification.recovered {
        return Err(format!(
            "HWP 재로드 검증 실패: page_count_before={}, page_count_after={}",
            verification.page_count_before, verification.page_count_after
        ));
    }
    Ok(HwpReplaceCliResult {
        bytes: verification.bytes,
        count,
        details,
        page_count_before: verification.page_count_before,
        page_count_after: verification.page_count_after,
    })
}

fn list_hwp_fields_json_for_cli(data: &[u8]) -> Result<serde_json::Value, String> {
    let core = rhwp::document_core::DocumentCore::from_bytes(data)
        .map_err(|e| format!("HWP 파싱 실패: {}", e))?;
    let fields = parse_json_value(&core.get_field_list_json());
    let count = fields.as_array().map(|v| v.len()).unwrap_or(0);
    Ok(serde_json::json!({"ok": true, "count": count, "fields": fields}))
}

fn list_hwp_forms_json_for_cli(data: &[u8]) -> Result<serde_json::Value, String> {
    let core = rhwp::document_core::DocumentCore::from_bytes(data)
        .map_err(|e| format!("HWP 파싱 실패: {}", e))?;
    let mut forms = Vec::new();
    for (section_index, section) in core.document().sections.iter().enumerate() {
        for (paragraph_index, paragraph) in section.paragraphs.iter().enumerate() {
            collect_body_forms_for_cli(
                &mut forms,
                section_index,
                paragraph_index,
                &paragraph.controls,
            );
        }
    }
    Ok(serde_json::json!({
        "ok": true,
        "count": forms.len(),
        "forms": forms,
    }))
}

fn list_hwp_objects_json_for_cli(data: &[u8]) -> Result<serde_json::Value, String> {
    let core = rhwp::document_core::DocumentCore::from_bytes(data)
        .map_err(|e| format!("HWP 파싱 실패: {}", e))?;
    let mut objects = Vec::new();
    for (section_index, section) in core.document().sections.iter().enumerate() {
        for (paragraph_index, paragraph) in section.paragraphs.iter().enumerate() {
            collect_body_objects_for_cli(
                &core,
                &mut objects,
                section_index,
                paragraph_index,
                &paragraph.controls,
            );
        }
    }
    Ok(serde_json::json!({
        "ok": true,
        "count": objects.len(),
        "objects": objects,
    }))
}

fn collect_body_objects_for_cli(
    core: &rhwp::document_core::DocumentCore,
    objects: &mut Vec<serde_json::Value>,
    section_index: usize,
    paragraph_index: usize,
    controls: &[rhwp::model::control::Control],
) {
    for (control_index, control) in controls.iter().enumerate() {
        match control {
            rhwp::model::control::Control::Picture(picture) => {
                let cell_location = floating_picture_cell_location_for_cli(
                    core,
                    section_index,
                    paragraph_index,
                    controls,
                    picture,
                );
                let mut item = picture_object_json_for_cli(
                    if cell_location.is_some() {
                        "cell"
                    } else {
                        "body"
                    },
                    section_index,
                    paragraph_index,
                    control_index,
                    picture,
                );
                if let Some((table_control_index, cell_index, row, col, cell_paragraph_index)) =
                    cell_location
                {
                    add_cell_object_location_for_cli(
                        &mut item,
                        table_control_index,
                        cell_index,
                        row,
                        col,
                        cell_paragraph_index,
                    );
                }
                objects.push(item);
            }
            rhwp::model::control::Control::Shape(shape) => {
                objects.push(shape_object_json_for_cli(
                    "body",
                    section_index,
                    paragraph_index,
                    control_index,
                    shape.as_ref(),
                ));
            }
            rhwp::model::control::Control::Table(table) => {
                collect_cell_objects_for_cli(
                    objects,
                    section_index,
                    paragraph_index,
                    control_index,
                    table,
                );
            }
            _ => {}
        }
    }
}

fn floating_picture_cell_location_for_cli(
    core: &rhwp::document_core::DocumentCore,
    section_index: usize,
    paragraph_index: usize,
    controls: &[rhwp::model::control::Control],
    picture: &rhwp::model::image::Picture,
) -> Option<(usize, usize, u16, u16, usize)> {
    if picture.common.treat_as_char {
        return None;
    }

    for (table_control_index, control) in controls.iter().enumerate() {
        if let rhwp::model::control::Control::Table(table) = control {
            if let Some(location) = floating_picture_cell_location_in_table_for_cli(
                core,
                section_index,
                paragraph_index,
                table_control_index,
                table,
                picture,
            ) {
                return Some(location);
            }
        }
    }
    None
}

fn floating_picture_cell_location_in_table_for_cli(
    core: &rhwp::document_core::DocumentCore,
    section_index: usize,
    paragraph_index: usize,
    table_control_index: usize,
    table: &rhwp::model::table::Table,
    picture: &rhwp::model::image::Picture,
) -> Option<(usize, usize, u16, u16, usize)> {
    let picture_x = picture.common.horizontal_offset as f64 / 75.0;
    let picture_y = picture.common.vertical_offset as f64 / 75.0;

    for page_index in 0..core.page_count().max(1) {
        let tree = match core.build_page_render_tree(page_index) {
            Ok(tree) => tree,
            Err(_) => continue,
        };
        let (cell_index, row, col) = match find_table_cell_at_point_for_cli(
            &tree.root,
            section_index,
            paragraph_index,
            table_control_index,
            picture_x,
            picture_y,
        ) {
            Some(location) => location,
            None => continue,
        };
        let cell = table.cells.get(cell_index)?;
        if cell.paragraphs.is_empty() {
            continue;
        }
        return Some((table_control_index, cell_index, row, col, 0));
    }
    None
}

fn find_table_cell_at_point_for_cli(
    node: &rhwp::renderer::render_tree::RenderNode,
    section_index: usize,
    paragraph_index: usize,
    table_control_index: usize,
    x: f64,
    y: f64,
) -> Option<(usize, u16, u16)> {
    if let rhwp::renderer::render_tree::RenderNodeType::Table(table_node) = &node.node_type {
        if table_node.section_index == Some(section_index)
            && table_node.para_index == Some(paragraph_index)
            && table_node.control_index == Some(table_control_index)
        {
            const CELL_EDGE_EPSILON: f64 = 0.01;
            let mut best: Option<(f64, f64, usize, u16, u16)> = None;
            for child in &node.children {
                if let rhwp::renderer::render_tree::RenderNodeType::TableCell(cell_node) =
                    &child.node_type
                {
                    let bbox = &child.bbox;
                    if x + CELL_EDGE_EPSILON >= bbox.x
                        && y + CELL_EDGE_EPSILON >= bbox.y
                        && x < bbox.x + bbox.width + CELL_EDGE_EPSILON
                        && y < bbox.y + bbox.height + CELL_EDGE_EPSILON
                    {
                        let cell_index = cell_node.model_cell_index? as usize;
                        match best {
                            Some((best_y, best_x, _, _, _))
                                if bbox.y < best_y || (bbox.y == best_y && bbox.x <= best_x) => {}
                            _ => {
                                best = Some((
                                    bbox.y,
                                    bbox.x,
                                    cell_index,
                                    cell_node.row,
                                    cell_node.col,
                                ));
                            }
                        }
                    }
                }
            }
            if let Some((_, _, cell_index, row, col)) = best {
                return Some((cell_index, row, col));
            }
        }
    }

    for child in &node.children {
        if let Some(location) = find_table_cell_at_point_for_cli(
            child,
            section_index,
            paragraph_index,
            table_control_index,
            x,
            y,
        ) {
            return Some(location);
        }
    }
    None
}

fn collect_cell_objects_for_cli(
    objects: &mut Vec<serde_json::Value>,
    section_index: usize,
    paragraph_index: usize,
    table_control_index: usize,
    table: &rhwp::model::table::Table,
) {
    for (cell_index, cell) in table.cells.iter().enumerate() {
        for (cell_paragraph_index, paragraph) in cell.paragraphs.iter().enumerate() {
            for (control_index, control) in paragraph.controls.iter().enumerate() {
                match control {
                    rhwp::model::control::Control::Picture(picture) => {
                        let mut item = picture_object_json_for_cli(
                            "cell",
                            section_index,
                            paragraph_index,
                            control_index,
                            picture,
                        );
                        add_cell_object_location_for_cli(
                            &mut item,
                            table_control_index,
                            cell_index,
                            cell.row,
                            cell.col,
                            cell_paragraph_index,
                        );
                        objects.push(item);
                    }
                    rhwp::model::control::Control::Shape(shape) => {
                        let mut item = shape_object_json_for_cli(
                            "cell",
                            section_index,
                            paragraph_index,
                            control_index,
                            shape.as_ref(),
                        );
                        add_cell_object_location_for_cli(
                            &mut item,
                            table_control_index,
                            cell_index,
                            cell.row,
                            cell.col,
                            cell_paragraph_index,
                        );
                        objects.push(item);
                    }
                    _ => {}
                }
            }
        }
    }
}

fn add_cell_object_location_for_cli(
    item: &mut serde_json::Value,
    table_control_index: usize,
    cell_index: usize,
    row: u16,
    col: u16,
    cell_paragraph_index: usize,
) {
    if let serde_json::Value::Object(obj) = item {
        obj.insert(
            "tableControl".to_string(),
            serde_json::json!(table_control_index),
        );
        obj.insert("cellIndex".to_string(), serde_json::json!(cell_index));
        obj.insert("row".to_string(), serde_json::json!(row));
        obj.insert("col".to_string(), serde_json::json!(col));
        obj.insert(
            "cellParagraph".to_string(),
            serde_json::json!(cell_paragraph_index),
        );
        obj.insert(
            "cellPath".to_string(),
            serde_json::json!([{
                "controlIndex": table_control_index,
                "cellIndex": cell_index,
                "cellParaIndex": cell_paragraph_index,
            }]),
        );
    }
}

fn picture_object_json_for_cli(
    container: &str,
    section_index: usize,
    paragraph_index: usize,
    control_index: usize,
    picture: &rhwp::model::image::Picture,
) -> serde_json::Value {
    let c = &picture.common;
    serde_json::json!({
        "container": container,
        "kind": "picture",
        "section": section_index,
        "paragraph": paragraph_index,
        "control": control_index,
        "width": c.width,
        "height": c.height,
        "treatAsChar": c.treat_as_char,
        "textWrap": text_wrap_name_for_cli(c.text_wrap),
        "zOrder": c.z_order,
        "instanceId": c.instance_id,
        "horzOffset": c.horizontal_offset,
        "vertOffset": c.vertical_offset,
        "description": &c.description,
        "binDataId": picture.image_attr.bin_data_id,
        "brightness": picture.image_attr.brightness,
        "contrast": picture.image_attr.contrast,
    })
}

fn shape_object_json_for_cli(
    container: &str,
    section_index: usize,
    paragraph_index: usize,
    control_index: usize,
    shape: &rhwp::model::shape::ShapeObject,
) -> serde_json::Value {
    let c = shape.common();
    serde_json::json!({
        "container": container,
        "kind": if matches!(shape, rhwp::model::shape::ShapeObject::Picture(_)) { "picture" } else { "shape" },
        "shapeType": shape_type_name_for_cli(shape),
        "section": section_index,
        "paragraph": paragraph_index,
        "control": control_index,
        "width": c.width,
        "height": c.height,
        "treatAsChar": c.treat_as_char,
        "textWrap": text_wrap_name_for_cli(c.text_wrap),
        "zOrder": c.z_order,
        "instanceId": c.instance_id,
        "horzOffset": c.horizontal_offset,
        "vertOffset": c.vertical_offset,
        "description": &c.description,
        "hasTextBox": shape
            .drawing()
            .and_then(|drawing| drawing.text_box.as_ref())
            .is_some(),
    })
}

fn textbox_json_for_cli(textbox: &rhwp::model::shape::TextBox) -> serde_json::Value {
    let paragraphs: Vec<serde_json::Value> = textbox
        .paragraphs
        .iter()
        .enumerate()
        .map(|(index, paragraph)| {
            serde_json::json!({
                "index": index,
                "text": paragraph.text,
                "charCount": paragraph.text.chars().count(),
                "controlCount": paragraph.controls.len(),
            })
        })
        .collect();
    let text = textbox
        .paragraphs
        .iter()
        .map(|paragraph| paragraph.text.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    serde_json::json!({
        "paragraphCount": textbox.paragraphs.len(),
        "paragraphs": paragraphs,
        "text": text,
    })
}

fn shape_type_name_for_cli(shape: &rhwp::model::shape::ShapeObject) -> &'static str {
    if shape
        .drawing()
        .and_then(|drawing| drawing.text_box.as_ref())
        .is_some()
    {
        return "TextBox";
    }
    match shape {
        rhwp::model::shape::ShapeObject::Line(_) => "Line",
        rhwp::model::shape::ShapeObject::Rectangle(_) => "Rectangle",
        rhwp::model::shape::ShapeObject::Ellipse(_) => "Ellipse",
        rhwp::model::shape::ShapeObject::Arc(_) => "Arc",
        rhwp::model::shape::ShapeObject::Polygon(_) => "Polygon",
        rhwp::model::shape::ShapeObject::Curve(_) => "Curve",
        rhwp::model::shape::ShapeObject::Group(_) => "Group",
        rhwp::model::shape::ShapeObject::Picture(_) => "Picture",
        rhwp::model::shape::ShapeObject::Chart(_) => "Chart",
        rhwp::model::shape::ShapeObject::Ole(_) => "Ole",
    }
}

fn text_wrap_name_for_cli(wrap: rhwp::model::shape::TextWrap) -> &'static str {
    match wrap {
        rhwp::model::shape::TextWrap::Square => "Square",
        rhwp::model::shape::TextWrap::Tight => "Tight",
        rhwp::model::shape::TextWrap::Through => "Through",
        rhwp::model::shape::TextWrap::TopAndBottom => "TopAndBottom",
        rhwp::model::shape::TextWrap::BehindText => "BehindText",
        rhwp::model::shape::TextWrap::InFrontOfText => "InFrontOfText",
    }
}

fn collect_body_forms_for_cli(
    forms: &mut Vec<serde_json::Value>,
    section_index: usize,
    paragraph_index: usize,
    controls: &[rhwp::model::control::Control],
) {
    for (control_index, control) in controls.iter().enumerate() {
        match control {
            rhwp::model::control::Control::Form(form) => {
                let mut item = form_json_for_cli(form);
                if let serde_json::Value::Object(ref mut obj) = item {
                    obj.insert("container".to_string(), serde_json::json!("body"));
                    obj.insert("section".to_string(), serde_json::json!(section_index));
                    obj.insert("paragraph".to_string(), serde_json::json!(paragraph_index));
                    obj.insert("control".to_string(), serde_json::json!(control_index));
                }
                forms.push(item);
            }
            rhwp::model::control::Control::Table(table) => {
                collect_cell_forms_for_cli(
                    forms,
                    section_index,
                    paragraph_index,
                    control_index,
                    table,
                );
            }
            rhwp::model::control::Control::Shape(shape) => {
                collect_textbox_forms_for_cli(
                    forms,
                    section_index,
                    paragraph_index,
                    control_index,
                    shape,
                );
            }
            _ => {}
        }
    }
}

fn collect_cell_forms_for_cli(
    forms: &mut Vec<serde_json::Value>,
    section_index: usize,
    paragraph_index: usize,
    table_control_index: usize,
    table: &rhwp::model::table::Table,
) {
    for (cell_index, cell) in table.cells.iter().enumerate() {
        for (cell_paragraph_index, paragraph) in cell.paragraphs.iter().enumerate() {
            for (control_index, control) in paragraph.controls.iter().enumerate() {
                match control {
                    rhwp::model::control::Control::Form(form) => {
                        let mut item = form_json_for_cli(form);
                        if let serde_json::Value::Object(ref mut obj) = item {
                            obj.insert("container".to_string(), serde_json::json!("cell"));
                            obj.insert("section".to_string(), serde_json::json!(section_index));
                            obj.insert("paragraph".to_string(), serde_json::json!(paragraph_index));
                            obj.insert("control".to_string(), serde_json::json!(control_index));
                            obj.insert(
                                "tableControl".to_string(),
                                serde_json::json!(table_control_index),
                            );
                            obj.insert("cellIndex".to_string(), serde_json::json!(cell_index));
                            obj.insert("row".to_string(), serde_json::json!(cell.row));
                            obj.insert("col".to_string(), serde_json::json!(cell.col));
                            obj.insert(
                                "cellParagraph".to_string(),
                                serde_json::json!(cell_paragraph_index),
                            );
                            obj.insert(
                                "cellPath".to_string(),
                                serde_json::json!([{
                                    "controlIndex": table_control_index,
                                    "cellIndex": cell_index,
                                    "cellParaIndex": cell_paragraph_index,
                                }]),
                            );
                        }
                        forms.push(item);
                    }
                    rhwp::model::control::Control::Shape(shape) => {
                        collect_cell_textbox_forms_for_cli(
                            forms,
                            section_index,
                            paragraph_index,
                            table_control_index,
                            cell_index,
                            cell.row,
                            cell.col,
                            cell_paragraph_index,
                            control_index,
                            shape,
                        );
                    }
                    _ => {}
                }
            }
        }
    }
}

fn collect_textbox_forms_for_cli(
    forms: &mut Vec<serde_json::Value>,
    section_index: usize,
    paragraph_index: usize,
    shape_control_index: usize,
    shape: &rhwp::model::shape::ShapeObject,
) {
    if let Some(textbox) = shape
        .drawing()
        .and_then(|drawing| drawing.text_box.as_ref())
    {
        for (textbox_paragraph_index, paragraph) in textbox.paragraphs.iter().enumerate() {
            for (control_index, control) in paragraph.controls.iter().enumerate() {
                if let rhwp::model::control::Control::Form(form) = control {
                    let mut item = form_json_for_cli(form);
                    if let serde_json::Value::Object(ref mut obj) = item {
                        obj.insert("container".to_string(), serde_json::json!("textbox"));
                        obj.insert("section".to_string(), serde_json::json!(section_index));
                        obj.insert("paragraph".to_string(), serde_json::json!(paragraph_index));
                        obj.insert("control".to_string(), serde_json::json!(control_index));
                        obj.insert(
                            "shapeControl".to_string(),
                            serde_json::json!(shape_control_index),
                        );
                        obj.insert(
                            "textboxParagraph".to_string(),
                            serde_json::json!(textbox_paragraph_index),
                        );
                    }
                    forms.push(item);
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn collect_cell_textbox_forms_for_cli(
    forms: &mut Vec<serde_json::Value>,
    section_index: usize,
    paragraph_index: usize,
    table_control_index: usize,
    cell_index: usize,
    row: u16,
    col: u16,
    cell_paragraph_index: usize,
    shape_control_index: usize,
    shape: &rhwp::model::shape::ShapeObject,
) {
    if let Some(textbox) = shape
        .drawing()
        .and_then(|drawing| drawing.text_box.as_ref())
    {
        for (textbox_paragraph_index, paragraph) in textbox.paragraphs.iter().enumerate() {
            for (control_index, control) in paragraph.controls.iter().enumerate() {
                if let rhwp::model::control::Control::Form(form) = control {
                    let mut item = form_json_for_cli(form);
                    if let serde_json::Value::Object(ref mut obj) = item {
                        obj.insert("container".to_string(), serde_json::json!("cell_textbox"));
                        obj.insert("section".to_string(), serde_json::json!(section_index));
                        obj.insert("paragraph".to_string(), serde_json::json!(paragraph_index));
                        obj.insert("control".to_string(), serde_json::json!(control_index));
                        obj.insert(
                            "tableControl".to_string(),
                            serde_json::json!(table_control_index),
                        );
                        obj.insert("cellIndex".to_string(), serde_json::json!(cell_index));
                        obj.insert("row".to_string(), serde_json::json!(row));
                        obj.insert("col".to_string(), serde_json::json!(col));
                        obj.insert(
                            "cellParagraph".to_string(),
                            serde_json::json!(cell_paragraph_index),
                        );
                        obj.insert(
                            "shapeControl".to_string(),
                            serde_json::json!(shape_control_index),
                        );
                        obj.insert(
                            "textboxParagraph".to_string(),
                            serde_json::json!(textbox_paragraph_index),
                        );
                        obj.insert(
                            "cellPath".to_string(),
                            serde_json::json!([{
                                "controlIndex": table_control_index,
                                "cellIndex": cell_index,
                                "cellParaIndex": cell_paragraph_index,
                            }]),
                        );
                    }
                    forms.push(item);
                }
            }
        }
    }
}

fn form_json_for_cli(form: &rhwp::model::control::FormObject) -> serde_json::Value {
    serde_json::json!({
        "formType": form_type_name_for_cli(form.form_type),
        "name": form.name,
        "caption": form.caption,
        "text": form.text,
        "value": form.value,
        "enabled": form.enabled,
        "width": form.width,
        "height": form.height,
        "foreColor": form.fore_color,
        "backColor": form.back_color,
        "properties": form.properties,
    })
}

fn form_type_name_for_cli(form_type: rhwp::model::control::FormType) -> &'static str {
    match form_type {
        rhwp::model::control::FormType::PushButton => "PushButton",
        rhwp::model::control::FormType::CheckBox => "CheckBox",
        rhwp::model::control::FormType::ComboBox => "ComboBox",
        rhwp::model::control::FormType::RadioButton => "RadioButton",
        rhwp::model::control::FormType::Edit => "Edit",
    }
}

fn insert_hwp_clickhere_field_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
    char_offset: usize,
    name: &str,
    guide: &str,
    memo: &str,
    value: &str,
) -> Result<HwpFieldCliResult, String> {
    if name.trim().is_empty() {
        return Err("필드명은 비어 있을 수 없습니다.".to_string());
    }
    let mut core = rhwp::document_core::DocumentCore::from_bytes(data)
        .map_err(|e| format!("HWP 파싱 실패: {}", e))?;
    core.convert_to_editable_native()
        .map_err(|e| format!("편집 가능 변환 실패: {}", e))?;
    let details_json = core
        .insert_clickhere_field_native(section_idx, para_idx, char_offset, name, guide, memo, value)
        .map_err(|e| format!("누름틀 필드 생성 실패: {}", e))?;
    let verification = core
        .serialize_hwp_with_verify()
        .map_err(|e| format!("HWP 직렬화/재로드 검증 실패: {}", e))?;
    if !verification.recovered {
        return Err(format!(
            "HWP 재로드 검증 실패: page_count_before={}, page_count_after={}",
            verification.page_count_before, verification.page_count_after
        ));
    }
    Ok(HwpFieldCliResult {
        bytes: verification.bytes,
        details: parse_json_value(&details_json),
        page_count_before: verification.page_count_before,
        page_count_after: verification.page_count_after,
    })
}

fn get_hwp_field_info_json_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
    char_offset: usize,
) -> Result<serde_json::Value, String> {
    let core = rhwp::document_core::DocumentCore::from_bytes(data)
        .map_err(|e| format!("HWP 파싱 실패: {}", e))?;
    let mut info = parse_json_value(&core.get_field_info_at(section_idx, para_idx, char_offset));
    if let serde_json::Value::Object(ref mut obj) = info {
        obj.insert("ok".to_string(), serde_json::Value::Bool(true));
    }
    Ok(info)
}

fn remove_hwp_field_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
    char_offset: usize,
) -> Result<HwpFieldCliResult, String> {
    let mut core = rhwp::document_core::DocumentCore::from_bytes(data)
        .map_err(|e| format!("HWP 파싱 실패: {}", e))?;
    core.convert_to_editable_native()
        .map_err(|e| format!("편집 가능 변환 실패: {}", e))?;
    let details_json = core
        .remove_field_at(section_idx, para_idx, char_offset)
        .map_err(|e| format!("누름틀 필드 제거 실패: {}", e))?;
    let verification = core
        .serialize_hwp_with_verify()
        .map_err(|e| format!("HWP 직렬화/재로드 검증 실패: {}", e))?;
    if !verification.recovered {
        return Err(format!(
            "HWP 재로드 검증 실패: page_count_before={}, page_count_after={}",
            verification.page_count_before, verification.page_count_after
        ));
    }
    Ok(HwpFieldCliResult {
        bytes: verification.bytes,
        details: parse_json_value(&details_json),
        page_count_before: verification.page_count_before,
        page_count_after: verification.page_count_after,
    })
}

fn insert_hwp_nested_clickhere_field_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    parent_para_idx: usize,
    control_idx: usize,
    cell_idx: usize,
    cell_para_idx: usize,
    char_offset: usize,
    is_textbox: bool,
    name: &str,
    guide: &str,
    memo: &str,
    value: &str,
) -> Result<HwpEditCliResult, String> {
    if name.trim().is_empty() {
        return Err("필드명은 비어 있을 수 없습니다.".to_string());
    }
    edit_hwp_table_structure_bytes_for_cli(data, "insert-clickhere-field", |core| {
        core.insert_clickhere_field_in_cell_native(
            section_idx,
            parent_para_idx,
            control_idx,
            cell_idx,
            cell_para_idx,
            char_offset,
            is_textbox,
            name,
            guide,
            memo,
            value,
        )
        .map_err(|e| format!("누름틀 필드 생성 실패: {}", e))
    })
}

#[allow(clippy::too_many_arguments)]
fn insert_hwp_clickhere_field_by_path_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    parent_para_idx: usize,
    cell_path_json: &str,
    control_idx: usize,
    textbox_para_idx: usize,
    char_offset: usize,
    name: &str,
    guide: &str,
    memo: &str,
    value: &str,
) -> Result<HwpEditCliResult, String> {
    if name.trim().is_empty() {
        return Err("필드명은 비어 있을 수 없습니다.".to_string());
    }
    let mut cell_path = parse_cell_path_for_cli(cell_path_json)?;
    if cell_path.is_empty() {
        return Err("cell-path는 비어 있을 수 없습니다.".to_string());
    }
    cell_path.push((control_idx, 0, textbox_para_idx));
    edit_hwp_table_structure_bytes_for_cli(data, "insert-clickhere-field", |core| {
        core.insert_clickhere_field_by_path_native(
            section_idx,
            parent_para_idx,
            &cell_path,
            char_offset,
            name,
            guide,
            memo,
            value,
        )
        .map_err(|e| format!("누름틀 필드 생성 실패: {}", e))
    })
}

fn get_hwp_nested_field_info_json_for_cli(
    data: &[u8],
    section_idx: usize,
    parent_para_idx: usize,
    control_idx: usize,
    cell_idx: usize,
    cell_para_idx: usize,
    char_offset: usize,
    is_textbox: bool,
) -> Result<serde_json::Value, String> {
    let core = rhwp::document_core::DocumentCore::from_bytes(data)
        .map_err(|e| format!("HWP 파싱 실패: {}", e))?;
    let mut info = parse_json_value(&core.get_field_info_at_in_cell(
        section_idx,
        parent_para_idx,
        control_idx,
        cell_idx,
        cell_para_idx,
        char_offset,
        is_textbox,
    ));
    if let serde_json::Value::Object(ref mut obj) = info {
        obj.insert("ok".to_string(), serde_json::Value::Bool(true));
    }
    Ok(info)
}

fn cell_textbox_path_for_field_cli(
    cell_path_json: &str,
    control_idx: usize,
    textbox_para_idx: usize,
) -> Result<Vec<(usize, usize, usize)>, String> {
    let mut cell_path = parse_cell_path_for_cli(cell_path_json)?;
    if cell_path.is_empty() {
        return Err("cell-path는 비어 있을 수 없습니다.".to_string());
    }
    cell_path.push((control_idx, 0, textbox_para_idx));
    Ok(cell_path)
}

fn get_hwp_field_info_by_path_json_for_cli(
    data: &[u8],
    section_idx: usize,
    parent_para_idx: usize,
    cell_path_json: &str,
    control_idx: usize,
    textbox_para_idx: usize,
    char_offset: usize,
) -> Result<serde_json::Value, String> {
    let cell_path = cell_textbox_path_for_field_cli(cell_path_json, control_idx, textbox_para_idx)?;
    let core = rhwp::document_core::DocumentCore::from_bytes(data)
        .map_err(|e| format!("HWP 파싱 실패: {}", e))?;
    let mut info = parse_json_value(&core.get_field_info_at_by_path(
        section_idx,
        parent_para_idx,
        &cell_path,
        char_offset,
    ));
    if let serde_json::Value::Object(ref mut obj) = info {
        obj.insert("ok".to_string(), serde_json::Value::Bool(true));
        obj.insert(
            "cellPath".to_string(),
            serde_json::Value::Array(
                cell_path
                    .iter()
                    .map(|(control_index, cell_index, cell_para_index)| {
                        serde_json::json!({
                            "controlIndex": control_index,
                            "cellIndex": cell_index,
                            "cellParaIndex": cell_para_index,
                        })
                    })
                    .collect(),
            ),
        );
    }
    Ok(info)
}

fn remove_hwp_nested_field_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    parent_para_idx: usize,
    control_idx: usize,
    cell_idx: usize,
    cell_para_idx: usize,
    char_offset: usize,
    is_textbox: bool,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "remove-field", |core| {
        core.remove_field_at_in_cell(
            section_idx,
            parent_para_idx,
            control_idx,
            cell_idx,
            cell_para_idx,
            char_offset,
            is_textbox,
        )
        .map_err(|e| format!("누름틀 필드 제거 실패: {}", e))
    })
}

fn remove_hwp_field_by_path_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    parent_para_idx: usize,
    cell_path_json: &str,
    control_idx: usize,
    textbox_para_idx: usize,
    char_offset: usize,
) -> Result<HwpEditCliResult, String> {
    let cell_path = cell_textbox_path_for_field_cli(cell_path_json, control_idx, textbox_para_idx)?;
    edit_hwp_table_structure_bytes_for_cli(data, "remove-field", |core| {
        core.remove_field_at_by_path(section_idx, parent_para_idx, &cell_path, char_offset)
            .map_err(|e| format!("누름틀 필드 제거 실패: {}", e))
    })
}

#[allow(clippy::too_many_arguments)]
fn insert_hwp_cell_shape_clickhere_field_at_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    parent_para_idx: usize,
    table_control_idx: usize,
    row: u16,
    col: u16,
    cell_para_idx: usize,
    control_idx: usize,
    textbox_para_idx: usize,
    char_offset: usize,
    name: &str,
    guide: &str,
    memo: &str,
    value: &str,
) -> Result<HwpEditCliResult, String> {
    let (cell_path, cell_idx) = cell_path_from_row_col_for_cli(
        data,
        section_idx,
        parent_para_idx,
        table_control_idx,
        row,
        col,
        cell_para_idx,
    )?;
    let mut result = insert_hwp_clickhere_field_by_path_bytes_for_cli(
        data,
        section_idx,
        parent_para_idx,
        &cell_path,
        control_idx,
        textbox_para_idx,
        char_offset,
        name,
        guide,
        memo,
        value,
    )?;
    if let Some(obj) = result.details.as_object_mut() {
        obj.insert("row".to_string(), serde_json::json!(row));
        obj.insert("col".to_string(), serde_json::json!(col));
        obj.insert("cellIndex".to_string(), serde_json::json!(cell_idx));
        obj.insert(
            "tableControl".to_string(),
            serde_json::json!(table_control_idx),
        );
        obj.insert(
            "cellParaIndex".to_string(),
            serde_json::json!(cell_para_idx),
        );
    }
    Ok(result)
}

#[allow(clippy::too_many_arguments)]
fn get_hwp_cell_shape_field_info_at_json_for_cli(
    data: &[u8],
    section_idx: usize,
    parent_para_idx: usize,
    table_control_idx: usize,
    row: u16,
    col: u16,
    cell_para_idx: usize,
    control_idx: usize,
    textbox_para_idx: usize,
    char_offset: usize,
) -> Result<serde_json::Value, String> {
    let (cell_path, cell_idx) = cell_path_from_row_col_for_cli(
        data,
        section_idx,
        parent_para_idx,
        table_control_idx,
        row,
        col,
        cell_para_idx,
    )?;
    let mut info = get_hwp_field_info_by_path_json_for_cli(
        data,
        section_idx,
        parent_para_idx,
        &cell_path,
        control_idx,
        textbox_para_idx,
        char_offset,
    )?;
    if let Some(obj) = info.as_object_mut() {
        obj.insert("row".to_string(), serde_json::json!(row));
        obj.insert("col".to_string(), serde_json::json!(col));
        obj.insert("cellIndex".to_string(), serde_json::json!(cell_idx));
        obj.insert(
            "tableControl".to_string(),
            serde_json::json!(table_control_idx),
        );
        obj.insert(
            "cellParaIndex".to_string(),
            serde_json::json!(cell_para_idx),
        );
    }
    Ok(info)
}

#[allow(clippy::too_many_arguments)]
fn remove_hwp_cell_shape_field_at_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    parent_para_idx: usize,
    table_control_idx: usize,
    row: u16,
    col: u16,
    cell_para_idx: usize,
    control_idx: usize,
    textbox_para_idx: usize,
    char_offset: usize,
) -> Result<HwpEditCliResult, String> {
    let (cell_path, cell_idx) = cell_path_from_row_col_for_cli(
        data,
        section_idx,
        parent_para_idx,
        table_control_idx,
        row,
        col,
        cell_para_idx,
    )?;
    let mut result = remove_hwp_field_by_path_bytes_for_cli(
        data,
        section_idx,
        parent_para_idx,
        &cell_path,
        control_idx,
        textbox_para_idx,
        char_offset,
    )?;
    if let Some(obj) = result.details.as_object_mut() {
        obj.insert("row".to_string(), serde_json::json!(row));
        obj.insert("col".to_string(), serde_json::json!(col));
        obj.insert("cellIndex".to_string(), serde_json::json!(cell_idx));
        obj.insert(
            "tableControl".to_string(),
            serde_json::json!(table_control_idx),
        );
        obj.insert(
            "cellParaIndex".to_string(),
            serde_json::json!(cell_para_idx),
        );
    }
    Ok(result)
}

fn set_hwp_field_bytes_for_cli(
    data: &[u8],
    name: &str,
    value: &str,
) -> Result<HwpFieldCliResult, String> {
    if name.is_empty() {
        return Err("필드명은 비어 있을 수 없습니다.".to_string());
    }
    let mut core = rhwp::document_core::DocumentCore::from_bytes(data)
        .map_err(|e| format!("HWP 파싱 실패: {}", e))?;
    core.convert_to_editable_native()
        .map_err(|e| format!("편집 가능 변환 실패: {}", e))?;
    let details_json = core
        .set_field_value_by_name(name, value)
        .map_err(|e| format!("필드 설정 실패: {}", e))?;
    let verification = core
        .serialize_hwp_with_verify()
        .map_err(|e| format!("HWP 직렬화/재로드 검증 실패: {}", e))?;
    if !verification.recovered {
        return Err(format!(
            "HWP 재로드 검증 실패: page_count_before={}, page_count_after={}",
            verification.page_count_before, verification.page_count_after
        ));
    }
    Ok(HwpFieldCliResult {
        bytes: verification.bytes,
        details: parse_json_value(&details_json),
        page_count_before: verification.page_count_before,
        page_count_after: verification.page_count_after,
    })
}

fn parse_form_type_for_cli(form_type: &str) -> Result<rhwp::model::control::FormType, String> {
    match form_type.trim().to_ascii_lowercase().as_str() {
        "button" | "pushbutton" | "push-button" => Ok(rhwp::model::control::FormType::PushButton),
        "checkbox" | "check" | "check-box" => Ok(rhwp::model::control::FormType::CheckBox),
        "combo" | "combobox" | "combo-box" => Ok(rhwp::model::control::FormType::ComboBox),
        "radio" | "radiobutton" | "radio-button" => Ok(rhwp::model::control::FormType::RadioButton),
        "edit" | "input" | "text" => Ok(rhwp::model::control::FormType::Edit),
        _ => Err(format!("지원하지 않는 form-type: {}", form_type)),
    }
}

fn create_hwp_form_object_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
    char_offset: usize,
    form_type: &str,
    name: &str,
    caption: &str,
    text: &str,
    width: u32,
    height: u32,
    value: i32,
    enabled: bool,
    properties_json: &str,
) -> Result<HwpTableCliResult, String> {
    if name.trim().is_empty() {
        return Err("양식 개체 이름은 비어 있을 수 없습니다.".to_string());
    }
    if width == 0 || height == 0 {
        return Err("양식 개체 width/height는 0보다 커야 합니다.".to_string());
    }
    let form_type = parse_form_type_for_cli(form_type)?;
    let mut core = rhwp::document_core::DocumentCore::from_bytes(data)
        .map_err(|e| format!("HWP 파싱 실패: {}", e))?;
    core.convert_to_editable_native()
        .map_err(|e| format!("편집 가능 변환 실패: {}", e))?;
    let details_json = core
        .create_form_object_native(
            section_idx,
            para_idx,
            char_offset,
            form_type,
            name,
            caption,
            text,
            width,
            height,
            value,
            enabled,
            properties_json,
        )
        .map_err(|e| format!("양식 개체 생성 실패: {}", e))?;
    let details = parse_json_value(&details_json);
    let para_idx = details
        .get("paraIdx")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| format!("양식 개체 생성 결과에 paraIdx가 없습니다: {}", details_json))?
        as usize;
    let control_idx = details
        .get("controlIdx")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| {
            format!(
                "양식 개체 생성 결과에 controlIdx가 없습니다: {}",
                details_json
            )
        })? as usize;
    let (bytes, page_count_before, page_count_after) = serialize_hwp_verified_for_cli(&mut core)?;
    Ok(HwpTableCliResult {
        bytes,
        para_idx,
        control_idx,
        details,
        page_count_before,
        page_count_after,
    })
}

#[allow(clippy::too_many_arguments)]
fn create_hwp_cell_form_object_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    parent_para_idx: usize,
    cell_path_json: &str,
    char_offset: usize,
    form_type: &str,
    name: &str,
    caption: &str,
    text: &str,
    width: u32,
    height: u32,
    value: i32,
    enabled: bool,
    properties_json: &str,
) -> Result<HwpTableCliResult, String> {
    if name.trim().is_empty() {
        return Err("양식 개체 이름은 비어 있을 수 없습니다.".to_string());
    }
    if width == 0 || height == 0 {
        return Err("양식 개체 width/height는 0보다 커야 합니다.".to_string());
    }
    let form_type = parse_form_type_for_cli(form_type)?;
    let cell_path = parse_cell_path_for_cli(cell_path_json)?;
    if cell_path.is_empty() {
        return Err("cell-path는 비어 있을 수 없습니다.".to_string());
    }
    let mut core = rhwp::document_core::DocumentCore::from_bytes(data)
        .map_err(|e| format!("HWP 파싱 실패: {}", e))?;
    core.convert_to_editable_native()
        .map_err(|e| format!("편집 가능 변환 실패: {}", e))?;
    let details_json = core
        .create_cell_form_object_native(
            section_idx,
            parent_para_idx,
            &cell_path,
            char_offset,
            form_type,
            name,
            caption,
            text,
            width,
            height,
            value,
            enabled,
            properties_json,
        )
        .map_err(|e| format!("셀 양식 개체 생성 실패: {}", e))?;
    let details = parse_json_value(&details_json);
    let para_idx = details
        .get("paraIdx")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| format!("양식 개체 생성 결과에 paraIdx가 없습니다: {}", details_json))?
        as usize;
    let control_idx = details
        .get("controlIdx")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| {
            format!(
                "양식 개체 생성 결과에 controlIdx가 없습니다: {}",
                details_json
            )
        })? as usize;
    let (bytes, page_count_before, page_count_after) = serialize_hwp_verified_for_cli(&mut core)?;
    Ok(HwpTableCliResult {
        bytes,
        para_idx,
        control_idx,
        details,
        page_count_before,
        page_count_after,
    })
}

fn get_hwp_form_info_json_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
    control_idx: usize,
) -> Result<serde_json::Value, String> {
    let core = rhwp::document_core::DocumentCore::from_bytes(data)
        .map_err(|e| format!("HWP 파싱 실패: {}", e))?;
    Ok(parse_json_value(
        &core
            .get_form_object_info_native(section_idx, para_idx, control_idx)
            .map_err(|e| format!("양식 개체 조회 실패: {}", e))?,
    ))
}

fn get_hwp_cell_form_info_json_for_cli(
    data: &[u8],
    section_idx: usize,
    parent_para_idx: usize,
    cell_path_json: &str,
    control_idx: usize,
) -> Result<serde_json::Value, String> {
    let cell_path = parse_cell_path_for_cli(cell_path_json)?;
    if cell_path.is_empty() {
        return Err("cell-path는 비어 있을 수 없습니다.".to_string());
    }
    let core = rhwp::document_core::DocumentCore::from_bytes(data)
        .map_err(|e| format!("HWP 파싱 실패: {}", e))?;
    Ok(parse_json_value(
        &core
            .get_cell_form_object_info_native(section_idx, parent_para_idx, &cell_path, control_idx)
            .map_err(|e| format!("셀 양식 개체 조회 실패: {}", e))?,
    ))
}

fn set_hwp_form_value_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
    control_idx: usize,
    value_json: &str,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "set-form", |core| {
        core.set_form_value_native(section_idx, para_idx, control_idx, value_json)
            .map_err(|e| format!("양식 개체 설정 실패: {}", e))
    })
}

fn set_hwp_cell_form_value_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    parent_para_idx: usize,
    cell_path_json: &str,
    control_idx: usize,
    value_json: &str,
) -> Result<HwpEditCliResult, String> {
    let cell_path = parse_cell_path_for_cli(cell_path_json)?;
    if cell_path.is_empty() {
        return Err("cell-path는 비어 있을 수 없습니다.".to_string());
    }
    edit_hwp_table_structure_bytes_for_cli(data, "set-form", |core| {
        core.set_cell_form_value_by_path_native(
            section_idx,
            parent_para_idx,
            &cell_path,
            control_idx,
            value_json,
        )
        .map_err(|e| format!("셀 양식 개체 설정 실패: {}", e))
    })
}

#[allow(clippy::too_many_arguments)]
fn create_hwp_cell_form_object_at_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    parent_para_idx: usize,
    table_control_idx: usize,
    row: u16,
    col: u16,
    cell_para_idx: usize,
    char_offset: usize,
    form_type: &str,
    name: &str,
    caption: &str,
    text: &str,
    width: u32,
    height: u32,
    value: i32,
    enabled: bool,
    properties_json: &str,
) -> Result<HwpTableCliResult, String> {
    let (cell_path, cell_idx) = cell_path_from_row_col_for_cli(
        data,
        section_idx,
        parent_para_idx,
        table_control_idx,
        row,
        col,
        cell_para_idx,
    )?;
    let mut result = create_hwp_cell_form_object_bytes_for_cli(
        data,
        section_idx,
        parent_para_idx,
        &cell_path,
        char_offset,
        form_type,
        name,
        caption,
        text,
        width,
        height,
        value,
        enabled,
        properties_json,
    )?;
    if let Some(obj) = result.details.as_object_mut() {
        obj.insert("container".to_string(), serde_json::json!("cell"));
        obj.insert("row".to_string(), serde_json::json!(row));
        obj.insert("col".to_string(), serde_json::json!(col));
        obj.insert("cellIndex".to_string(), serde_json::json!(cell_idx));
        obj.insert(
            "cellParaIndex".to_string(),
            serde_json::json!(cell_para_idx),
        );
        obj.insert(
            "tableControl".to_string(),
            serde_json::json!(table_control_idx),
        );
    }
    Ok(result)
}

#[allow(clippy::too_many_arguments)]
fn get_hwp_cell_form_info_at_json_for_cli(
    data: &[u8],
    section_idx: usize,
    parent_para_idx: usize,
    table_control_idx: usize,
    row: u16,
    col: u16,
    cell_para_idx: usize,
    control_idx: usize,
) -> Result<serde_json::Value, String> {
    let (cell_path, cell_idx) = cell_path_from_row_col_for_cli(
        data,
        section_idx,
        parent_para_idx,
        table_control_idx,
        row,
        col,
        cell_para_idx,
    )?;
    let mut info = get_hwp_cell_form_info_json_for_cli(
        data,
        section_idx,
        parent_para_idx,
        &cell_path,
        control_idx,
    )?;
    if let Some(obj) = info.as_object_mut() {
        obj.insert("container".to_string(), serde_json::json!("cell"));
        obj.insert("row".to_string(), serde_json::json!(row));
        obj.insert("col".to_string(), serde_json::json!(col));
        obj.insert("cellIndex".to_string(), serde_json::json!(cell_idx));
        obj.insert(
            "cellParaIndex".to_string(),
            serde_json::json!(cell_para_idx),
        );
        obj.insert(
            "tableControl".to_string(),
            serde_json::json!(table_control_idx),
        );
    }
    Ok(info)
}

#[allow(clippy::too_many_arguments)]
fn set_hwp_cell_form_value_at_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    parent_para_idx: usize,
    table_control_idx: usize,
    row: u16,
    col: u16,
    cell_para_idx: usize,
    control_idx: usize,
    value_json: &str,
) -> Result<HwpEditCliResult, String> {
    let (cell_path, cell_idx) = cell_path_from_row_col_for_cli(
        data,
        section_idx,
        parent_para_idx,
        table_control_idx,
        row,
        col,
        cell_para_idx,
    )?;
    let mut result = set_hwp_cell_form_value_bytes_for_cli(
        data,
        section_idx,
        parent_para_idx,
        &cell_path,
        control_idx,
        value_json,
    )?;
    if let Some(obj) = result.details.as_object_mut() {
        obj.insert("container".to_string(), serde_json::json!("cell"));
        obj.insert("row".to_string(), serde_json::json!(row));
        obj.insert("col".to_string(), serde_json::json!(col));
        obj.insert("cellIndex".to_string(), serde_json::json!(cell_idx));
        obj.insert(
            "cellParaIndex".to_string(),
            serde_json::json!(cell_para_idx),
        );
        obj.insert(
            "tableControl".to_string(),
            serde_json::json!(table_control_idx),
        );
    }
    Ok(result)
}

fn delete_hwp_form_object_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
    control_idx: usize,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "delete-form", |core| {
        core.delete_form_object_native(section_idx, para_idx, control_idx)
            .map_err(|e| format!("양식 개체 삭제 실패: {}", e))
    })
}

fn delete_hwp_cell_form_object_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    parent_para_idx: usize,
    cell_path_json: &str,
    control_idx: usize,
) -> Result<HwpEditCliResult, String> {
    let cell_path = parse_cell_path_for_cli(cell_path_json)?;
    if cell_path.is_empty() {
        return Err("cell-path는 비어 있을 수 없습니다.".to_string());
    }
    edit_hwp_table_structure_bytes_for_cli(data, "delete-form", |core| {
        core.delete_cell_form_object_by_path_native(
            section_idx,
            parent_para_idx,
            &cell_path,
            control_idx,
        )
        .map_err(|e| format!("셀 양식 개체 삭제 실패: {}", e))
    })
}

#[allow(clippy::too_many_arguments)]
fn delete_hwp_cell_form_object_at_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    parent_para_idx: usize,
    table_control_idx: usize,
    row: u16,
    col: u16,
    cell_para_idx: usize,
    control_idx: usize,
) -> Result<HwpEditCliResult, String> {
    let (cell_path, cell_idx) = cell_path_from_row_col_for_cli(
        data,
        section_idx,
        parent_para_idx,
        table_control_idx,
        row,
        col,
        cell_para_idx,
    )?;
    let mut result = delete_hwp_cell_form_object_bytes_for_cli(
        data,
        section_idx,
        parent_para_idx,
        &cell_path,
        control_idx,
    )?;
    if let Some(obj) = result.details.as_object_mut() {
        obj.insert("container".to_string(), serde_json::json!("cell"));
        obj.insert("row".to_string(), serde_json::json!(row));
        obj.insert("col".to_string(), serde_json::json!(col));
        obj.insert("cellIndex".to_string(), serde_json::json!(cell_idx));
        obj.insert(
            "cellParaIndex".to_string(),
            serde_json::json!(cell_para_idx),
        );
        obj.insert(
            "tableControl".to_string(),
            serde_json::json!(table_control_idx),
        );
    }
    Ok(result)
}

fn hwp_note_paragraphs_json_for_cli(
    paragraphs: &[rhwp::model::paragraph::Paragraph],
) -> Vec<serde_json::Value> {
    paragraphs
        .iter()
        .enumerate()
        .map(|(index, para)| {
            serde_json::json!({
                "index": index,
                "text": para.text,
                "charCount": para.text.chars().count(),
                "controlCount": para.controls.len(),
            })
        })
        .collect()
}

fn hwp_note_control_json_for_cli(
    control_index: usize,
    control: &rhwp::model::control::Control,
) -> Option<serde_json::Value> {
    match control {
        rhwp::model::control::Control::Footnote(note) => {
            let texts: Vec<&str> = note
                .paragraphs
                .iter()
                .map(|para| para.text.as_str())
                .collect();
            Some(serde_json::json!({
                "kind": "footnote",
                "type": "footnote",
                "controlIndex": control_index,
                "number": note.number,
                "paragraphCount": note.paragraphs.len(),
                "paragraphs": hwp_note_paragraphs_json_for_cli(&note.paragraphs),
                "texts": texts,
            }))
        }
        rhwp::model::control::Control::Endnote(note) => {
            let texts: Vec<&str> = note
                .paragraphs
                .iter()
                .map(|para| para.text.as_str())
                .collect();
            Some(serde_json::json!({
                "kind": "endnote",
                "type": "endnote",
                "controlIndex": control_index,
                "number": note.number,
                "paragraphCount": note.paragraphs.len(),
                "paragraphs": hwp_note_paragraphs_json_for_cli(&note.paragraphs),
                "texts": texts,
            }))
        }
        _ => None,
    }
}

fn extract_hwp_structure_json_for_cli(data: &[u8]) -> Result<serde_json::Value, String> {
    let core = rhwp::document_core::DocumentCore::from_bytes(data)
        .map_err(|e| format!("HWP 파싱 실패: {}", e))?;
    let mut sections = Vec::new();
    for (section_index, section) in core.document().sections.iter().enumerate() {
        let paragraphs: Vec<serde_json::Value> = section
            .paragraphs
            .iter()
            .enumerate()
            .map(|(index, para)| {
                let controls: Vec<serde_json::Value> = para
                    .controls
                    .iter()
                    .enumerate()
                    .filter_map(|(control_index, control)| {
                        hwp_note_control_json_for_cli(control_index, control)
                    })
                    .collect();
                serde_json::json!({
                    "index": index,
                    "text": para.text,
                    "charCount": para.text.chars().count(),
                    "controlCount": para.controls.len(),
                    "controls": controls,
                })
            })
            .collect();

        let mut tables = Vec::new();
        let mut shapes = Vec::new();
        for (para_index, para) in section.paragraphs.iter().enumerate() {
            for (control_index, control) in para.controls.iter().enumerate() {
                match control {
                    rhwp::model::control::Control::Table(table) => {
                        let cells: Vec<serde_json::Value> = table
                            .cells
                            .iter()
                            .enumerate()
                            .map(|(cell_index, cell)| {
                                let text = cell
                                    .paragraphs
                                    .iter()
                                    .map(|p| p.text.as_str())
                                    .collect::<Vec<_>>()
                                    .join("\n");
                                let mut cell_shapes = Vec::new();
                                for (cell_paragraph_index, paragraph) in
                                    cell.paragraphs.iter().enumerate()
                                {
                                    for (shape_control_index, cell_control) in
                                        paragraph.controls.iter().enumerate()
                                    {
                                        if let rhwp::model::control::Control::Shape(shape) =
                                            cell_control
                                        {
                                            let mut item = shape_object_json_for_cli(
                                                "cell",
                                                section_index,
                                                para_index,
                                                shape_control_index,
                                                shape.as_ref(),
                                            );
                                            if let serde_json::Value::Object(ref mut obj) = item {
                                                obj.insert(
                                                    "tableControl".to_string(),
                                                    serde_json::json!(control_index),
                                                );
                                                obj.insert(
                                                    "cellIndex".to_string(),
                                                    serde_json::json!(cell_index),
                                                );
                                                obj.insert(
                                                    "row".to_string(),
                                                    serde_json::json!(cell.row),
                                                );
                                                obj.insert(
                                                    "col".to_string(),
                                                    serde_json::json!(cell.col),
                                                );
                                                obj.insert(
                                                    "cellParagraph".to_string(),
                                                    serde_json::json!(cell_paragraph_index),
                                                );
                                                obj.insert(
                                                    "shapeControl".to_string(),
                                                    serde_json::json!(shape_control_index),
                                                );
                                                obj.insert(
                                                    "cellPath".to_string(),
                                                    serde_json::json!([{
                                                        "controlIndex": control_index,
                                                        "cellIndex": cell_index,
                                                        "cellParaIndex": cell_paragraph_index,
                                                    }]),
                                                );
                                                if let Some(textbox) = shape
                                                    .drawing()
                                                    .and_then(|drawing| drawing.text_box.as_ref())
                                                {
                                                    obj.insert(
                                                        "textBox".to_string(),
                                                        textbox_json_for_cli(textbox),
                                                    );
                                                }
                                            }
                                            cell_shapes.push(item);
                                        }
                                    }
                                }
                                serde_json::json!({
                                    "index": cell_index,
                                    "row": cell.row,
                                    "col": cell.col,
                                    "rowSpan": cell.row_span,
                                    "colSpan": cell.col_span,
                                    "paragraphCount": cell.paragraphs.len(),
                                    "text": text,
                                    "shapeCount": cell_shapes.len(),
                                    "shapes": cell_shapes,
                                })
                            })
                            .collect();
                        tables.push(serde_json::json!({
                            "paragraphIndex": para_index,
                            "controlIndex": control_index,
                            "rowCount": table.row_count,
                            "colCount": table.col_count,
                            "cellCount": table.cells.len(),
                            "cells": cells,
                        }));
                    }
                    rhwp::model::control::Control::Shape(shape) => {
                        let mut item = shape_object_json_for_cli(
                            "body",
                            section_index,
                            para_index,
                            control_index,
                            shape.as_ref(),
                        );
                        if let serde_json::Value::Object(ref mut obj) = item {
                            obj.insert("paragraphIndex".to_string(), serde_json::json!(para_index));
                            obj.insert(
                                "controlIndex".to_string(),
                                serde_json::json!(control_index),
                            );
                            if let Some(textbox) = shape
                                .drawing()
                                .and_then(|drawing| drawing.text_box.as_ref())
                            {
                                obj.insert("textBox".to_string(), textbox_json_for_cli(textbox));
                            }
                        }
                        shapes.push(item);
                    }
                    _ => {}
                }
            }
        }

        sections.push(serde_json::json!({
            "index": section_index,
            "paragraphCount": section.paragraphs.len(),
            "paragraphs": paragraphs,
            "tableCount": tables.len(),
            "tables": tables,
            "shapeCount": shapes.len(),
            "shapes": shapes,
        }));
    }
    Ok(serde_json::json!({
        "ok": true,
        "sectionCount": core.document().sections.len(),
        "pageCount": core.page_count(),
        "sections": sections,
    }))
}

fn set_hwp_paragraph_text_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
    text: &str,
) -> Result<HwpEditCliResult, String> {
    let mut core = rhwp::document_core::DocumentCore::from_bytes(data)
        .map_err(|e| format!("HWP 파싱 실패: {}", e))?;
    core.convert_to_editable_native()
        .map_err(|e| format!("편집 가능 변환 실패: {}", e))?;
    let len = core
        .get_paragraph_length_native(section_idx, para_idx)
        .map_err(|e| format!("문단 길이 조회 실패: {}", e))?;
    if len > 0 {
        core.delete_text_native(section_idx, para_idx, 0, len)
            .map_err(|e| format!("문단 텍스트 삭제 실패: {}", e))?;
    }
    core.insert_text_native(section_idx, para_idx, 0, text)
        .map_err(|e| format!("문단 텍스트 삽입 실패: {}", e))?;
    let (bytes, page_count_before, page_count_after) = serialize_hwp_verified_for_cli(&mut core)?;
    Ok(HwpEditCliResult {
        bytes,
        details: serde_json::json!({
            "ok": true,
            "section": section_idx,
            "paragraph": para_idx,
            "text": text,
        }),
        page_count_before,
        page_count_after,
    })
}

fn insert_hwp_text_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
    char_offset: usize,
    text: &str,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "insert-text", |core| {
        core.insert_text_native(section_idx, para_idx, char_offset, text)
            .map_err(|e| format!("본문 텍스트 삽입 실패: {}", e))
    })
}

fn delete_hwp_text_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
    char_offset: usize,
    count: usize,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "delete-text", |core| {
        core.delete_text_native(section_idx, para_idx, char_offset, count)
            .map_err(|e| format!("본문 텍스트 삭제 실패: {}", e))
    })
}

fn insert_hwp_paragraph_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
    text: Option<&str>,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "insert-paragraph", |core| {
        let details_json = core
            .insert_paragraph_native(section_idx, para_idx)
            .map_err(|e| format!("문단 삽입 실패: {}", e))?;
        if let Some(text) = text {
            if !text.is_empty() {
                core.insert_text_native(section_idx, para_idx, 0, text)
                    .map_err(|e| format!("삽입 문단 텍스트 설정 실패: {}", e))?;
            }
        }
        let mut details = parse_json_value(&details_json);
        if let Some(obj) = details.as_object_mut() {
            obj.insert("section".to_string(), serde_json::json!(section_idx));
            if let Some(text) = text {
                obj.insert("text".to_string(), serde_json::json!(text));
            }
        }
        serde_json::to_string(&details).map_err(|e| format!("문단 삽입 결과 JSON 생성 실패: {}", e))
    })
}

fn copy_hwp_paragraph_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
    after: bool,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "copy-paragraph", |core| {
        core.copy_paragraph_native(section_idx, para_idx, after)
            .map_err(|e| format!("문단 복제 실패: {}", e))
    })
}

fn copy_hwp_paragraph_range_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    start_para_idx: usize,
    end_para_idx: usize,
    after: bool,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "copy-paragraph-range", |core| {
        core.copy_paragraph_range_native(section_idx, start_para_idx, end_para_idx, after)
            .map_err(|e| format!("문단 범위 복제 실패: {}", e))
    })
}

fn copy_hwp_paragraph_range_with_replacements_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    start_para_idx: usize,
    end_para_idx: usize,
    after: bool,
    replacements: &[(String, String)],
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "copy-paragraph-range", |core| {
        core.copy_paragraph_range_with_replacements_native(
            section_idx,
            start_para_idx,
            end_para_idx,
            after,
            replacements,
        )
        .map_err(|e| format!("문단 범위 복제/치환 실패: {}", e))
    })
}

fn split_hwp_paragraph_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
    char_offset: usize,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "split-paragraph", |core| {
        core.split_paragraph_native(section_idx, para_idx, char_offset)
            .map_err(|e| format!("문단 분할 실패: {}", e))
    })
}

fn merge_hwp_paragraph_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "merge-paragraph", |core| {
        core.merge_paragraph_native(section_idx, para_idx)
            .map_err(|e| format!("문단 병합 실패: {}", e))
    })
}

fn delete_hwp_paragraph_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "delete-paragraph", |core| {
        core.delete_paragraph_native(section_idx, para_idx)
            .map_err(|e| format!("문단 삭제 실패: {}", e))
    })
}

fn insert_hwp_page_break_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
    char_offset: usize,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "insert-page-break", |core| {
        core.insert_page_break_native(section_idx, para_idx, char_offset)
            .map_err(|e| format!("쪽 나누기 삽입 실패: {}", e))
    })
}

fn insert_hwp_column_break_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
    char_offset: usize,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "insert-column-break", |core| {
        core.insert_column_break_native(section_idx, para_idx, char_offset)
            .map_err(|e| format!("단 나누기 삽입 실패: {}", e))
    })
}

fn set_hwp_column_def_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    column_count: u16,
    column_type: u8,
    same_width: bool,
    spacing_hu: i16,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "set-column-def", |core| {
        let details_json = core
            .set_column_def_native(
                section_idx,
                column_count,
                column_type,
                same_width,
                spacing_hu,
            )
            .map_err(|e| format!("다단 설정 실패: {}", e))?;
        let mut details = parse_json_value(&details_json);
        if let Some(obj) = details.as_object_mut() {
            obj.insert("section".to_string(), serde_json::json!(section_idx));
            obj.insert("columnCount".to_string(), serde_json::json!(column_count));
            obj.insert("columnType".to_string(), serde_json::json!(column_type));
            obj.insert("sameWidth".to_string(), serde_json::json!(same_width));
            obj.insert("spacing".to_string(), serde_json::json!(spacing_hu));
        }
        serde_json::to_string(&details).map_err(|e| format!("다단 설정 결과 JSON 생성 실패: {}", e))
    })
}

fn insert_hwp_new_number_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
    char_offset: usize,
    start_number: u16,
) -> Result<HwpEditCliResult, String> {
    if start_number == 0 {
        return Err("새 쪽 번호 시작값은 1 이상이어야 합니다.".to_string());
    }
    edit_hwp_table_structure_bytes_for_cli(data, "insert-new-number", |core| {
        let details_json = core
            .insert_new_number_native(section_idx, para_idx, char_offset, start_number)
            .map_err(|e| format!("새 쪽 번호 삽입 실패: {}", e))?;
        let mut details = parse_json_value(&details_json);
        if let Some(obj) = details.as_object_mut() {
            obj.insert("section".to_string(), serde_json::json!(section_idx));
            obj.insert("paragraph".to_string(), serde_json::json!(para_idx));
            obj.insert("offset".to_string(), serde_json::json!(char_offset));
            obj.insert("startNumber".to_string(), serde_json::json!(start_number));
        }
        serde_json::to_string(&details)
            .map_err(|e| format!("새 쪽 번호 결과 JSON 생성 실패: {}", e))
    })
}

fn get_hwp_page_hide_json_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
) -> Result<serde_json::Value, String> {
    let core = rhwp::document_core::DocumentCore::from_bytes(data)
        .map_err(|e| format!("HWP 파싱 실패: {}", e))?;
    let details_json = core
        .get_page_hide_native(section_idx, para_idx)
        .map_err(|e| format!("쪽 감추기 조회 실패: {}", e))?;
    let mut details = parse_json_value(&details_json);
    if let Some(obj) = details.as_object_mut() {
        obj.insert("section".to_string(), serde_json::json!(section_idx));
        obj.insert("paragraph".to_string(), serde_json::json!(para_idx));
    }
    Ok(details)
}

fn set_hwp_page_hide_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
    hide_header: bool,
    hide_footer: bool,
    hide_master_page: bool,
    hide_border: bool,
    hide_fill: bool,
    hide_page_num: bool,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "set-page-hide", |core| {
        let details_json = core
            .set_page_hide_native(
                section_idx,
                para_idx,
                hide_header,
                hide_footer,
                hide_master_page,
                hide_border,
                hide_fill,
                hide_page_num,
            )
            .map_err(|e| format!("쪽 감추기 설정 실패: {}", e))?;
        let mut details = parse_json_value(&details_json);
        if let Some(obj) = details.as_object_mut() {
            obj.insert("section".to_string(), serde_json::json!(section_idx));
            obj.insert("paragraph".to_string(), serde_json::json!(para_idx));
            obj.insert("hideHeader".to_string(), serde_json::json!(hide_header));
            obj.insert("hideFooter".to_string(), serde_json::json!(hide_footer));
            obj.insert(
                "hideMasterPage".to_string(),
                serde_json::json!(hide_master_page),
            );
            obj.insert("hideBorder".to_string(), serde_json::json!(hide_border));
            obj.insert("hideFill".to_string(), serde_json::json!(hide_fill));
            obj.insert("hidePageNum".to_string(), serde_json::json!(hide_page_num));
        }
        serde_json::to_string(&details).map_err(|e| format!("쪽 감추기 결과 JSON 생성 실패: {}", e))
    })
}

fn ensure_hwp_json_ok_for_cli(details: &serde_json::Value, action: &str) -> Result<(), String> {
    if details.get("ok").and_then(|v| v.as_bool()) == Some(false) {
        let message = details
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("알 수 없는 오류");
        return Err(format!("{} 실패: {}", action, message));
    }
    Ok(())
}

fn get_hwp_bookmarks_json_for_cli(data: &[u8]) -> Result<serde_json::Value, String> {
    let core = rhwp::document_core::DocumentCore::from_bytes(data)
        .map_err(|e| format!("HWP 파싱 실패: {}", e))?;
    let bookmarks_json = core
        .get_bookmarks_native()
        .map_err(|e| format!("책갈피 목록 조회 실패: {}", e))?;
    let bookmarks = parse_json_value(&bookmarks_json);
    Ok(serde_json::json!({
        "ok": true,
        "bookmarks": bookmarks,
    }))
}

fn add_hwp_bookmark_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
    char_offset: usize,
    name: &str,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "add-bookmark", |core| {
        let details_json = core
            .add_bookmark_native(section_idx, para_idx, char_offset, name)
            .map_err(|e| format!("책갈피 추가 실패: {}", e))?;
        let mut details = parse_json_value(&details_json);
        ensure_hwp_json_ok_for_cli(&details, "책갈피 추가")?;
        if let Some(section) = core.document_mut().sections.get_mut(section_idx) {
            section.raw_stream = None;
        }
        if let Some(obj) = details.as_object_mut() {
            obj.insert("section".to_string(), serde_json::json!(section_idx));
            obj.insert("paragraph".to_string(), serde_json::json!(para_idx));
            obj.insert("offset".to_string(), serde_json::json!(char_offset));
            obj.insert("name".to_string(), serde_json::json!(name));
        }
        serde_json::to_string(&details)
            .map_err(|e| format!("책갈피 추가 결과 JSON 생성 실패: {}", e))
    })
}

fn rename_hwp_bookmark_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
    control_idx: usize,
    name: &str,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "rename-bookmark", |core| {
        let details_json = core
            .rename_bookmark_native(section_idx, para_idx, control_idx, name)
            .map_err(|e| format!("책갈피 이름 변경 실패: {}", e))?;
        let mut details = parse_json_value(&details_json);
        ensure_hwp_json_ok_for_cli(&details, "책갈피 이름 변경")?;
        if let Some(section) = core.document_mut().sections.get_mut(section_idx) {
            section.raw_stream = None;
        }
        if let Some(obj) = details.as_object_mut() {
            obj.insert("section".to_string(), serde_json::json!(section_idx));
            obj.insert("paragraph".to_string(), serde_json::json!(para_idx));
            obj.insert("control".to_string(), serde_json::json!(control_idx));
            obj.insert("name".to_string(), serde_json::json!(name));
        }
        serde_json::to_string(&details)
            .map_err(|e| format!("책갈피 이름 변경 결과 JSON 생성 실패: {}", e))
    })
}

fn delete_hwp_bookmark_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
    control_idx: usize,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "delete-bookmark", |core| {
        let details_json = core
            .delete_bookmark_native(section_idx, para_idx, control_idx)
            .map_err(|e| format!("책갈피 삭제 실패: {}", e))?;
        let mut details = parse_json_value(&details_json);
        ensure_hwp_json_ok_for_cli(&details, "책갈피 삭제")?;
        if let Some(section) = core.document_mut().sections.get_mut(section_idx) {
            section.raw_stream = None;
        }
        if let Some(obj) = details.as_object_mut() {
            obj.insert("section".to_string(), serde_json::json!(section_idx));
            obj.insert("paragraph".to_string(), serde_json::json!(para_idx));
            obj.insert("control".to_string(), serde_json::json!(control_idx));
        }
        serde_json::to_string(&details)
            .map_err(|e| format!("책갈피 삭제 결과 JSON 생성 실패: {}", e))
    })
}

fn create_hwp_note_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
    char_offset: usize,
    is_endnote: bool,
    text: Option<&str>,
) -> Result<HwpEditCliResult, String> {
    let operation = if is_endnote {
        "create-endnote"
    } else {
        "create-footnote"
    };
    edit_hwp_table_structure_bytes_for_cli(data, operation, |core| {
        let details_json = if is_endnote {
            core.insert_endnote_native(section_idx, para_idx, char_offset)
                .map_err(|e| format!("미주 생성 실패: {}", e))?
        } else {
            core.insert_footnote_native(section_idx, para_idx, char_offset)
                .map_err(|e| format!("각주 생성 실패: {}", e))?
        };
        let mut details = parse_json_value(&details_json);
        let control_idx = details
            .get("controlIdx")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| "각주/미주 생성 결과에서 controlIdx를 찾을 수 없습니다.".to_string())?
            as usize;
        if let Some(text) = text {
            if !text.is_empty() {
                core.insert_text_in_footnote_native(section_idx, para_idx, control_idx, 0, 2, text)
                    .map_err(|e| format!("각주/미주 내용 입력 실패: {}", e))?;
            }
        }
        if let Some(obj) = details.as_object_mut() {
            obj.insert(
                "kind".to_string(),
                serde_json::json!(if is_endnote { "endnote" } else { "footnote" }),
            );
            obj.insert("section".to_string(), serde_json::json!(section_idx));
            obj.insert("paragraph".to_string(), serde_json::json!(para_idx));
            obj.insert("offset".to_string(), serde_json::json!(char_offset));
            if let Some(text) = text {
                obj.insert("text".to_string(), serde_json::json!(text));
            }
        }
        serde_json::to_string(&details)
            .map_err(|e| format!("각주/미주 생성 결과 JSON 생성 실패: {}", e))
    })
}

fn get_hwp_footnote_info_json_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
    control_idx: usize,
) -> Result<serde_json::Value, String> {
    let core = rhwp::document_core::DocumentCore::from_bytes(data)
        .map_err(|e| format!("HWP 파싱 실패: {}", e))?;
    let info_json = core
        .get_footnote_info_native(section_idx, para_idx, control_idx)
        .map_err(|e| format!("각주/미주 정보 조회 실패: {}", e))?;
    let mut info = parse_json_value(&info_json);
    if let Some(obj) = info.as_object_mut() {
        obj.insert("section".to_string(), serde_json::json!(section_idx));
        obj.insert("paragraph".to_string(), serde_json::json!(para_idx));
        obj.insert("control".to_string(), serde_json::json!(control_idx));
    }
    Ok(info)
}

fn insert_hwp_footnote_text_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
    control_idx: usize,
    note_para_idx: usize,
    char_offset: usize,
    text: &str,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "insert-footnote-text", |core| {
        core.insert_text_in_footnote_native(
            section_idx,
            para_idx,
            control_idx,
            note_para_idx,
            char_offset,
            text,
        )
        .map_err(|e| format!("각주/미주 텍스트 삽입 실패: {}", e))
    })
}

fn delete_hwp_footnote_text_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
    control_idx: usize,
    note_para_idx: usize,
    char_offset: usize,
    count: usize,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "delete-footnote-text", |core| {
        core.delete_text_in_footnote_native(
            section_idx,
            para_idx,
            control_idx,
            note_para_idx,
            char_offset,
            count,
        )
        .map_err(|e| format!("각주/미주 텍스트 삭제 실패: {}", e))
    })
}

fn split_hwp_footnote_paragraph_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
    control_idx: usize,
    note_para_idx: usize,
    char_offset: usize,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "split-footnote-paragraph", |core| {
        core.split_paragraph_in_footnote_native(
            section_idx,
            para_idx,
            control_idx,
            note_para_idx,
            char_offset,
        )
        .map_err(|e| format!("각주/미주 문단 분할 실패: {}", e))
    })
}

fn merge_hwp_footnote_paragraph_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
    control_idx: usize,
    note_para_idx: usize,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "merge-footnote-paragraph", |core| {
        core.merge_paragraph_in_footnote_native(section_idx, para_idx, control_idx, note_para_idx)
            .map_err(|e| format!("각주/미주 문단 병합 실패: {}", e))
    })
}

fn delete_hwp_footnote_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
    control_idx: usize,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "delete-footnote", |core| {
        core.delete_footnote_native(section_idx, para_idx, control_idx)
            .map_err(|e| format!("각주 삭제 실패: {}", e))
    })
}

fn create_hwp_table_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
    char_offset: usize,
    rows: u16,
    cols: u16,
) -> Result<HwpTableCliResult, String> {
    let mut core = rhwp::document_core::DocumentCore::from_bytes(data)
        .map_err(|e| format!("HWP 파싱 실패: {}", e))?;
    core.convert_to_editable_native()
        .map_err(|e| format!("편집 가능 변환 실패: {}", e))?;
    let details_json = core
        .create_table_native(section_idx, para_idx, char_offset, rows, cols)
        .map_err(|e| format!("표 생성 실패: {}", e))?;
    let details = parse_json_value(&details_json);
    let para_idx = details
        .get("paraIdx")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| format!("표 생성 결과에 paraIdx가 없습니다: {}", details_json))?
        as usize;
    let control_idx = details
        .get("controlIdx")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;
    let (bytes, page_count_before, page_count_after) = serialize_hwp_verified_for_cli(&mut core)?;
    Ok(HwpTableCliResult {
        bytes,
        para_idx,
        control_idx,
        details,
        page_count_before,
        page_count_after,
    })
}

fn copy_hwp_table_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    table_para_idx: usize,
    control_idx: usize,
    after: bool,
) -> Result<HwpEditCliResult, String> {
    let mut core = rhwp::document_core::DocumentCore::from_bytes(data)
        .map_err(|e| format!("HWP 파싱 실패: {}", e))?;
    core.convert_to_editable_native()
        .map_err(|e| format!("편집 가능 변환 실패: {}", e))?;
    let details_json = core
        .copy_table_native(section_idx, table_para_idx, control_idx, after)
        .map_err(|e| format!("표 복제 실패: {}", e))?;
    let mut details = parse_json_value(&details_json);
    if let Some(obj) = details.as_object_mut() {
        obj.insert("operation".to_string(), serde_json::json!("copy-table"));
    }
    let (bytes, page_count_before, page_count_after) = serialize_hwp_verified_for_cli(&mut core)?;
    Ok(HwpEditCliResult {
        bytes,
        details,
        page_count_before,
        page_count_after,
    })
}

fn copy_hwp_table_with_replacements_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    table_para_idx: usize,
    control_idx: usize,
    after: bool,
    replacements: &[(String, String)],
) -> Result<HwpEditCliResult, String> {
    let mut core = rhwp::document_core::DocumentCore::from_bytes(data)
        .map_err(|e| format!("HWP 파싱 실패: {}", e))?;
    core.convert_to_editable_native()
        .map_err(|e| format!("편집 가능 변환 실패: {}", e))?;
    let details_json = core
        .copy_table_with_replacements_native(
            section_idx,
            table_para_idx,
            control_idx,
            after,
            replacements,
        )
        .map_err(|e| format!("표 복제/치환 실패: {}", e))?;
    let mut details = parse_json_value(&details_json);
    if let Some(obj) = details.as_object_mut() {
        obj.insert("operation".to_string(), serde_json::json!("copy-table"));
    }
    let (bytes, page_count_before, page_count_after) = serialize_hwp_verified_for_cli(&mut core)?;
    Ok(HwpEditCliResult {
        bytes,
        details,
        page_count_before,
        page_count_after,
    })
}

fn delete_hwp_table_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    table_para_idx: usize,
    control_idx: usize,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "delete-table", |core| {
        core.delete_table_native(section_idx, table_para_idx, control_idx)
            .map_err(|e| format!("표 삭제 실패: {}", e))
    })
}

fn set_hwp_cell_text_bytes_for_cli(
    data: &[u8],
    table_para_idx: usize,
    control_idx: usize,
    cell_idx: usize,
    cell_para_idx: usize,
    text: &str,
) -> Result<HwpEditCliResult, String> {
    let mut core = rhwp::document_core::DocumentCore::from_bytes(data)
        .map_err(|e| format!("HWP 파싱 실패: {}", e))?;
    core.convert_to_editable_native()
        .map_err(|e| format!("편집 가능 변환 실패: {}", e))?;
    let len = core
        .get_cell_paragraph_length_native(0, table_para_idx, control_idx, cell_idx, cell_para_idx)
        .map_err(|e| format!("셀 문단 길이 조회 실패: {}", e))?;
    if len > 0 {
        core.delete_text_in_cell_native(
            0,
            table_para_idx,
            control_idx,
            cell_idx,
            cell_para_idx,
            0,
            len,
        )
        .map_err(|e| format!("셀 텍스트 삭제 실패: {}", e))?;
    }
    core.insert_text_in_cell_native(
        0,
        table_para_idx,
        control_idx,
        cell_idx,
        cell_para_idx,
        0,
        text,
    )
    .map_err(|e| format!("셀 텍스트 삽입 실패: {}", e))?;
    let (bytes, page_count_before, page_count_after) = serialize_hwp_verified_for_cli(&mut core)?;
    Ok(HwpEditCliResult {
        bytes,
        details: serde_json::json!({
            "ok": true,
            "tableParagraph": table_para_idx,
            "control": control_idx,
            "cell": cell_idx,
            "cellParagraph": cell_para_idx,
            "text": text,
        }),
        page_count_before,
        page_count_after,
    })
}

fn set_hwp_cell_text_by_position_bytes_for_cli(
    data: &[u8],
    table_para_idx: usize,
    control_idx: usize,
    row: u16,
    col: u16,
    cell_para_idx: usize,
    text: &str,
) -> Result<HwpEditCliResult, String> {
    let mut core = rhwp::document_core::DocumentCore::from_bytes(data)
        .map_err(|e| format!("HWP 파싱 실패: {}", e))?;
    core.convert_to_editable_native()
        .map_err(|e| format!("편집 가능 변환 실패: {}", e))?;
    let cell_idx = core
        .get_table_cell_index_native(0, table_para_idx, control_idx, row, col)
        .map_err(|e| format!("셀 좌표 조회 실패: {}", e))?;
    let len = core
        .get_cell_paragraph_length_native(0, table_para_idx, control_idx, cell_idx, cell_para_idx)
        .map_err(|e| format!("셀 문단 길이 조회 실패: {}", e))?;
    if len > 0 {
        core.delete_text_in_cell_native(
            0,
            table_para_idx,
            control_idx,
            cell_idx,
            cell_para_idx,
            0,
            len,
        )
        .map_err(|e| format!("셀 텍스트 삭제 실패: {}", e))?;
    }
    core.insert_text_in_cell_native(
        0,
        table_para_idx,
        control_idx,
        cell_idx,
        cell_para_idx,
        0,
        text,
    )
    .map_err(|e| format!("셀 텍스트 삽입 실패: {}", e))?;
    let (bytes, page_count_before, page_count_after) = serialize_hwp_verified_for_cli(&mut core)?;
    Ok(HwpEditCliResult {
        bytes,
        details: serde_json::json!({
            "ok": true,
            "tableParagraph": table_para_idx,
            "control": control_idx,
            "cell": cell_idx,
            "row": row,
            "col": col,
            "cellParagraph": cell_para_idx,
            "text": text,
        }),
        page_count_before,
        page_count_after,
    })
}

fn add_cell_text_edit_details(
    details_json: &str,
    table_para_idx: usize,
    control_idx: usize,
    cell_idx: usize,
    cell_para_idx: usize,
    row_col: Option<(u16, u16)>,
) -> String {
    let mut value = parse_json_value(details_json);
    if let Some(obj) = value.as_object_mut() {
        obj.insert(
            "tableParagraph".to_string(),
            serde_json::json!(table_para_idx),
        );
        obj.insert("control".to_string(), serde_json::json!(control_idx));
        obj.insert("cell".to_string(), serde_json::json!(cell_idx));
        obj.insert(
            "cellParagraph".to_string(),
            serde_json::json!(cell_para_idx),
        );
        if let Some((row, col)) = row_col {
            obj.insert("row".to_string(), serde_json::json!(row));
            obj.insert("col".to_string(), serde_json::json!(col));
        }
    }
    value.to_string()
}

fn insert_hwp_cell_text_bytes_for_cli(
    data: &[u8],
    table_para_idx: usize,
    control_idx: usize,
    cell_idx: usize,
    cell_para_idx: usize,
    char_offset: usize,
    text: &str,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "insert-cell-text", |core| {
        let details = core
            .insert_text_in_cell_native(
                0,
                table_para_idx,
                control_idx,
                cell_idx,
                cell_para_idx,
                char_offset,
                text,
            )
            .map_err(|e| format!("셀 텍스트 삽입 실패: {}", e))?;
        Ok(add_cell_text_edit_details(
            &details,
            table_para_idx,
            control_idx,
            cell_idx,
            cell_para_idx,
            None,
        ))
    })
}

fn delete_hwp_cell_text_bytes_for_cli(
    data: &[u8],
    table_para_idx: usize,
    control_idx: usize,
    cell_idx: usize,
    cell_para_idx: usize,
    char_offset: usize,
    count: usize,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "delete-cell-text", |core| {
        let details = core
            .delete_text_in_cell_native(
                0,
                table_para_idx,
                control_idx,
                cell_idx,
                cell_para_idx,
                char_offset,
                count,
            )
            .map_err(|e| format!("셀 텍스트 삭제 실패: {}", e))?;
        Ok(add_cell_text_edit_details(
            &details,
            table_para_idx,
            control_idx,
            cell_idx,
            cell_para_idx,
            None,
        ))
    })
}

fn insert_hwp_cell_text_by_position_bytes_for_cli(
    data: &[u8],
    table_para_idx: usize,
    control_idx: usize,
    row: u16,
    col: u16,
    cell_para_idx: usize,
    char_offset: usize,
    text: &str,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "insert-cell-text", |core| {
        let cell_idx = core
            .get_table_cell_index_native(0, table_para_idx, control_idx, row, col)
            .map_err(|e| format!("셀 좌표 조회 실패: {}", e))?;
        let details = core
            .insert_text_in_cell_native(
                0,
                table_para_idx,
                control_idx,
                cell_idx,
                cell_para_idx,
                char_offset,
                text,
            )
            .map_err(|e| format!("셀 텍스트 삽입 실패: {}", e))?;
        Ok(add_cell_text_edit_details(
            &details,
            table_para_idx,
            control_idx,
            cell_idx,
            cell_para_idx,
            Some((row, col)),
        ))
    })
}

fn delete_hwp_cell_text_by_position_bytes_for_cli(
    data: &[u8],
    table_para_idx: usize,
    control_idx: usize,
    row: u16,
    col: u16,
    cell_para_idx: usize,
    char_offset: usize,
    count: usize,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "delete-cell-text", |core| {
        let cell_idx = core
            .get_table_cell_index_native(0, table_para_idx, control_idx, row, col)
            .map_err(|e| format!("셀 좌표 조회 실패: {}", e))?;
        let details = core
            .delete_text_in_cell_native(
                0,
                table_para_idx,
                control_idx,
                cell_idx,
                cell_para_idx,
                char_offset,
                count,
            )
            .map_err(|e| format!("셀 텍스트 삭제 실패: {}", e))?;
        Ok(add_cell_text_edit_details(
            &details,
            table_para_idx,
            control_idx,
            cell_idx,
            cell_para_idx,
            Some((row, col)),
        ))
    })
}

fn split_hwp_cell_paragraph_bytes_for_cli(
    data: &[u8],
    table_para_idx: usize,
    control_idx: usize,
    cell_idx: usize,
    cell_para_idx: usize,
    char_offset: usize,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "split-cell-paragraph", |core| {
        let details = core
            .split_paragraph_in_cell_native(
                0,
                table_para_idx,
                control_idx,
                cell_idx,
                cell_para_idx,
                char_offset,
            )
            .map_err(|e| format!("셀 문단 분할 실패: {}", e))?;
        Ok(add_cell_text_edit_details(
            &details,
            table_para_idx,
            control_idx,
            cell_idx,
            cell_para_idx,
            None,
        ))
    })
}

fn merge_hwp_cell_paragraph_bytes_for_cli(
    data: &[u8],
    table_para_idx: usize,
    control_idx: usize,
    cell_idx: usize,
    cell_para_idx: usize,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "merge-cell-paragraph", |core| {
        let details = core
            .merge_paragraph_in_cell_native(0, table_para_idx, control_idx, cell_idx, cell_para_idx)
            .map_err(|e| format!("셀 문단 병합 실패: {}", e))?;
        Ok(add_cell_text_edit_details(
            &details,
            table_para_idx,
            control_idx,
            cell_idx,
            cell_para_idx,
            None,
        ))
    })
}

fn split_hwp_cell_paragraph_by_position_bytes_for_cli(
    data: &[u8],
    table_para_idx: usize,
    control_idx: usize,
    row: u16,
    col: u16,
    cell_para_idx: usize,
    char_offset: usize,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "split-cell-paragraph", |core| {
        let cell_idx = core
            .get_table_cell_index_native(0, table_para_idx, control_idx, row, col)
            .map_err(|e| format!("셀 좌표 조회 실패: {}", e))?;
        let details = core
            .split_paragraph_in_cell_native(
                0,
                table_para_idx,
                control_idx,
                cell_idx,
                cell_para_idx,
                char_offset,
            )
            .map_err(|e| format!("셀 문단 분할 실패: {}", e))?;
        Ok(add_cell_text_edit_details(
            &details,
            table_para_idx,
            control_idx,
            cell_idx,
            cell_para_idx,
            Some((row, col)),
        ))
    })
}

fn merge_hwp_cell_paragraph_by_position_bytes_for_cli(
    data: &[u8],
    table_para_idx: usize,
    control_idx: usize,
    row: u16,
    col: u16,
    cell_para_idx: usize,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "merge-cell-paragraph", |core| {
        let cell_idx = core
            .get_table_cell_index_native(0, table_para_idx, control_idx, row, col)
            .map_err(|e| format!("셀 좌표 조회 실패: {}", e))?;
        let details = core
            .merge_paragraph_in_cell_native(0, table_para_idx, control_idx, cell_idx, cell_para_idx)
            .map_err(|e| format!("셀 문단 병합 실패: {}", e))?;
        Ok(add_cell_text_edit_details(
            &details,
            table_para_idx,
            control_idx,
            cell_idx,
            cell_para_idx,
            Some((row, col)),
        ))
    })
}

fn insert_hwp_cell_paragraph_bytes_for_cli(
    data: &[u8],
    table_para_idx: usize,
    control_idx: usize,
    cell_idx: usize,
    cell_para_idx: usize,
    text: Option<&str>,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "insert-cell-paragraph", |core| {
        let details = core
            .insert_paragraph_in_cell_native(
                0,
                table_para_idx,
                control_idx,
                cell_idx,
                cell_para_idx,
                text,
            )
            .map_err(|e| format!("셀 문단 삽입 실패: {}", e))?;
        Ok(add_cell_text_edit_details(
            &details,
            table_para_idx,
            control_idx,
            cell_idx,
            cell_para_idx,
            None,
        ))
    })
}

fn delete_hwp_cell_paragraph_bytes_for_cli(
    data: &[u8],
    table_para_idx: usize,
    control_idx: usize,
    cell_idx: usize,
    cell_para_idx: usize,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "delete-cell-paragraph", |core| {
        let details = core
            .delete_paragraph_in_cell_native(
                0,
                table_para_idx,
                control_idx,
                cell_idx,
                cell_para_idx,
            )
            .map_err(|e| format!("셀 문단 삭제 실패: {}", e))?;
        Ok(add_cell_text_edit_details(
            &details,
            table_para_idx,
            control_idx,
            cell_idx,
            cell_para_idx,
            None,
        ))
    })
}

fn insert_hwp_cell_paragraph_by_position_bytes_for_cli(
    data: &[u8],
    table_para_idx: usize,
    control_idx: usize,
    row: u16,
    col: u16,
    cell_para_idx: usize,
    text: Option<&str>,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "insert-cell-paragraph", |core| {
        let cell_idx = core
            .get_table_cell_index_native(0, table_para_idx, control_idx, row, col)
            .map_err(|e| format!("셀 좌표 조회 실패: {}", e))?;
        let details = core
            .insert_paragraph_in_cell_native(
                0,
                table_para_idx,
                control_idx,
                cell_idx,
                cell_para_idx,
                text,
            )
            .map_err(|e| format!("셀 문단 삽입 실패: {}", e))?;
        Ok(add_cell_text_edit_details(
            &details,
            table_para_idx,
            control_idx,
            cell_idx,
            cell_para_idx,
            Some((row, col)),
        ))
    })
}

fn delete_hwp_cell_paragraph_by_position_bytes_for_cli(
    data: &[u8],
    table_para_idx: usize,
    control_idx: usize,
    row: u16,
    col: u16,
    cell_para_idx: usize,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "delete-cell-paragraph", |core| {
        let cell_idx = core
            .get_table_cell_index_native(0, table_para_idx, control_idx, row, col)
            .map_err(|e| format!("셀 좌표 조회 실패: {}", e))?;
        let details = core
            .delete_paragraph_in_cell_native(
                0,
                table_para_idx,
                control_idx,
                cell_idx,
                cell_para_idx,
            )
            .map_err(|e| format!("셀 문단 삭제 실패: {}", e))?;
        Ok(add_cell_text_edit_details(
            &details,
            table_para_idx,
            control_idx,
            cell_idx,
            cell_para_idx,
            Some((row, col)),
        ))
    })
}

fn set_hwp_cell_field_bytes_for_cli(
    data: &[u8],
    table_para_idx: usize,
    control_idx: usize,
    cell_idx: usize,
    name: Option<&str>,
) -> Result<HwpEditCliResult, String> {
    let operation = if name.is_some() {
        "set-cell-field"
    } else {
        "clear-cell-field"
    };
    edit_hwp_table_structure_bytes_for_cli(data, operation, |core| {
        core.set_cell_field_name_native(0, table_para_idx, control_idx, cell_idx, name)
            .map_err(|e| format!("셀 필드 설정 실패: {}", e))
    })
}

fn set_hwp_cell_field_by_position_bytes_for_cli(
    data: &[u8],
    table_para_idx: usize,
    control_idx: usize,
    row: u16,
    col: u16,
    name: Option<&str>,
) -> Result<HwpEditCliResult, String> {
    let operation = if name.is_some() {
        "set-cell-field"
    } else {
        "clear-cell-field"
    };
    edit_hwp_table_structure_bytes_for_cli(data, operation, |core| {
        let cell_idx = core
            .get_table_cell_index_native(0, table_para_idx, control_idx, row, col)
            .map_err(|e| format!("셀 좌표 조회 실패: {}", e))?;
        let details = core
            .set_cell_field_name_native(0, table_para_idx, control_idx, cell_idx, name)
            .map_err(|e| format!("셀 필드 설정 실패: {}", e))?;
        let mut value = parse_json_value(&details);
        if let Some(obj) = value.as_object_mut() {
            obj.insert("row".to_string(), serde_json::json!(row));
            obj.insert("col".to_string(), serde_json::json!(col));
        }
        Ok(value.to_string())
    })
}

fn edit_hwp_table_structure_bytes_for_cli<F>(
    data: &[u8],
    operation: &str,
    edit: F,
) -> Result<HwpEditCliResult, String>
where
    F: FnOnce(&mut rhwp::document_core::DocumentCore) -> Result<String, String>,
{
    let mut core = rhwp::document_core::DocumentCore::from_bytes(data)
        .map_err(|e| format!("HWP 파싱 실패: {}", e))?;
    core.convert_to_editable_native()
        .map_err(|e| format!("편집 가능 변환 실패: {}", e))?;
    let details_json = edit(&mut core)?;
    let mut details = parse_json_value(&details_json);
    if let Some(obj) = details.as_object_mut() {
        obj.insert("operation".to_string(), serde_json::json!(operation));
    }
    let (bytes, page_count_before, page_count_after) = serialize_hwp_verified_for_cli(&mut core)?;
    Ok(HwpEditCliResult {
        bytes,
        details,
        page_count_before,
        page_count_after,
    })
}

fn insert_hwp_table_row_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    table_para_idx: usize,
    control_idx: usize,
    row_idx: u16,
    below: bool,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "insert-table-row", |core| {
        core.insert_table_row_native(section_idx, table_para_idx, control_idx, row_idx, below)
            .map_err(|e| format!("표 행 삽입 실패: {}", e))
    })
}

fn copy_hwp_table_row_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    table_para_idx: usize,
    control_idx: usize,
    row_idx: u16,
    below: bool,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "copy-table-row", |core| {
        core.copy_table_row_native(section_idx, table_para_idx, control_idx, row_idx, below)
            .map_err(|e| format!("표 행 복제 실패: {}", e))
    })
}

fn copy_hwp_table_row_with_replacements_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    table_para_idx: usize,
    control_idx: usize,
    row_idx: u16,
    below: bool,
    replacements: &[(String, String)],
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "copy-table-row", |core| {
        core.copy_table_row_with_replacements_native(
            section_idx,
            table_para_idx,
            control_idx,
            row_idx,
            below,
            replacements,
        )
        .map_err(|e| format!("표 행 복제/치환 실패: {}", e))
    })
}

fn delete_hwp_table_row_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    table_para_idx: usize,
    control_idx: usize,
    row_idx: u16,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "delete-table-row", |core| {
        core.delete_table_row_native(section_idx, table_para_idx, control_idx, row_idx)
            .map_err(|e| format!("표 행 삭제 실패: {}", e))
    })
}

fn insert_hwp_table_column_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    table_para_idx: usize,
    control_idx: usize,
    col_idx: u16,
    right: bool,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "insert-table-column", |core| {
        core.insert_table_column_native(section_idx, table_para_idx, control_idx, col_idx, right)
            .map_err(|e| format!("표 열 삽입 실패: {}", e))
    })
}

fn copy_hwp_table_column_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    table_para_idx: usize,
    control_idx: usize,
    col_idx: u16,
    right: bool,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "copy-table-column", |core| {
        core.copy_table_column_native(section_idx, table_para_idx, control_idx, col_idx, right)
            .map_err(|e| format!("표 열 복제 실패: {}", e))
    })
}

fn copy_hwp_table_column_with_replacements_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    table_para_idx: usize,
    control_idx: usize,
    col_idx: u16,
    right: bool,
    replacements: &[(String, String)],
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "copy-table-column", |core| {
        core.copy_table_column_with_replacements_native(
            section_idx,
            table_para_idx,
            control_idx,
            col_idx,
            right,
            replacements,
        )
        .map_err(|e| format!("표 열 복제/치환 실패: {}", e))
    })
}

fn delete_hwp_table_column_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    table_para_idx: usize,
    control_idx: usize,
    col_idx: u16,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "delete-table-column", |core| {
        core.delete_table_column_native(section_idx, table_para_idx, control_idx, col_idx)
            .map_err(|e| format!("표 열 삭제 실패: {}", e))
    })
}

fn merge_hwp_table_cells_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    table_para_idx: usize,
    control_idx: usize,
    start_row: u16,
    start_col: u16,
    end_row: u16,
    end_col: u16,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "merge-table-cells", |core| {
        core.merge_table_cells_native(
            section_idx,
            table_para_idx,
            control_idx,
            start_row,
            start_col,
            end_row,
            end_col,
        )
        .map_err(|e| format!("표 셀 병합 실패: {}", e))
    })
}

fn split_hwp_table_cell_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    table_para_idx: usize,
    control_idx: usize,
    row: u16,
    col: u16,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "split-table-cell", |core| {
        core.split_table_cell_native(section_idx, table_para_idx, control_idx, row, col)
            .map_err(|e| format!("표 셀 분할 실패: {}", e))
    })
}

fn get_hwp_cell_properties_json_for_cli(
    data: &[u8],
    section_idx: usize,
    table_para_idx: usize,
    control_idx: usize,
    cell_idx: usize,
) -> Result<serde_json::Value, String> {
    let core = rhwp::document_core::DocumentCore::from_bytes(data)
        .map_err(|e| format!("HWP 파싱 실패: {}", e))?;
    let details_json = core
        .get_cell_properties_native(section_idx, table_para_idx, control_idx, cell_idx)
        .map_err(|e| format!("셀 속성 조회 실패: {}", e))?;
    let mut details = parse_json_value(&details_json);
    if let Some(obj) = details.as_object_mut() {
        obj.insert("ok".to_string(), serde_json::json!(true));
    }
    Ok(details)
}

fn get_hwp_cell_properties_at_json_for_cli(
    data: &[u8],
    section_idx: usize,
    table_para_idx: usize,
    control_idx: usize,
    row: u16,
    col: u16,
) -> Result<serde_json::Value, String> {
    let core = rhwp::document_core::DocumentCore::from_bytes(data)
        .map_err(|e| format!("HWP 파싱 실패: {}", e))?;
    let cell_idx = core
        .get_table_cell_index_native(section_idx, table_para_idx, control_idx, row, col)
        .map_err(|e| format!("셀 좌표 조회 실패: {}", e))?;
    let details_json = core
        .get_cell_properties_native(section_idx, table_para_idx, control_idx, cell_idx)
        .map_err(|e| format!("셀 속성 조회 실패: {}", e))?;
    let mut details = parse_json_value(&details_json);
    if let Some(obj) = details.as_object_mut() {
        obj.insert("ok".to_string(), serde_json::json!(true));
        obj.insert("row".to_string(), serde_json::json!(row));
        obj.insert("col".to_string(), serde_json::json!(col));
        obj.insert("cellIndex".to_string(), serde_json::json!(cell_idx));
    }
    Ok(details)
}

fn set_hwp_cell_properties_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    table_para_idx: usize,
    control_idx: usize,
    cell_idx: usize,
    props_json: &str,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "set-cell-properties", |core| {
        core.set_cell_properties_native(
            section_idx,
            table_para_idx,
            control_idx,
            cell_idx,
            props_json,
        )
        .map_err(|e| format!("셀 속성 설정 실패: {}", e))
    })
}

fn set_hwp_cell_properties_at_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    table_para_idx: usize,
    control_idx: usize,
    row: u16,
    col: u16,
    props_json: &str,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "set-cell-properties", |core| {
        let cell_idx = core
            .get_table_cell_index_native(section_idx, table_para_idx, control_idx, row, col)
            .map_err(|e| format!("셀 좌표 조회 실패: {}", e))?;
        core.set_cell_properties_native(
            section_idx,
            table_para_idx,
            control_idx,
            cell_idx,
            props_json,
        )
        .map_err(|e| format!("셀 속성 설정 실패: {}", e))
    })
}

fn get_hwp_table_properties_json_for_cli(
    data: &[u8],
    section_idx: usize,
    table_para_idx: usize,
    control_idx: usize,
) -> Result<serde_json::Value, String> {
    let core = rhwp::document_core::DocumentCore::from_bytes(data)
        .map_err(|e| format!("HWP 파싱 실패: {}", e))?;
    let details_json = core
        .get_table_properties_native(section_idx, table_para_idx, control_idx)
        .map_err(|e| format!("표 속성 조회 실패: {}", e))?;
    let mut details = parse_json_value(&details_json);
    if let Some(obj) = details.as_object_mut() {
        obj.insert("ok".to_string(), serde_json::json!(true));
    }
    Ok(details)
}

fn set_hwp_table_properties_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    table_para_idx: usize,
    control_idx: usize,
    props_json: &str,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "set-table-properties", |core| {
        core.set_table_properties_native(section_idx, table_para_idx, control_idx, props_json)
            .map_err(|e| format!("표 속성 설정 실패: {}", e))
    })
}

fn resize_hwp_table_cells_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    table_para_idx: usize,
    control_idx: usize,
    updates_json: &str,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "resize-table-cells", |core| {
        core.resize_table_cells_native(section_idx, table_para_idx, control_idx, updates_json)
            .map_err(|e| format!("표 셀 크기 조절 실패: {}", e))
    })
}

fn get_hwp_char_properties_json_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
    char_offset: usize,
) -> Result<serde_json::Value, String> {
    let core = rhwp::document_core::DocumentCore::from_bytes(data)
        .map_err(|e| format!("HWP 파싱 실패: {}", e))?;
    let details_json = core
        .get_char_properties_at_native(section_idx, para_idx, char_offset)
        .map_err(|e| format!("글자 속성 조회 실패: {}", e))?;
    let mut details = parse_json_value(&details_json);
    if let Some(obj) = details.as_object_mut() {
        obj.insert("ok".to_string(), serde_json::json!(true));
    }
    Ok(details)
}

fn set_hwp_char_format_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
    start_offset: usize,
    end_offset: usize,
    props_json: &str,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "set-char-format", |core| {
        core.apply_char_format_native(section_idx, para_idx, start_offset, end_offset, props_json)
            .map_err(|e| format!("글자 서식 설정 실패: {}", e))
    })
}

fn get_hwp_para_properties_json_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
) -> Result<serde_json::Value, String> {
    let core = rhwp::document_core::DocumentCore::from_bytes(data)
        .map_err(|e| format!("HWP 파싱 실패: {}", e))?;
    let details_json = core
        .get_para_properties_at_native(section_idx, para_idx)
        .map_err(|e| format!("문단 속성 조회 실패: {}", e))?;
    let mut details = parse_json_value(&details_json);
    if let Some(obj) = details.as_object_mut() {
        obj.insert("ok".to_string(), serde_json::json!(true));
    }
    Ok(details)
}

fn set_hwp_para_format_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
    props_json: &str,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "set-para-format", |core| {
        core.apply_para_format_native(section_idx, para_idx, props_json)
            .map_err(|e| format!("문단 서식 설정 실패: {}", e))
    })
}

fn list_hwp_styles_json_for_cli(data: &[u8]) -> Result<serde_json::Value, String> {
    let core = rhwp::document_core::DocumentCore::from_bytes(data)
        .map_err(|e| format!("HWP 파싱 실패: {}", e))?;
    let styles: Vec<serde_json::Value> = core
        .document()
        .doc_info
        .styles
        .iter()
        .enumerate()
        .map(|(id, style)| {
            let style_type = match style.style_type {
                0 => "paragraph",
                1 => "character",
                _ => "unknown",
            };
            serde_json::json!({
                "id": id,
                "localName": style.local_name,
                "englishName": style.english_name,
                "styleType": style.style_type,
                "styleTypeLabel": style_type,
                "nextStyleId": style.next_style_id,
                "langId": style.lang_id,
                "paraShapeId": style.para_shape_id,
                "charShapeId": style.char_shape_id,
            })
        })
        .collect();
    Ok(serde_json::json!({
        "ok": true,
        "count": styles.len(),
        "styles": styles,
    }))
}

fn resolve_hwp_style_id_for_cli(
    data: &[u8],
    style_id: Option<usize>,
    style_name: Option<&str>,
) -> Result<usize, String> {
    if let Some(id) = style_id {
        return Ok(id);
    }
    let name =
        style_name.ok_or_else(|| "--style-id 또는 --style-name 값이 필요합니다.".to_string())?;
    let core = rhwp::document_core::DocumentCore::from_bytes(data)
        .map_err(|e| format!("HWP 파싱 실패: {}", e))?;
    core.document()
        .doc_info
        .styles
        .iter()
        .position(|style| style.local_name == name || style.english_name == name)
        .ok_or_else(|| format!("스타일을 찾을 수 없음: {}", name))
}

fn apply_hwp_style_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
    style_id: usize,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "apply-style", |core| {
        core.apply_style_native(section_idx, para_idx, style_id)
            .map_err(|e| format!("문단 스타일 적용 실패: {}", e))?;
        Ok(serde_json::json!({"ok": true, "styleId": style_id}).to_string())
    })
}

fn apply_hwp_cell_style_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    table_para_idx: usize,
    control_idx: usize,
    cell_idx: usize,
    cell_para_idx: usize,
    style_id: usize,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "apply-cell-style", |core| {
        core.apply_cell_style_native(
            section_idx,
            table_para_idx,
            control_idx,
            cell_idx,
            cell_para_idx,
            style_id,
        )
        .map_err(|e| format!("셀 문단 스타일 적용 실패: {}", e))?;
        Ok(serde_json::json!({"ok": true, "styleId": style_id}).to_string())
    })
}

#[allow(clippy::too_many_arguments)]
fn apply_hwp_cell_style_at_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    table_para_idx: usize,
    control_idx: usize,
    row: u16,
    col: u16,
    cell_para_idx: usize,
    style_id: usize,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "apply-cell-style", |core| {
        let cell_idx = core
            .get_table_cell_index_native(section_idx, table_para_idx, control_idx, row, col)
            .map_err(|e| format!("셀 좌표 조회 실패: {}", e))?;
        core.apply_cell_style_native(
            section_idx,
            table_para_idx,
            control_idx,
            cell_idx,
            cell_para_idx,
            style_id,
        )
        .map_err(|e| format!("셀 문단 스타일 적용 실패: {}", e))?;
        Ok(serde_json::json!({
            "ok": true,
            "styleId": style_id,
            "row": row,
            "col": col,
            "cellIndex": cell_idx,
        })
        .to_string())
    })
}

fn get_hwp_cell_char_properties_json_for_cli(
    data: &[u8],
    section_idx: usize,
    table_para_idx: usize,
    control_idx: usize,
    cell_idx: usize,
    cell_para_idx: usize,
    char_offset: usize,
) -> Result<serde_json::Value, String> {
    let core = rhwp::document_core::DocumentCore::from_bytes(data)
        .map_err(|e| format!("HWP 파싱 실패: {}", e))?;
    let details_json = core
        .get_cell_char_properties_at_native(
            section_idx,
            table_para_idx,
            control_idx,
            cell_idx,
            cell_para_idx,
            char_offset,
        )
        .map_err(|e| format!("셀 글자 속성 조회 실패: {}", e))?;
    let mut details = parse_json_value(&details_json);
    if let Some(obj) = details.as_object_mut() {
        obj.insert("ok".to_string(), serde_json::json!(true));
    }
    Ok(details)
}

#[allow(clippy::too_many_arguments)]
fn get_hwp_cell_char_properties_at_json_for_cli(
    data: &[u8],
    section_idx: usize,
    table_para_idx: usize,
    control_idx: usize,
    row: u16,
    col: u16,
    cell_para_idx: usize,
    char_offset: usize,
) -> Result<serde_json::Value, String> {
    let core = rhwp::document_core::DocumentCore::from_bytes(data)
        .map_err(|e| format!("HWP 파싱 실패: {}", e))?;
    let cell_idx = core
        .get_table_cell_index_native(section_idx, table_para_idx, control_idx, row, col)
        .map_err(|e| format!("셀 좌표 조회 실패: {}", e))?;
    let details_json = core
        .get_cell_char_properties_at_native(
            section_idx,
            table_para_idx,
            control_idx,
            cell_idx,
            cell_para_idx,
            char_offset,
        )
        .map_err(|e| format!("셀 글자 속성 조회 실패: {}", e))?;
    let mut details = parse_json_value(&details_json);
    if let Some(obj) = details.as_object_mut() {
        obj.insert("ok".to_string(), serde_json::json!(true));
        obj.insert("row".to_string(), serde_json::json!(row));
        obj.insert("col".to_string(), serde_json::json!(col));
        obj.insert("cellIndex".to_string(), serde_json::json!(cell_idx));
    }
    Ok(details)
}

fn set_hwp_cell_char_format_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    table_para_idx: usize,
    control_idx: usize,
    cell_idx: usize,
    cell_para_idx: usize,
    start_offset: usize,
    end_offset: usize,
    props_json: &str,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "set-cell-char-format", |core| {
        core.apply_char_format_in_cell_native(
            section_idx,
            table_para_idx,
            control_idx,
            cell_idx,
            cell_para_idx,
            start_offset,
            end_offset,
            props_json,
        )
        .map_err(|e| format!("셀 글자 서식 설정 실패: {}", e))
    })
}

#[allow(clippy::too_many_arguments)]
fn set_hwp_cell_char_format_at_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    table_para_idx: usize,
    control_idx: usize,
    row: u16,
    col: u16,
    cell_para_idx: usize,
    start_offset: usize,
    end_offset: usize,
    props_json: &str,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "set-cell-char-format", |core| {
        let cell_idx = core
            .get_table_cell_index_native(section_idx, table_para_idx, control_idx, row, col)
            .map_err(|e| format!("셀 좌표 조회 실패: {}", e))?;
        let details_json = core
            .apply_char_format_in_cell_native(
                section_idx,
                table_para_idx,
                control_idx,
                cell_idx,
                cell_para_idx,
                start_offset,
                end_offset,
                props_json,
            )
            .map_err(|e| format!("셀 글자 서식 설정 실패: {}", e))?;
        let mut details = parse_json_value(&details_json);
        if let Some(obj) = details.as_object_mut() {
            obj.insert("row".to_string(), serde_json::json!(row));
            obj.insert("col".to_string(), serde_json::json!(col));
            obj.insert("cellIndex".to_string(), serde_json::json!(cell_idx));
        }
        Ok(details.to_string())
    })
}

fn get_hwp_cell_para_properties_json_for_cli(
    data: &[u8],
    section_idx: usize,
    table_para_idx: usize,
    control_idx: usize,
    cell_idx: usize,
    cell_para_idx: usize,
) -> Result<serde_json::Value, String> {
    let core = rhwp::document_core::DocumentCore::from_bytes(data)
        .map_err(|e| format!("HWP 파싱 실패: {}", e))?;
    let details_json = core
        .get_cell_para_properties_at_native(
            section_idx,
            table_para_idx,
            control_idx,
            cell_idx,
            cell_para_idx,
        )
        .map_err(|e| format!("셀 문단 속성 조회 실패: {}", e))?;
    let mut details = parse_json_value(&details_json);
    if let Some(obj) = details.as_object_mut() {
        obj.insert("ok".to_string(), serde_json::json!(true));
    }
    Ok(details)
}

fn get_hwp_cell_para_properties_at_json_for_cli(
    data: &[u8],
    section_idx: usize,
    table_para_idx: usize,
    control_idx: usize,
    row: u16,
    col: u16,
    cell_para_idx: usize,
) -> Result<serde_json::Value, String> {
    let core = rhwp::document_core::DocumentCore::from_bytes(data)
        .map_err(|e| format!("HWP 파싱 실패: {}", e))?;
    let cell_idx = core
        .get_table_cell_index_native(section_idx, table_para_idx, control_idx, row, col)
        .map_err(|e| format!("셀 좌표 조회 실패: {}", e))?;
    let details_json = core
        .get_cell_para_properties_at_native(
            section_idx,
            table_para_idx,
            control_idx,
            cell_idx,
            cell_para_idx,
        )
        .map_err(|e| format!("셀 문단 속성 조회 실패: {}", e))?;
    let mut details = parse_json_value(&details_json);
    if let Some(obj) = details.as_object_mut() {
        obj.insert("ok".to_string(), serde_json::json!(true));
        obj.insert("row".to_string(), serde_json::json!(row));
        obj.insert("col".to_string(), serde_json::json!(col));
        obj.insert("cellIndex".to_string(), serde_json::json!(cell_idx));
    }
    Ok(details)
}

fn set_hwp_cell_para_format_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    table_para_idx: usize,
    control_idx: usize,
    cell_idx: usize,
    cell_para_idx: usize,
    props_json: &str,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "set-cell-para-format", |core| {
        core.apply_para_format_in_cell_native(
            section_idx,
            table_para_idx,
            control_idx,
            cell_idx,
            cell_para_idx,
            props_json,
        )
        .map_err(|e| format!("셀 문단 서식 설정 실패: {}", e))
    })
}

#[allow(clippy::too_many_arguments)]
fn set_hwp_cell_para_format_at_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    table_para_idx: usize,
    control_idx: usize,
    row: u16,
    col: u16,
    cell_para_idx: usize,
    props_json: &str,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "set-cell-para-format", |core| {
        let cell_idx = core
            .get_table_cell_index_native(section_idx, table_para_idx, control_idx, row, col)
            .map_err(|e| format!("셀 좌표 조회 실패: {}", e))?;
        let details_json = core
            .apply_para_format_in_cell_native(
                section_idx,
                table_para_idx,
                control_idx,
                cell_idx,
                cell_para_idx,
                props_json,
            )
            .map_err(|e| format!("셀 문단 서식 설정 실패: {}", e))?;
        let mut details = parse_json_value(&details_json);
        if let Some(obj) = details.as_object_mut() {
            obj.insert("row".to_string(), serde_json::json!(row));
            obj.insert("col".to_string(), serde_json::json!(col));
            obj.insert("cellIndex".to_string(), serde_json::json!(cell_idx));
        }
        Ok(details.to_string())
    })
}

fn get_hwp_page_def_json_for_cli(
    data: &[u8],
    section_idx: usize,
) -> Result<serde_json::Value, String> {
    let core = rhwp::document_core::DocumentCore::from_bytes(data)
        .map_err(|e| format!("HWP 파싱 실패: {}", e))?;
    let details_json = core
        .get_page_def_native(section_idx)
        .map_err(|e| format!("용지 설정 조회 실패: {}", e))?;
    let mut details = parse_json_value(&details_json);
    if let Some(obj) = details.as_object_mut() {
        obj.insert("ok".to_string(), serde_json::json!(true));
    }
    Ok(details)
}

fn set_hwp_page_def_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    props_json: &str,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "set-page-def", |core| {
        core.set_page_def_native(section_idx, props_json)
            .map_err(|e| format!("용지 설정 실패: {}", e))
    })
}

fn get_hwp_section_def_json_for_cli(
    data: &[u8],
    section_idx: usize,
) -> Result<serde_json::Value, String> {
    let core = rhwp::document_core::DocumentCore::from_bytes(data)
        .map_err(|e| format!("HWP 파싱 실패: {}", e))?;
    let details_json = core
        .get_section_def_native(section_idx)
        .map_err(|e| format!("구역 설정 조회 실패: {}", e))?;
    let mut details = parse_json_value(&details_json);
    if let Some(obj) = details.as_object_mut() {
        obj.insert("ok".to_string(), serde_json::json!(true));
    }
    Ok(details)
}

fn set_hwp_section_def_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    props_json: &str,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "set-section-def", |core| {
        core.set_section_def_native(section_idx, props_json)
            .map_err(|e| format!("구역 설정 실패: {}", e))
    })
}

fn get_hwp_page_border_fill_json_for_cli(
    data: &[u8],
    section_idx: usize,
) -> Result<serde_json::Value, String> {
    let core = rhwp::document_core::DocumentCore::from_bytes(data)
        .map_err(|e| format!("HWP 파싱 실패: {}", e))?;
    let details_json = core
        .get_page_border_fill_native(section_idx)
        .map_err(|e| format!("쪽 테두리/배경 조회 실패: {}", e))?;
    let mut details = parse_json_value(&details_json);
    if let Some(obj) = details.as_object_mut() {
        obj.insert("ok".to_string(), serde_json::json!(true));
    }
    Ok(details)
}

fn set_hwp_page_border_fill_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    props_json: &str,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "set-page-border-fill", |core| {
        core.set_page_border_fill_native(section_idx, props_json)
            .map_err(|e| format!("쪽 테두리/배경 설정 실패: {}", e))
    })
}

fn parse_cell_path_for_cli(cell_path_json: &str) -> Result<Vec<(usize, usize, usize)>, String> {
    let trimmed = cell_path_json.trim();
    if trimmed.is_empty() || trimmed == "[]" {
        return Ok(Vec::new());
    }
    let value: serde_json::Value =
        serde_json::from_str(trimmed).map_err(|e| format!("cellPath JSON 파싱 실패: {}", e))?;
    let arr = value
        .as_array()
        .ok_or_else(|| "cellPath JSON은 배열이어야 합니다.".to_string())?;
    let mut path = Vec::with_capacity(arr.len());
    for (idx, item) in arr.iter().enumerate() {
        if let Some(tuple) = item.as_array() {
            if tuple.len() != 3 {
                return Err(format!(
                    "cellPath[{}] 배열은 [ctrl, cell, cellPara]여야 합니다.",
                    idx
                ));
            }
            let ctrl = tuple[0]
                .as_u64()
                .ok_or_else(|| format!("cellPath[{}][0] 값이 정수가 아닙니다.", idx))?
                as usize;
            let cell = tuple[1]
                .as_u64()
                .ok_or_else(|| format!("cellPath[{}][1] 값이 정수가 아닙니다.", idx))?
                as usize;
            let cell_para = tuple[2]
                .as_u64()
                .ok_or_else(|| format!("cellPath[{}][2] 값이 정수가 아닙니다.", idx))?
                as usize;
            path.push((ctrl, cell, cell_para));
            continue;
        }

        let obj = item
            .as_object()
            .ok_or_else(|| format!("cellPath[{}] 값은 객체 또는 배열이어야 합니다.", idx))?;
        let read_key = |key: &str| -> Result<usize, String> {
            obj.get(key)
                .and_then(|v| v.as_u64())
                .map(|v| v as usize)
                .ok_or_else(|| format!("cellPath[{}].{} 값이 정수가 아닙니다.", idx, key))
        };
        path.push((
            read_key("controlIndex")?,
            read_key("cellIndex")?,
            read_key("cellParaIndex")?,
        ));
    }
    Ok(path)
}

fn parse_polygon_points_for_cli(points_json: &str) -> Result<Vec<rhwp::model::Point>, String> {
    let trimmed = points_json.trim();
    if trimmed.is_empty() || trimmed == "[]" {
        return Ok(Vec::new());
    }
    let value: serde_json::Value = serde_json::from_str(trimmed)
        .map_err(|e| format!("polygonPoints JSON 파싱 실패: {}", e))?;
    let arr = value
        .as_array()
        .ok_or_else(|| "polygonPoints JSON은 배열이어야 합니다.".to_string())?;
    let mut points = Vec::with_capacity(arr.len());
    for (idx, item) in arr.iter().enumerate() {
        if let Some(tuple) = item.as_array() {
            if tuple.len() != 2 {
                return Err(format!("polygonPoints[{}] 배열은 [x, y]여야 합니다.", idx));
            }
            let x = tuple[0]
                .as_i64()
                .ok_or_else(|| format!("polygonPoints[{}][0] 값이 정수가 아닙니다.", idx))?
                as i32;
            let y = tuple[1]
                .as_i64()
                .ok_or_else(|| format!("polygonPoints[{}][1] 값이 정수가 아닙니다.", idx))?
                as i32;
            points.push(rhwp::model::Point { x, y });
            continue;
        }

        let obj = item
            .as_object()
            .ok_or_else(|| format!("polygonPoints[{}] 값은 객체 또는 배열이어야 합니다.", idx))?;
        let read_key = |key: &str| -> Result<i32, String> {
            obj.get(key)
                .and_then(|v| v.as_i64())
                .map(|v| v as i32)
                .ok_or_else(|| format!("polygonPoints[{}].{} 값이 정수가 아닙니다.", idx, key))
        };
        points.push(rhwp::model::Point {
            x: read_key("x")?,
            y: read_key("y")?,
        });
    }
    Ok(points)
}

fn parse_shape_targets_for_cli(targets_json: &str) -> Result<Vec<(usize, usize)>, String> {
    let value: serde_json::Value = serde_json::from_str(targets_json.trim())
        .map_err(|e| format!("targets JSON 파싱 실패: {}", e))?;
    let arr = value
        .as_array()
        .ok_or_else(|| "targets JSON은 배열이어야 합니다.".to_string())?;
    let mut targets = Vec::with_capacity(arr.len());
    for (idx, item) in arr.iter().enumerate() {
        if let Some(tuple) = item.as_array() {
            if tuple.len() != 2 {
                return Err(format!(
                    "targets[{}] 배열은 [paraIdx, controlIdx]여야 합니다.",
                    idx
                ));
            }
            let para = tuple[0]
                .as_u64()
                .ok_or_else(|| format!("targets[{}][0] 값이 정수가 아닙니다.", idx))?
                as usize;
            let ctrl = tuple[1]
                .as_u64()
                .ok_or_else(|| format!("targets[{}][1] 값이 정수가 아닙니다.", idx))?
                as usize;
            targets.push((para, ctrl));
            continue;
        }

        let obj = item
            .as_object()
            .ok_or_else(|| format!("targets[{}] 값은 객체 또는 배열이어야 합니다.", idx))?;
        let read_one = |keys: &[&str]| -> Result<usize, String> {
            for key in keys {
                if let Some(value) = obj.get(*key).and_then(|v| v.as_u64()) {
                    return Ok(value as usize);
                }
            }
            Err(format!("targets[{}] 값에 {:?} 키가 없습니다.", idx, keys))
        };
        targets.push((
            read_one(&["paraIdx", "paragraph", "para"])?,
            read_one(&["controlIdx", "control", "ctrl"])?,
        ));
    }
    Ok(targets)
}

fn parse_header_footer_kind_for_cli(kind: &str) -> Result<bool, String> {
    match kind.trim().to_ascii_lowercase().as_str() {
        "header" | "head" | "h" | "머리말" => Ok(true),
        "footer" | "foot" | "f" | "꼬리말" => Ok(false),
        _ => Err(format!("kind는 header 또는 footer여야 합니다: {}", kind)),
    }
}

fn parse_header_footer_apply_to_for_cli(value: &str) -> Result<u8, String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "0" | "both" | "all" | "양쪽" | "양 쪽" => Ok(0),
        "1" | "even" | "짝수" | "짝수쪽" | "짝수 쪽" => Ok(1),
        "2" | "odd" | "홀수" | "홀수쪽" | "홀수 쪽" => Ok(2),
        _ => Err(format!(
            "apply-to는 both/even/odd 또는 0/1/2여야 합니다: {}",
            value
        )),
    }
}

fn parse_header_footer_field_type_for_cli(value: &str) -> Result<u8, String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "page" | "page-number" | "current-page" | "쪽번호" | "현재쪽" => Ok(1),
        "2" | "total" | "total-pages" | "page-count" | "총쪽수" => Ok(2),
        "3" | "filename" | "file-name" | "file" | "파일명" | "파일이름" => Ok(3),
        _ => Err(format!(
            "field는 page-number, total-pages, filename 또는 1/2/3이어야 합니다: {}",
            value
        )),
    }
}

fn parse_column_type_for_cli(value: &str) -> Result<u8, String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "0" | "normal" | "일반" => Ok(0),
        "1" | "distribute" | "distributed" | "배분" => Ok(1),
        "2" | "parallel" | "평행" => Ok(2),
        _ => Err(format!(
            "type은 normal/distribute/parallel 또는 0/1/2여야 합니다: {}",
            value
        )),
    }
}

fn get_hwp_header_footer_json_for_cli(
    data: &[u8],
    section_idx: usize,
    is_header: bool,
    apply_to: u8,
) -> Result<serde_json::Value, String> {
    let core = rhwp::document_core::DocumentCore::from_bytes(data)
        .map_err(|e| format!("HWP 파싱 실패: {}", e))?;
    let details_json = core
        .get_header_footer_native(section_idx, is_header, apply_to)
        .map_err(|e| format!("머리말/꼬리말 조회 실패: {}", e))?;
    let mut details = parse_json_value(&details_json);
    if let Some(obj) = details.as_object_mut() {
        obj.insert("ok".to_string(), serde_json::json!(true));
    }
    Ok(details)
}

fn list_hwp_header_footer_json_for_cli(
    data: &[u8],
    section_idx: usize,
    is_header: bool,
    apply_to: u8,
) -> Result<serde_json::Value, String> {
    let core = rhwp::document_core::DocumentCore::from_bytes(data)
        .map_err(|e| format!("HWP 파싱 실패: {}", e))?;
    let details_json = core
        .get_header_footer_list_native(section_idx, is_header, apply_to)
        .map_err(|e| format!("머리말/꼬리말 목록 조회 실패: {}", e))?;
    let mut details = parse_json_value(&details_json);
    if let Some(obj) = details.as_object_mut() {
        obj.insert("ok".to_string(), serde_json::json!(true));
    }
    Ok(details)
}

fn get_hwp_header_footer_para_info_json_for_cli(
    data: &[u8],
    section_idx: usize,
    is_header: bool,
    apply_to: u8,
    hf_para_idx: usize,
) -> Result<serde_json::Value, String> {
    let core = rhwp::document_core::DocumentCore::from_bytes(data)
        .map_err(|e| format!("HWP 파싱 실패: {}", e))?;
    let details_json = core
        .get_header_footer_para_info_native(section_idx, is_header, apply_to, hf_para_idx)
        .map_err(|e| format!("머리말/꼬리말 문단 정보 조회 실패: {}", e))?;
    let mut details = parse_json_value(&details_json);
    if let Some(obj) = details.as_object_mut() {
        obj.insert("ok".to_string(), serde_json::json!(true));
    }
    Ok(details)
}

fn get_hwp_header_footer_para_properties_json_for_cli(
    data: &[u8],
    section_idx: usize,
    is_header: bool,
    apply_to: u8,
    hf_para_idx: usize,
) -> Result<serde_json::Value, String> {
    let core = rhwp::document_core::DocumentCore::from_bytes(data)
        .map_err(|e| format!("HWP 파싱 실패: {}", e))?;
    let details_json = core
        .get_para_properties_in_hf_native(section_idx, is_header, apply_to, hf_para_idx)
        .map_err(|e| format!("머리말/꼬리말 문단 서식 조회 실패: {}", e))?;
    let mut details = parse_json_value(&details_json);
    if let Some(obj) = details.as_object_mut() {
        obj.insert("ok".to_string(), serde_json::json!(true));
    }
    Ok(details)
}

fn create_hwp_header_footer_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    is_header: bool,
    apply_to: u8,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "create-header-footer", |core| {
        core.create_header_footer_native(section_idx, is_header, apply_to)
            .map_err(|e| format!("머리말/꼬리말 생성 실패: {}", e))
    })
}

fn delete_hwp_header_footer_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    is_header: bool,
    apply_to: u8,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "delete-header-footer", |core| {
        core.delete_header_footer_native(section_idx, is_header, apply_to)
            .map_err(|e| format!("머리말/꼬리말 삭제 실패: {}", e))
    })
}

fn insert_hwp_header_footer_text_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    is_header: bool,
    apply_to: u8,
    hf_para_idx: usize,
    char_offset: usize,
    text: &str,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "insert-header-footer-text", |core| {
        core.insert_text_in_header_footer_native(
            section_idx,
            is_header,
            apply_to,
            hf_para_idx,
            char_offset,
            text,
        )
        .map_err(|e| format!("머리말/꼬리말 텍스트 삽입 실패: {}", e))
    })
}

fn delete_hwp_header_footer_text_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    is_header: bool,
    apply_to: u8,
    hf_para_idx: usize,
    char_offset: usize,
    count: usize,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "delete-header-footer-text", |core| {
        core.delete_text_in_header_footer_native(
            section_idx,
            is_header,
            apply_to,
            hf_para_idx,
            char_offset,
            count,
        )
        .map_err(|e| format!("머리말/꼬리말 텍스트 삭제 실패: {}", e))
    })
}

fn split_hwp_header_footer_paragraph_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    is_header: bool,
    apply_to: u8,
    hf_para_idx: usize,
    char_offset: usize,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "split-header-footer-paragraph", |core| {
        core.split_paragraph_in_header_footer_native(
            section_idx,
            is_header,
            apply_to,
            hf_para_idx,
            char_offset,
        )
        .map_err(|e| format!("머리말/꼬리말 문단 분할 실패: {}", e))
    })
}

fn merge_hwp_header_footer_paragraph_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    is_header: bool,
    apply_to: u8,
    hf_para_idx: usize,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "merge-header-footer-paragraph", |core| {
        core.merge_paragraph_in_header_footer_native(section_idx, is_header, apply_to, hf_para_idx)
            .map_err(|e| format!("머리말/꼬리말 문단 병합 실패: {}", e))
    })
}

fn set_hwp_header_footer_para_format_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    is_header: bool,
    apply_to: u8,
    hf_para_idx: usize,
    props_json: &str,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "set-header-footer-para-format", |core| {
        core.apply_para_format_in_hf_native(
            section_idx,
            is_header,
            apply_to,
            hf_para_idx,
            props_json,
        )
        .map_err(|e| format!("머리말/꼬리말 문단 서식 설정 실패: {}", e))
    })
}

fn insert_hwp_header_footer_field_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    is_header: bool,
    apply_to: u8,
    hf_para_idx: usize,
    char_offset: usize,
    field_type: u8,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "insert-header-footer-field", |core| {
        core.insert_field_in_hf_native(
            section_idx,
            is_header,
            apply_to,
            hf_para_idx,
            char_offset,
            field_type,
        )
        .map_err(|e| format!("머리말/꼬리말 필드 삽입 실패: {}", e))
    })
}

fn apply_hwp_header_footer_template_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    is_header: bool,
    apply_to: u8,
    template_id: u8,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "apply-header-footer-template", |core| {
        core.apply_hf_template_native(section_idx, is_header, apply_to, template_id)
            .map_err(|e| format!("머리말/꼬리말 템플릿 적용 실패: {}", e))
    })
}

fn header_footer_apply_from_u8_for_cli(
    apply_to: u8,
) -> rhwp::model::header_footer::HeaderFooterApply {
    match apply_to {
        1 => rhwp::model::header_footer::HeaderFooterApply::Even,
        2 => rhwp::model::header_footer::HeaderFooterApply::Odd,
        _ => rhwp::model::header_footer::HeaderFooterApply::Both,
    }
}

fn header_footer_apply_to_u8_for_cli(
    apply_to: rhwp::model::header_footer::HeaderFooterApply,
) -> u8 {
    match apply_to {
        rhwp::model::header_footer::HeaderFooterApply::Both => 0,
        rhwp::model::header_footer::HeaderFooterApply::Even => 1,
        rhwp::model::header_footer::HeaderFooterApply::Odd => 2,
    }
}

fn header_footer_apply_label_for_cli(
    apply_to: rhwp::model::header_footer::HeaderFooterApply,
) -> &'static str {
    match apply_to {
        rhwp::model::header_footer::HeaderFooterApply::Both => "양 쪽",
        rhwp::model::header_footer::HeaderFooterApply::Even => "짝수 쪽",
        rhwp::model::header_footer::HeaderFooterApply::Odd => "홀수 쪽",
    }
}

fn prepare_master_pages_for_model_serialization(section: &mut rhwp::model::document::Section) {
    for master_page in &mut section.section_def.master_pages {
        master_page.raw_list_header.clear();
    }
    section
        .section_def
        .extra_child_records
        .retain(|raw| raw.tag_id != rhwp::parser::tags::HWPTAG_LIST_HEADER);
    section.raw_stream = None;
}

fn sync_section_def_control_for_cli(section: &mut rhwp::model::document::Section) {
    let updated_section_def = section.section_def.clone();
    for para in &mut section.paragraphs {
        for ctrl in &mut para.controls {
            if let rhwp::model::control::Control::SectionDef(section_def) = ctrl {
                **section_def = updated_section_def.clone();
                return;
            }
        }
    }
}

fn materialize_master_page_section_contract(section: &mut rhwp::model::document::Section) {
    const MASTER_PAGE_FLAGS_MASK: u32 = 0xe000_0000;
    let count = section.section_def.master_pages.len();
    section.section_def.flags &= !MASTER_PAGE_FLAGS_MASK;
    if count == 0 {
        section.section_def.raw_ctrl_extra.clear();
        sync_section_def_control_for_cli(section);
        section.raw_stream = None;
        return;
    }

    section.section_def.flags |= if count == 1 { 0x2000_0000 } else { 0xC000_0000 };
    let mut extra = vec![0; 19];
    extra[0..2].copy_from_slice(&0u16.to_le_bytes());
    if count >= 3 {
        extra[2..4].copy_from_slice(&1u16.to_le_bytes());
    }
    section.section_def.raw_ctrl_extra = extra;
    sync_section_def_control_for_cli(section);
    section.raw_stream = None;
}

fn master_page_text_for_cli(master_page: &rhwp::model::header_footer::MasterPage) -> String {
    master_page
        .paragraphs
        .iter()
        .map(|p| p.text.clone())
        .collect::<Vec<_>>()
        .join("\n")
}

fn list_hwp_master_pages_json_for_cli(
    data: &[u8],
    section_idx: usize,
) -> Result<serde_json::Value, String> {
    let core = rhwp::document_core::DocumentCore::from_bytes(data)
        .map_err(|e| format!("HWP 파싱 실패: {}", e))?;
    let section = core
        .document()
        .sections
        .get(section_idx)
        .ok_or_else(|| format!("구역 인덱스 {} 범위 초과", section_idx))?;
    let items = section
        .section_def
        .master_pages
        .iter()
        .enumerate()
        .map(|(idx, master_page)| {
            serde_json::json!({
                "index": idx,
                "applyTo": header_footer_apply_to_u8_for_cli(master_page.apply_to),
                "label": header_footer_apply_label_for_cli(master_page.apply_to),
                "isExtension": master_page.is_extension,
                "overlap": master_page.overlap,
                "replaceBase": master_page.replace_base,
                "paragraphCount": master_page.paragraphs.len(),
                "textWidth": master_page.text_width,
                "textHeight": master_page.text_height,
                "text": master_page_text_for_cli(master_page),
            })
        })
        .collect::<Vec<_>>();
    Ok(serde_json::json!({
        "ok": true,
        "section": section_idx,
        "count": items.len(),
        "items": items,
    }))
}

fn create_hwp_master_page_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    apply_to: u8,
    is_extension: bool,
    overlap: bool,
    text: &str,
) -> Result<HwpEditCliResult, String> {
    let mut core = rhwp::document_core::DocumentCore::from_bytes(data)
        .map_err(|e| format!("HWP 파싱 실패: {}", e))?;
    core.convert_to_editable_native()
        .map_err(|e| format!("편집 가능 변환 실패: {}", e))?;
    let section = core
        .document_mut()
        .sections
        .get_mut(section_idx)
        .ok_or_else(|| format!("구역 인덱스 {} 범위 초과", section_idx))?;
    prepare_master_pages_for_model_serialization(section);

    let page_def = &section.section_def.page_def;
    let text_width = page_def
        .width
        .saturating_sub(page_def.margin_left.saturating_add(page_def.margin_right));
    let text_height = page_def
        .height
        .saturating_sub(page_def.margin_top.saturating_add(page_def.margin_bottom));
    let mut paragraph = rhwp::model::paragraph::Paragraph::new_empty();
    paragraph.insert_text_at(0, text);
    let master_page = rhwp::model::header_footer::MasterPage {
        apply_to: header_footer_apply_from_u8_for_cli(apply_to),
        is_extension,
        overlap,
        paragraphs: vec![paragraph],
        text_width,
        text_height,
        text_ref: 0,
        num_ref: 0,
        ..Default::default()
    };
    section.section_def.master_pages.push(master_page);
    materialize_master_page_section_contract(section);
    let master_page_idx = section.section_def.master_pages.len() - 1;
    let (bytes, page_count_before, page_count_after) = serialize_hwp_verified_for_cli(&mut core)?;
    Ok(HwpEditCliResult {
        bytes,
        details: serde_json::json!({
            "ok": true,
            "operation": "create-master-page",
            "section": section_idx,
            "masterPageIndex": master_page_idx,
            "applyTo": apply_to,
            "isExtension": is_extension,
            "overlap": overlap,
            "text": text,
        }),
        page_count_before,
        page_count_after,
    })
}

fn set_hwp_master_page_text_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    master_page_idx: usize,
    para_idx: usize,
    text: &str,
) -> Result<HwpEditCliResult, String> {
    let mut core = rhwp::document_core::DocumentCore::from_bytes(data)
        .map_err(|e| format!("HWP 파싱 실패: {}", e))?;
    core.convert_to_editable_native()
        .map_err(|e| format!("편집 가능 변환 실패: {}", e))?;
    let section = core
        .document_mut()
        .sections
        .get_mut(section_idx)
        .ok_or_else(|| format!("구역 인덱스 {} 범위 초과", section_idx))?;
    prepare_master_pages_for_model_serialization(section);
    let master_page = section
        .section_def
        .master_pages
        .get_mut(master_page_idx)
        .ok_or_else(|| format!("바탕쪽 인덱스 {} 범위 초과", master_page_idx))?;
    master_page.raw_list_header.clear();
    let paragraph = master_page
        .paragraphs
        .get_mut(para_idx)
        .ok_or_else(|| format!("바탕쪽 문단 인덱스 {} 범위 초과", para_idx))?;
    let len = paragraph.text.chars().count();
    paragraph.delete_text_at(0, len);
    paragraph.insert_text_at(0, text);
    materialize_master_page_section_contract(section);
    let (bytes, page_count_before, page_count_after) = serialize_hwp_verified_for_cli(&mut core)?;
    Ok(HwpEditCliResult {
        bytes,
        details: serde_json::json!({
            "ok": true,
            "operation": "set-master-page-text",
            "section": section_idx,
            "masterPageIndex": master_page_idx,
            "paragraph": para_idx,
            "text": text,
        }),
        page_count_before,
        page_count_after,
    })
}

fn delete_hwp_master_page_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    master_page_idx: usize,
) -> Result<HwpEditCliResult, String> {
    let mut core = rhwp::document_core::DocumentCore::from_bytes(data)
        .map_err(|e| format!("HWP 파싱 실패: {}", e))?;
    core.convert_to_editable_native()
        .map_err(|e| format!("편집 가능 변환 실패: {}", e))?;
    let section = core
        .document_mut()
        .sections
        .get_mut(section_idx)
        .ok_or_else(|| format!("구역 인덱스 {} 범위 초과", section_idx))?;
    prepare_master_pages_for_model_serialization(section);
    if master_page_idx >= section.section_def.master_pages.len() {
        return Err(format!("바탕쪽 인덱스 {} 범위 초과", master_page_idx));
    }
    section.section_def.master_pages.remove(master_page_idx);
    materialize_master_page_section_contract(section);
    let (bytes, page_count_before, page_count_after) = serialize_hwp_verified_for_cli(&mut core)?;
    Ok(HwpEditCliResult {
        bytes,
        details: serde_json::json!({
            "ok": true,
            "operation": "delete-master-page",
            "section": section_idx,
            "masterPageIndex": master_page_idx,
        }),
        page_count_before,
        page_count_after,
    })
}

#[allow(clippy::too_many_arguments)]
fn insert_hwp_picture_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
    char_offset: usize,
    cell_path_json: &str,
    image_data: &[u8],
    width: u32,
    height: u32,
    natural_width_px: u32,
    natural_height_px: u32,
    extension: &str,
    description: &str,
    paper_offset_x_hu: Option<i32>,
    paper_offset_y_hu: Option<i32>,
) -> Result<HwpTableCliResult, String> {
    let cell_path = parse_cell_path_for_cli(cell_path_json)?;
    let mut core = rhwp::document_core::DocumentCore::from_bytes(data)
        .map_err(|e| format!("HWP 파싱 실패: {}", e))?;
    core.convert_to_editable_native()
        .map_err(|e| format!("편집 가능 변환 실패: {}", e))?;
    let details_json = core
        .insert_picture_native(
            section_idx,
            para_idx,
            char_offset,
            &cell_path,
            image_data,
            width,
            height,
            natural_width_px,
            natural_height_px,
            extension,
            description,
            paper_offset_x_hu,
            paper_offset_y_hu,
        )
        .map_err(|e| format!("그림 삽입 실패: {}", e))?;
    let mut details = parse_json_value(&details_json);
    if let Some(obj) = details.as_object_mut() {
        obj.insert("operation".to_string(), serde_json::json!("insert-picture"));
    }
    let para_idx = details
        .get("paraIdx")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| format!("그림 삽입 결과에 paraIdx가 없습니다: {}", details_json))?
        as usize;
    let control_idx = details
        .get("controlIdx")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| format!("그림 삽입 결과에 controlIdx가 없습니다: {}", details_json))?
        as usize;
    let (bytes, page_count_before, page_count_after) = serialize_hwp_verified_for_cli(&mut core)?;
    Ok(HwpTableCliResult {
        bytes,
        para_idx,
        control_idx,
        details,
        page_count_before,
        page_count_after,
    })
}

fn get_hwp_picture_properties_json_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
    control_idx: usize,
) -> Result<serde_json::Value, String> {
    let core = rhwp::document_core::DocumentCore::from_bytes(data)
        .map_err(|e| format!("HWP 파싱 실패: {}", e))?;
    let details_json = core
        .get_picture_properties_native(section_idx, para_idx, control_idx)
        .map_err(|e| format!("그림 속성 조회 실패: {}", e))?;
    let mut details = parse_json_value(&details_json);
    if let Some(obj) = details.as_object_mut() {
        obj.insert("ok".to_string(), serde_json::json!(true));
    }
    Ok(details)
}

fn get_hwp_cell_picture_properties_json_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
    cell_path_json: &str,
    control_idx: usize,
) -> Result<serde_json::Value, String> {
    let cell_path = parse_cell_path_for_cli(cell_path_json)?;
    if cell_path.is_empty() {
        return Err("cell-path는 비어 있을 수 없습니다.".to_string());
    }
    let core = rhwp::document_core::DocumentCore::from_bytes(data)
        .map_err(|e| format!("HWP 파싱 실패: {}", e))?;
    let details_json = match core.get_cell_picture_properties_by_path_native(
        section_idx,
        para_idx,
        cell_path_json,
        control_idx,
    ) {
        Ok(json) => json,
        Err(_) => core
            .get_picture_properties_native(section_idx, para_idx, control_idx)
            .map_err(|e| format!("셀 그림 속성 조회 실패: {}", e))?,
    };
    let mut details = parse_json_value(&details_json);
    if let Some(obj) = details.as_object_mut() {
        obj.insert("ok".to_string(), serde_json::json!(true));
        obj.insert("container".to_string(), serde_json::json!("cell"));
    }
    Ok(details)
}

fn cell_path_from_row_col_for_cli(
    data: &[u8],
    section_idx: usize,
    table_para_idx: usize,
    table_control_idx: usize,
    row: u16,
    col: u16,
    cell_para_idx: usize,
) -> Result<(String, usize), String> {
    let core = rhwp::document_core::DocumentCore::from_bytes(data)
        .map_err(|e| format!("HWP 파싱 실패: {}", e))?;
    let cell_idx = core
        .get_table_cell_index_native(section_idx, table_para_idx, table_control_idx, row, col)
        .map_err(|e| format!("셀 좌표 조회 실패: {}", e))?;
    let cell_path = serde_json::json!([
        {
            "controlIndex": table_control_idx,
            "cellIndex": cell_idx,
            "cellParaIndex": cell_para_idx,
        }
    ])
    .to_string();
    Ok((cell_path, cell_idx))
}

#[allow(clippy::too_many_arguments)]
fn insert_hwp_cell_picture_at_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
    table_control_idx: usize,
    row: u16,
    col: u16,
    cell_para_idx: usize,
    char_offset: usize,
    image_data: &[u8],
    width: u32,
    height: u32,
    natural_width_px: u32,
    natural_height_px: u32,
    extension: &str,
    description: &str,
    paper_offset_x_hu: Option<i32>,
    paper_offset_y_hu: Option<i32>,
) -> Result<HwpTableCliResult, String> {
    let (cell_path, cell_idx) = cell_path_from_row_col_for_cli(
        data,
        section_idx,
        para_idx,
        table_control_idx,
        row,
        col,
        cell_para_idx,
    )?;
    let mut result = insert_hwp_picture_bytes_for_cli(
        data,
        section_idx,
        para_idx,
        char_offset,
        &cell_path,
        image_data,
        width,
        height,
        natural_width_px,
        natural_height_px,
        extension,
        description,
        paper_offset_x_hu,
        paper_offset_y_hu,
    )?;
    if let Some(obj) = result.details.as_object_mut() {
        obj.insert("container".to_string(), serde_json::json!("cell"));
        obj.insert("row".to_string(), serde_json::json!(row));
        obj.insert("col".to_string(), serde_json::json!(col));
        obj.insert("cellIndex".to_string(), serde_json::json!(cell_idx));
        obj.insert(
            "cellParaIndex".to_string(),
            serde_json::json!(cell_para_idx),
        );
        obj.insert(
            "tableControl".to_string(),
            serde_json::json!(table_control_idx),
        );
    }
    Ok(result)
}

#[allow(clippy::too_many_arguments)]
fn insert_hwp_cell_picture_inline_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
    table_control_idx: usize,
    row: u16,
    col: u16,
    cell_para_idx: usize,
    char_offset: usize,
    image_data: &[u8],
    width: u32,
    height: u32,
    natural_width_px: u32,
    natural_height_px: u32,
    extension: &str,
    description: &str,
) -> Result<HwpTableCliResult, String> {
    let (cell_path_json, cell_idx) = cell_path_from_row_col_for_cli(
        data,
        section_idx,
        para_idx,
        table_control_idx,
        row,
        col,
        cell_para_idx,
    )?;
    let cell_path = parse_cell_path_for_cli(&cell_path_json)?;
    let mut core = rhwp::document_core::DocumentCore::from_bytes(data)
        .map_err(|e| format!("HWP 파싱 실패: {}", e))?;
    core.convert_to_editable_native()
        .map_err(|e| format!("편집 가능 변환 실패: {}", e))?;
    let details_json = core
        .insert_cell_picture_inline_native(
            section_idx,
            para_idx,
            &cell_path,
            char_offset,
            image_data,
            width,
            height,
            natural_width_px,
            natural_height_px,
            extension,
            description,
        )
        .map_err(|e| format!("셀 인라인 그림 삽입 실패: {}", e))?;
    let mut details = parse_json_value(&details_json);
    if let Some(obj) = details.as_object_mut() {
        obj.insert("operation".to_string(), serde_json::json!("insert-picture"));
        obj.insert("inline".to_string(), serde_json::json!(true));
        obj.insert("row".to_string(), serde_json::json!(row));
        obj.insert("col".to_string(), serde_json::json!(col));
        obj.insert("cellIndex".to_string(), serde_json::json!(cell_idx));
        obj.insert(
            "cellParaIndex".to_string(),
            serde_json::json!(cell_para_idx),
        );
        obj.insert(
            "tableControl".to_string(),
            serde_json::json!(table_control_idx),
        );
    }
    let para_idx_out = details
        .get("paraIdx")
        .and_then(|v| v.as_u64())
        .unwrap_or(para_idx as u64) as usize;
    let control_idx = details
        .get("controlIdx")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;
    let (bytes, page_count_before, page_count_after) = serialize_hwp_verified_for_cli(&mut core)?;
    Ok(HwpTableCliResult {
        bytes,
        para_idx: para_idx_out,
        control_idx,
        details,
        page_count_before,
        page_count_after,
    })
}

#[allow(clippy::too_many_arguments)]
fn get_hwp_cell_picture_properties_at_json_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
    table_control_idx: usize,
    row: u16,
    col: u16,
    cell_para_idx: usize,
    picture_control_idx: usize,
) -> Result<serde_json::Value, String> {
    let (cell_path, cell_idx) = cell_path_from_row_col_for_cli(
        data,
        section_idx,
        para_idx,
        table_control_idx,
        row,
        col,
        cell_para_idx,
    )?;
    let mut details = get_hwp_cell_picture_properties_json_for_cli(
        data,
        section_idx,
        para_idx,
        &cell_path,
        picture_control_idx,
    )?;
    if let Some(obj) = details.as_object_mut() {
        obj.insert("row".to_string(), serde_json::json!(row));
        obj.insert("col".to_string(), serde_json::json!(col));
        obj.insert("cellIndex".to_string(), serde_json::json!(cell_idx));
        obj.insert(
            "cellParaIndex".to_string(),
            serde_json::json!(cell_para_idx),
        );
        obj.insert(
            "tableControl".to_string(),
            serde_json::json!(table_control_idx),
        );
    }
    Ok(details)
}

fn set_hwp_picture_properties_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
    control_idx: usize,
    props_json: &str,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "set-picture-properties", |core| {
        core.set_picture_properties_native(section_idx, para_idx, control_idx, props_json)
            .map_err(|e| format!("그림 속성 설정 실패: {}", e))
    })
}

fn set_hwp_cell_picture_properties_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
    cell_path_json: &str,
    control_idx: usize,
    props_json: &str,
) -> Result<HwpEditCliResult, String> {
    let cell_path = parse_cell_path_for_cli(cell_path_json)?;
    if cell_path.is_empty() {
        return Err("cell-path는 비어 있을 수 없습니다.".to_string());
    }
    let mut result =
        edit_hwp_table_structure_bytes_for_cli(data, "set-picture-properties", |core| match core
            .set_cell_picture_properties_by_path_native(
                section_idx,
                para_idx,
                cell_path_json,
                control_idx,
                props_json,
            ) {
            Ok(json) => Ok(json),
            Err(_) => core
                .set_picture_properties_native(section_idx, para_idx, control_idx, props_json)
                .map_err(|e| format!("셀 그림 속성 설정 실패: {}", e)),
        })?;
    if let Some(obj) = result.details.as_object_mut() {
        obj.insert("container".to_string(), serde_json::json!("cell"));
    }
    Ok(result)
}

#[allow(clippy::too_many_arguments)]
fn set_hwp_cell_picture_properties_at_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
    table_control_idx: usize,
    row: u16,
    col: u16,
    cell_para_idx: usize,
    picture_control_idx: usize,
    props_json: &str,
) -> Result<HwpEditCliResult, String> {
    let (cell_path, cell_idx) = cell_path_from_row_col_for_cli(
        data,
        section_idx,
        para_idx,
        table_control_idx,
        row,
        col,
        cell_para_idx,
    )?;
    let mut result = set_hwp_cell_picture_properties_bytes_for_cli(
        data,
        section_idx,
        para_idx,
        &cell_path,
        picture_control_idx,
        props_json,
    )?;
    if let Some(obj) = result.details.as_object_mut() {
        obj.insert("row".to_string(), serde_json::json!(row));
        obj.insert("col".to_string(), serde_json::json!(col));
        obj.insert("cellIndex".to_string(), serde_json::json!(cell_idx));
        obj.insert(
            "cellParaIndex".to_string(),
            serde_json::json!(cell_para_idx),
        );
        obj.insert(
            "tableControl".to_string(),
            serde_json::json!(table_control_idx),
        );
    }
    Ok(result)
}

fn delete_hwp_picture_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
    control_idx: usize,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "delete-picture", |core| {
        core.delete_picture_control_native(section_idx, para_idx, control_idx)
            .map_err(|e| format!("그림 삭제 실패: {}", e))
    })
}

fn delete_hwp_cell_picture_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
    cell_path_json: &str,
    control_idx: usize,
) -> Result<HwpEditCliResult, String> {
    let cell_path = parse_cell_path_for_cli(cell_path_json)?;
    if cell_path.is_empty() {
        return Err("cell-path는 비어 있을 수 없습니다.".to_string());
    }
    let mut result = edit_hwp_table_structure_bytes_for_cli(data, "delete-picture", |core| {
        match core.delete_cell_picture_control_by_path_native(
            section_idx,
            para_idx,
            cell_path_json,
            control_idx,
        ) {
            Ok(json) => Ok(json),
            Err(_) => core
                .delete_picture_control_native(section_idx, para_idx, control_idx)
                .map_err(|e| format!("셀 그림 삭제 실패: {}", e)),
        }
    })?;
    if let Some(obj) = result.details.as_object_mut() {
        obj.insert("container".to_string(), serde_json::json!("cell"));
    }
    Ok(result)
}

#[allow(clippy::too_many_arguments)]
fn delete_hwp_cell_picture_at_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
    table_control_idx: usize,
    row: u16,
    col: u16,
    cell_para_idx: usize,
    picture_control_idx: usize,
) -> Result<HwpEditCliResult, String> {
    let (cell_path, cell_idx) = cell_path_from_row_col_for_cli(
        data,
        section_idx,
        para_idx,
        table_control_idx,
        row,
        col,
        cell_para_idx,
    )?;
    let mut result = delete_hwp_cell_picture_bytes_for_cli(
        data,
        section_idx,
        para_idx,
        &cell_path,
        picture_control_idx,
    )?;
    if let Some(obj) = result.details.as_object_mut() {
        obj.insert("row".to_string(), serde_json::json!(row));
        obj.insert("col".to_string(), serde_json::json!(col));
        obj.insert("cellIndex".to_string(), serde_json::json!(cell_idx));
        obj.insert(
            "cellParaIndex".to_string(),
            serde_json::json!(cell_para_idx),
        );
        obj.insert(
            "tableControl".to_string(),
            serde_json::json!(table_control_idx),
        );
    }
    Ok(result)
}

#[allow(clippy::too_many_arguments)]
fn create_hwp_shape_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
    char_offset: usize,
    width: u32,
    height: u32,
    horz_offset: u32,
    vert_offset: u32,
    treat_as_char: bool,
    text_wrap: &str,
    shape_type: &str,
    line_flip_x: bool,
    line_flip_y: bool,
    polygon_points_json: &str,
) -> Result<HwpTableCliResult, String> {
    let polygon_points = parse_polygon_points_for_cli(polygon_points_json)?;
    let mut core = rhwp::document_core::DocumentCore::from_bytes(data)
        .map_err(|e| format!("HWP 파싱 실패: {}", e))?;
    core.convert_to_editable_native()
        .map_err(|e| format!("편집 가능 변환 실패: {}", e))?;
    let details_json = core
        .create_shape_control_native(
            section_idx,
            para_idx,
            char_offset,
            width,
            height,
            horz_offset,
            vert_offset,
            treat_as_char,
            text_wrap,
            shape_type,
            line_flip_x,
            line_flip_y,
            &polygon_points,
        )
        .map_err(|e| format!("도형 생성 실패: {}", e))?;
    let mut details = parse_json_value(&details_json);
    if let Some(obj) = details.as_object_mut() {
        obj.insert("operation".to_string(), serde_json::json!("create-shape"));
    }
    let para_idx = details
        .get("paraIdx")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| format!("도형 생성 결과에 paraIdx가 없습니다: {}", details_json))?
        as usize;
    let control_idx = details
        .get("controlIdx")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| format!("도형 생성 결과에 controlIdx가 없습니다: {}", details_json))?
        as usize;
    let (bytes, page_count_before, page_count_after) = serialize_hwp_verified_for_cli(&mut core)?;
    Ok(HwpTableCliResult {
        bytes,
        para_idx,
        control_idx,
        details,
        page_count_before,
        page_count_after,
    })
}

fn resolve_cell_paragraph_mut_for_cli<'a>(
    section: &'a mut rhwp::model::document::Section,
    parent_para_idx: usize,
    path: &[(usize, usize, usize)],
) -> Result<&'a mut rhwp::model::paragraph::Paragraph, String> {
    let mut current_para = section
        .paragraphs
        .get_mut(parent_para_idx)
        .ok_or_else(|| format!("문단 인덱스 {} 범위 초과", parent_para_idx))?;
    for (path_index, &(control_index, cell_index, cell_para_index)) in path.iter().enumerate() {
        let control = current_para
            .controls
            .get_mut(control_index)
            .ok_or_else(|| {
                format!(
                    "경로[{}]: controls[{}] 범위 초과",
                    path_index, control_index
                )
            })?;
        current_para = match control {
            rhwp::model::control::Control::Table(table) => {
                table.dirty = true;
                let cell = table.cells.get_mut(cell_index).ok_or_else(|| {
                    format!("경로[{}]: cells[{}] 범위 초과", path_index, cell_index)
                })?;
                cell.paragraphs.get_mut(cell_para_index).ok_or_else(|| {
                    format!(
                        "경로[{}]: paragraphs[{}] 범위 초과",
                        path_index, cell_para_index
                    )
                })?
            }
            rhwp::model::control::Control::Shape(shape) => {
                if cell_index != 0 {
                    return Err(format!(
                        "경로[{}]: 글상자의 cellIndex는 0이어야 합니다 ({})",
                        path_index, cell_index
                    ));
                }
                let textbox = shape
                    .drawing_mut()
                    .and_then(|drawing| drawing.text_box.as_mut())
                    .ok_or_else(|| {
                        format!(
                            "경로[{}]: controls[{}]가 텍스트 글상자가 아닙니다",
                            path_index, control_index
                        )
                    })?;
                textbox.paragraphs.get_mut(cell_para_index).ok_or_else(|| {
                    format!(
                        "경로[{}]: 글상자 문단 {} 범위 초과",
                        path_index, cell_para_index
                    )
                })?
            }
            _ => {
                return Err(format!(
                    "경로[{}]: controls[{}]가 표/글상자가 아닙니다",
                    path_index, control_index
                ));
            }
        };
    }
    Ok(current_para)
}

fn insert_cell_shape_control_for_cli(
    paragraph: &mut rhwp::model::paragraph::Paragraph,
    char_offset: usize,
    shape: Box<rhwp::model::shape::ShapeObject>,
) -> usize {
    let positions = paragraph.control_text_positions();
    let mut insert_index = paragraph.controls.len();
    for (control_index, position) in positions.iter().enumerate() {
        if *position > char_offset {
            insert_index = control_index;
            break;
        }
    }

    paragraph
        .controls
        .insert(insert_index, rhwp::model::control::Control::Shape(shape));
    let ctrl_data_index = insert_index.min(paragraph.ctrl_data_records.len());
    paragraph.ctrl_data_records.insert(ctrl_data_index, None);

    if !paragraph.char_offsets.is_empty() {
        let raw_offset = if insert_index > 0 && insert_index <= paragraph.char_offsets.len() {
            paragraph.char_offsets[insert_index - 1] + 8
        } else if !paragraph.char_offsets.is_empty() {
            paragraph.char_offsets[0].saturating_sub(8)
        } else {
            (char_offset * 2) as u32
        };
        let offset_index = insert_index.min(paragraph.char_offsets.len());
        paragraph.char_offsets.insert(offset_index, raw_offset);
        for offset in paragraph.char_offsets.iter_mut().skip(offset_index + 1) {
            *offset += 8;
        }
    }

    paragraph.char_count += 8;
    paragraph.control_mask |= 0x00000800;
    paragraph.has_para_text = true;
    insert_index
}

fn remove_cell_shape_control_for_cli(
    paragraph: &mut rhwp::model::paragraph::Paragraph,
    control_index: usize,
) -> Result<(), String> {
    if control_index >= paragraph.controls.len() {
        return Err(format!("셀 내 컨트롤 {} 범위 초과", control_index));
    }
    if !matches!(
        paragraph.controls.get(control_index),
        Some(rhwp::model::control::Control::Shape(_))
    ) {
        return Err("지정된 셀 내 컨트롤이 Shape이 아닙니다".to_string());
    }

    let text_chars: Vec<char> = paragraph.text.chars().collect();
    let mut current_control_index = 0usize;
    let mut previous_end: u32 = 0;
    let mut gap_start: Option<u32> = None;
    'scan: for (text_index, ch) in text_chars.iter().enumerate() {
        let offset = paragraph
            .char_offsets
            .get(text_index)
            .copied()
            .unwrap_or(previous_end);
        while previous_end + 8 <= offset && current_control_index < paragraph.controls.len() {
            if current_control_index == control_index {
                gap_start = Some(previous_end);
                break 'scan;
            }
            current_control_index += 1;
            previous_end += 8;
        }
        let char_size = if *ch == '\t' {
            8
        } else {
            ch.len_utf16() as u32
        };
        previous_end = offset + char_size;
    }
    if gap_start.is_none() {
        while current_control_index < paragraph.controls.len() {
            if current_control_index == control_index {
                gap_start = Some(previous_end);
                break;
            }
            current_control_index += 1;
            previous_end += 8;
        }
    }

    if let Some(start) = gap_start {
        let threshold = start + 8;
        for offset in paragraph.char_offsets.iter_mut() {
            if *offset >= threshold {
                *offset -= 8;
            }
        }
    }

    paragraph.controls.remove(control_index);
    if control_index < paragraph.ctrl_data_records.len() {
        paragraph.ctrl_data_records.remove(control_index);
    }
    paragraph.char_count = paragraph.char_count.saturating_sub(8);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn create_hwp_cell_shape_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
    char_offset: usize,
    cell_path_json: &str,
    width: u32,
    height: u32,
    horz_offset: u32,
    vert_offset: u32,
    treat_as_char: bool,
    text_wrap: &str,
    shape_type: &str,
    line_flip_x: bool,
    line_flip_y: bool,
    polygon_points_json: &str,
) -> Result<HwpTableCliResult, String> {
    let cell_path = parse_cell_path_for_cli(cell_path_json)?;
    if cell_path.is_empty() {
        return Err("cell-path는 비어 있을 수 없습니다.".to_string());
    }

    let shape_source = create_hwp_shape_bytes_for_cli(
        data,
        section_idx,
        para_idx,
        0,
        width,
        height,
        horz_offset,
        vert_offset,
        treat_as_char,
        text_wrap,
        shape_type,
        line_flip_x,
        line_flip_y,
        polygon_points_json,
    )?;
    let shape_control = {
        let shape_core = rhwp::document_core::DocumentCore::from_bytes(&shape_source.bytes)
            .map_err(|e| format!("도형 생성 결과 파싱 실패: {}", e))?;
        let paragraph = shape_core
            .document()
            .sections
            .get(section_idx)
            .and_then(|section| section.paragraphs.get(shape_source.para_idx))
            .ok_or_else(|| "도형 생성 결과 문단을 찾을 수 없습니다.".to_string())?;
        match paragraph.controls.get(shape_source.control_idx) {
            Some(rhwp::model::control::Control::Shape(shape)) => shape.clone(),
            _ => return Err("도형 생성 결과 컨트롤이 Shape이 아닙니다.".to_string()),
        }
    };

    let mut core = rhwp::document_core::DocumentCore::from_bytes(data)
        .map_err(|e| format!("HWP 파싱 실패: {}", e))?;
    core.convert_to_editable_native()
        .map_err(|e| format!("편집 가능 변환 실패: {}", e))?;
    let control_idx = {
        let section = core
            .document_mut()
            .sections
            .get_mut(section_idx)
            .ok_or_else(|| format!("구역 인덱스 {} 범위 초과", section_idx))?;
        section.raw_stream = None;
        let paragraph = resolve_cell_paragraph_mut_for_cli(section, para_idx, &cell_path)?;
        insert_cell_shape_control_for_cli(paragraph, char_offset, shape_control)
    };
    let mut details = serde_json::json!({
        "ok": true,
        "operation": "create-shape",
        "container": "cell",
        "paraIdx": para_idx,
        "controlIdx": control_idx,
        "tableControl": cell_path[0].0,
        "cellIndex": cell_path.last().map(|entry| entry.1).unwrap_or(0),
        "cellParaIndex": cell_path.last().map(|entry| entry.2).unwrap_or(0),
        "cellPath": parse_json_value(cell_path_json),
    });
    let (bytes, page_count_before, page_count_after) = serialize_hwp_verified_for_cli(&mut core)?;
    Ok(HwpTableCliResult {
        bytes,
        para_idx,
        control_idx,
        details: {
            if let Some(obj) = details.as_object_mut() {
                obj.insert("shapeType".to_string(), serde_json::json!(shape_type));
            }
            details
        },
        page_count_before,
        page_count_after,
    })
}

#[allow(clippy::too_many_arguments)]
fn create_hwp_cell_shape_at_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
    table_control_idx: usize,
    row: u16,
    col: u16,
    cell_para_idx: usize,
    char_offset: usize,
    width: u32,
    height: u32,
    horz_offset: u32,
    vert_offset: u32,
    treat_as_char: bool,
    text_wrap: &str,
    shape_type: &str,
    line_flip_x: bool,
    line_flip_y: bool,
    polygon_points_json: &str,
) -> Result<HwpTableCliResult, String> {
    let (cell_path, cell_idx) = cell_path_from_row_col_for_cli(
        data,
        section_idx,
        para_idx,
        table_control_idx,
        row,
        col,
        cell_para_idx,
    )?;
    let mut result = create_hwp_cell_shape_bytes_for_cli(
        data,
        section_idx,
        para_idx,
        char_offset,
        &cell_path,
        width,
        height,
        horz_offset,
        vert_offset,
        treat_as_char,
        text_wrap,
        shape_type,
        line_flip_x,
        line_flip_y,
        polygon_points_json,
    )?;
    if let Some(obj) = result.details.as_object_mut() {
        obj.insert("row".to_string(), serde_json::json!(row));
        obj.insert("col".to_string(), serde_json::json!(col));
        obj.insert("cellIndex".to_string(), serde_json::json!(cell_idx));
        obj.insert(
            "cellParaIndex".to_string(),
            serde_json::json!(cell_para_idx),
        );
        obj.insert(
            "tableControl".to_string(),
            serde_json::json!(table_control_idx),
        );
    }
    Ok(result)
}

fn set_hwp_cell_shape_text_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
    cell_path_json: &str,
    control_idx: usize,
    textbox_para_idx: usize,
    text: &str,
) -> Result<HwpEditCliResult, String> {
    let cell_path = parse_cell_path_for_cli(cell_path_json)?;
    if cell_path.is_empty() {
        return Err("cell-path는 비어 있을 수 없습니다.".to_string());
    }

    let mut core = rhwp::document_core::DocumentCore::from_bytes(data)
        .map_err(|e| format!("HWP 파싱 실패: {}", e))?;
    core.convert_to_editable_native()
        .map_err(|e| format!("편집 가능 변환 실패: {}", e))?;
    {
        let section = core
            .document_mut()
            .sections
            .get_mut(section_idx)
            .ok_or_else(|| format!("구역 인덱스 {} 범위 초과", section_idx))?;
        section.raw_stream = None;
        let cell_paragraph = resolve_cell_paragraph_mut_for_cli(section, para_idx, &cell_path)?;
        let shape = match cell_paragraph.controls.get_mut(control_idx) {
            Some(rhwp::model::control::Control::Shape(shape)) => shape,
            Some(_) => return Err("지정된 셀 내 컨트롤이 Shape이 아닙니다".to_string()),
            None => return Err(format!("셀 내 컨트롤 {} 범위 초과", control_idx)),
        };
        let textbox = shape
            .drawing_mut()
            .and_then(|drawing| drawing.text_box.as_mut())
            .ok_or_else(|| "지정된 셀 도형이 텍스트박스가 아닙니다".to_string())?;
        while textbox.paragraphs.len() <= textbox_para_idx {
            let template = textbox.paragraphs.last().cloned();
            let mut paragraph = rhwp::model::paragraph::Paragraph::new_empty();
            if let Some(template) = template {
                paragraph.para_shape_id = template.para_shape_id;
                paragraph.style_id = template.style_id;
                paragraph.raw_header_extra = template.raw_header_extra.clone();
                if let Some(line_seg) = template.line_segs.first() {
                    let mut inherited = line_seg.clone();
                    inherited.text_start = 0;
                    paragraph.line_segs = vec![inherited];
                }
                if let Some(char_shape) = template.char_shapes.first() {
                    let mut inherited = char_shape.clone();
                    inherited.start_pos = 0;
                    paragraph.char_shapes = vec![inherited];
                }
            }
            textbox.paragraphs.push(paragraph);
        }
        let paragraph = textbox
            .paragraphs
            .get_mut(textbox_para_idx)
            .ok_or_else(|| format!("글상자 문단 {} 범위 초과", textbox_para_idx))?;
        paragraph.text = text.to_string();
        paragraph.char_count = text.encode_utf16().count() as u32;
        paragraph.char_offsets = text
            .chars()
            .scan(0u32, |offset, ch| {
                let current = *offset;
                *offset += if ch == '\t' { 8 } else { ch.len_utf16() as u32 };
                Some(current)
            })
            .collect();
        paragraph.has_para_text = true;
    }
    let (bytes, page_count_before, page_count_after) = serialize_hwp_verified_for_cli(&mut core)?;
    Ok(HwpEditCliResult {
        bytes,
        details: serde_json::json!({
            "ok": true,
            "operation": "set-cell-shape-text",
            "container": "cell_textbox",
            "section": section_idx,
            "paraIdx": para_idx,
            "controlIdx": control_idx,
            "tableControl": cell_path[0].0,
            "cellIndex": cell_path.last().map(|entry| entry.1).unwrap_or(0),
            "cellParaIndex": cell_path.last().map(|entry| entry.2).unwrap_or(0),
            "cellPath": parse_json_value(cell_path_json),
            "textboxParagraph": textbox_para_idx,
            "text": text,
        }),
        page_count_before,
        page_count_after,
    })
}

#[allow(clippy::too_many_arguments)]
fn set_hwp_cell_shape_text_at_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
    table_control_idx: usize,
    row: u16,
    col: u16,
    cell_para_idx: usize,
    shape_control_idx: usize,
    textbox_para_idx: usize,
    text: &str,
) -> Result<HwpEditCliResult, String> {
    let (cell_path, cell_idx) = cell_path_from_row_col_for_cli(
        data,
        section_idx,
        para_idx,
        table_control_idx,
        row,
        col,
        cell_para_idx,
    )?;
    let mut result = set_hwp_cell_shape_text_bytes_for_cli(
        data,
        section_idx,
        para_idx,
        &cell_path,
        shape_control_idx,
        textbox_para_idx,
        text,
    )?;
    if let Some(obj) = result.details.as_object_mut() {
        obj.insert("row".to_string(), serde_json::json!(row));
        obj.insert("col".to_string(), serde_json::json!(col));
        obj.insert("cellIndex".to_string(), serde_json::json!(cell_idx));
        obj.insert(
            "cellParaIndex".to_string(),
            serde_json::json!(cell_para_idx),
        );
        obj.insert(
            "tableControl".to_string(),
            serde_json::json!(table_control_idx),
        );
    }
    Ok(result)
}

#[allow(clippy::too_many_arguments)]
fn set_hwp_cell_shape_char_format_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
    cell_path_json: &str,
    control_idx: usize,
    textbox_para_idx: usize,
    start_offset: usize,
    end_offset: usize,
    props_json: &str,
) -> Result<HwpEditCliResult, String> {
    let mut cell_path = parse_cell_path_for_cli(cell_path_json)?;
    if cell_path.is_empty() {
        return Err("cell-path는 비어 있을 수 없습니다.".to_string());
    }
    let table_control = cell_path[0].0;
    let cell_index = cell_path.last().map(|entry| entry.1).unwrap_or(0);
    let cell_para_index = cell_path.last().map(|entry| entry.2).unwrap_or(0);
    cell_path.push((control_idx, 0, textbox_para_idx));

    let mut core = rhwp::document_core::DocumentCore::from_bytes(data)
        .map_err(|e| format!("HWP 파싱 실패: {}", e))?;
    core.convert_to_editable_native()
        .map_err(|e| format!("편집 가능 변환 실패: {}", e))?;
    core.apply_char_format_in_cell_by_path_native(
        section_idx,
        para_idx,
        &cell_path,
        start_offset,
        end_offset,
        props_json,
    )
    .map_err(|e| format!("셀 글상자 글자 서식 설정 실패: {}", e))?;
    let (bytes, page_count_before, page_count_after) = serialize_hwp_verified_for_cli(&mut core)?;
    Ok(HwpEditCliResult {
        bytes,
        details: serde_json::json!({
            "ok": true,
            "operation": "set-cell-shape-char-format",
            "container": "cell_textbox",
            "section": section_idx,
            "paraIdx": para_idx,
            "controlIdx": control_idx,
            "tableControl": table_control,
            "cellIndex": cell_index,
            "cellParaIndex": cell_para_index,
            "cellPath": parse_json_value(cell_path_json),
            "textboxParagraph": textbox_para_idx,
            "start": start_offset,
            "end": end_offset,
        }),
        page_count_before,
        page_count_after,
    })
}

#[allow(clippy::too_many_arguments)]
fn set_hwp_cell_shape_char_format_at_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
    table_control_idx: usize,
    row: u16,
    col: u16,
    cell_para_idx: usize,
    shape_control_idx: usize,
    textbox_para_idx: usize,
    start_offset: usize,
    end_offset: usize,
    props_json: &str,
) -> Result<HwpEditCliResult, String> {
    let (cell_path, cell_idx) = cell_path_from_row_col_for_cli(
        data,
        section_idx,
        para_idx,
        table_control_idx,
        row,
        col,
        cell_para_idx,
    )?;
    let mut result = set_hwp_cell_shape_char_format_bytes_for_cli(
        data,
        section_idx,
        para_idx,
        &cell_path,
        shape_control_idx,
        textbox_para_idx,
        start_offset,
        end_offset,
        props_json,
    )?;
    if let Some(obj) = result.details.as_object_mut() {
        obj.insert("row".to_string(), serde_json::json!(row));
        obj.insert("col".to_string(), serde_json::json!(col));
        obj.insert("cellIndex".to_string(), serde_json::json!(cell_idx));
        obj.insert(
            "cellParaIndex".to_string(),
            serde_json::json!(cell_para_idx),
        );
        obj.insert(
            "tableControl".to_string(),
            serde_json::json!(table_control_idx),
        );
    }
    Ok(result)
}

fn set_hwp_cell_shape_para_format_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
    cell_path_json: &str,
    control_idx: usize,
    textbox_para_idx: usize,
    props_json: &str,
) -> Result<HwpEditCliResult, String> {
    let mut cell_path = parse_cell_path_for_cli(cell_path_json)?;
    if cell_path.is_empty() {
        return Err("cell-path는 비어 있을 수 없습니다.".to_string());
    }
    let table_control = cell_path[0].0;
    let cell_index = cell_path.last().map(|entry| entry.1).unwrap_or(0);
    let cell_para_index = cell_path.last().map(|entry| entry.2).unwrap_or(0);
    cell_path.push((control_idx, 0, textbox_para_idx));

    let mut core = rhwp::document_core::DocumentCore::from_bytes(data)
        .map_err(|e| format!("HWP 파싱 실패: {}", e))?;
    core.convert_to_editable_native()
        .map_err(|e| format!("편집 가능 변환 실패: {}", e))?;
    core.apply_para_format_in_cell_by_path_native(section_idx, para_idx, &cell_path, props_json)
        .map_err(|e| format!("셀 글상자 문단 서식 설정 실패: {}", e))?;
    let (bytes, page_count_before, page_count_after) = serialize_hwp_verified_for_cli(&mut core)?;
    Ok(HwpEditCliResult {
        bytes,
        details: serde_json::json!({
            "ok": true,
            "operation": "set-cell-shape-para-format",
            "container": "cell_textbox",
            "section": section_idx,
            "paraIdx": para_idx,
            "controlIdx": control_idx,
            "tableControl": table_control,
            "cellIndex": cell_index,
            "cellParaIndex": cell_para_index,
            "cellPath": parse_json_value(cell_path_json),
            "textboxParagraph": textbox_para_idx,
        }),
        page_count_before,
        page_count_after,
    })
}

#[allow(clippy::too_many_arguments)]
fn set_hwp_cell_shape_para_format_at_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
    table_control_idx: usize,
    row: u16,
    col: u16,
    cell_para_idx: usize,
    shape_control_idx: usize,
    textbox_para_idx: usize,
    props_json: &str,
) -> Result<HwpEditCliResult, String> {
    let (cell_path, cell_idx) = cell_path_from_row_col_for_cli(
        data,
        section_idx,
        para_idx,
        table_control_idx,
        row,
        col,
        cell_para_idx,
    )?;
    let mut result = set_hwp_cell_shape_para_format_bytes_for_cli(
        data,
        section_idx,
        para_idx,
        &cell_path,
        shape_control_idx,
        textbox_para_idx,
        props_json,
    )?;
    if let Some(obj) = result.details.as_object_mut() {
        obj.insert("row".to_string(), serde_json::json!(row));
        obj.insert("col".to_string(), serde_json::json!(col));
        obj.insert("cellIndex".to_string(), serde_json::json!(cell_idx));
        obj.insert(
            "cellParaIndex".to_string(),
            serde_json::json!(cell_para_idx),
        );
        obj.insert(
            "tableControl".to_string(),
            serde_json::json!(table_control_idx),
        );
    }
    Ok(result)
}

fn get_hwp_shape_properties_json_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
    control_idx: usize,
) -> Result<serde_json::Value, String> {
    let core = rhwp::document_core::DocumentCore::from_bytes(data)
        .map_err(|e| format!("HWP 파싱 실패: {}", e))?;
    let details_json = core
        .get_shape_properties_native(section_idx, para_idx, control_idx)
        .map_err(|e| format!("도형 속성 조회 실패: {}", e))?;
    let mut details = parse_json_value(&details_json);
    if let Some(obj) = details.as_object_mut() {
        obj.insert("ok".to_string(), serde_json::json!(true));
    }
    Ok(details)
}

fn get_hwp_cell_shape_properties_json_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
    cell_path_json: &str,
    control_idx: usize,
) -> Result<serde_json::Value, String> {
    let cell_path = parse_cell_path_for_cli(cell_path_json)?;
    if cell_path.is_empty() {
        return Err("cell-path는 비어 있을 수 없습니다.".to_string());
    }
    let core = rhwp::document_core::DocumentCore::from_bytes(data)
        .map_err(|e| format!("HWP 파싱 실패: {}", e))?;
    let details_json = core
        .get_cell_shape_properties_by_path_native(
            section_idx,
            para_idx,
            cell_path_json,
            control_idx,
        )
        .map_err(|e| format!("셀 도형 속성 조회 실패: {}", e))?;
    let mut details = parse_json_value(&details_json);
    if let Some(obj) = details.as_object_mut() {
        obj.insert("ok".to_string(), serde_json::json!(true));
        obj.insert("container".to_string(), serde_json::json!("cell"));
    }
    Ok(details)
}

#[allow(clippy::too_many_arguments)]
fn get_hwp_cell_shape_properties_at_json_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
    table_control_idx: usize,
    row: u16,
    col: u16,
    cell_para_idx: usize,
    shape_control_idx: usize,
) -> Result<serde_json::Value, String> {
    let (cell_path, cell_idx) = cell_path_from_row_col_for_cli(
        data,
        section_idx,
        para_idx,
        table_control_idx,
        row,
        col,
        cell_para_idx,
    )?;
    let mut details = get_hwp_cell_shape_properties_json_for_cli(
        data,
        section_idx,
        para_idx,
        &cell_path,
        shape_control_idx,
    )?;
    if let Some(obj) = details.as_object_mut() {
        obj.insert("row".to_string(), serde_json::json!(row));
        obj.insert("col".to_string(), serde_json::json!(col));
        obj.insert("cellIndex".to_string(), serde_json::json!(cell_idx));
        obj.insert(
            "cellParaIndex".to_string(),
            serde_json::json!(cell_para_idx),
        );
        obj.insert(
            "tableControl".to_string(),
            serde_json::json!(table_control_idx),
        );
    }
    Ok(details)
}

fn set_hwp_shape_properties_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
    control_idx: usize,
    props_json: &str,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "set-shape-properties", |core| {
        core.set_shape_properties_native(section_idx, para_idx, control_idx, props_json)
            .map_err(|e| format!("도형 속성 설정 실패: {}", e))
    })
}

fn set_hwp_cell_shape_properties_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
    cell_path_json: &str,
    control_idx: usize,
    props_json: &str,
) -> Result<HwpEditCliResult, String> {
    let cell_path = parse_cell_path_for_cli(cell_path_json)?;
    if cell_path.is_empty() {
        return Err("cell-path는 비어 있을 수 없습니다.".to_string());
    }
    let mut result =
        edit_hwp_table_structure_bytes_for_cli(data, "set-shape-properties", |core| {
            core.set_cell_shape_properties_by_path_native(
                section_idx,
                para_idx,
                cell_path_json,
                control_idx,
                props_json,
            )
            .map_err(|e| format!("셀 도형 속성 설정 실패: {}", e))
        })?;
    if let Some(obj) = result.details.as_object_mut() {
        obj.insert("container".to_string(), serde_json::json!("cell"));
    }
    Ok(result)
}

#[allow(clippy::too_many_arguments)]
fn set_hwp_cell_shape_properties_at_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
    table_control_idx: usize,
    row: u16,
    col: u16,
    cell_para_idx: usize,
    shape_control_idx: usize,
    props_json: &str,
) -> Result<HwpEditCliResult, String> {
    let (cell_path, cell_idx) = cell_path_from_row_col_for_cli(
        data,
        section_idx,
        para_idx,
        table_control_idx,
        row,
        col,
        cell_para_idx,
    )?;
    let mut result = set_hwp_cell_shape_properties_bytes_for_cli(
        data,
        section_idx,
        para_idx,
        &cell_path,
        shape_control_idx,
        props_json,
    )?;
    if let Some(obj) = result.details.as_object_mut() {
        obj.insert("row".to_string(), serde_json::json!(row));
        obj.insert("col".to_string(), serde_json::json!(col));
        obj.insert("cellIndex".to_string(), serde_json::json!(cell_idx));
        obj.insert(
            "cellParaIndex".to_string(),
            serde_json::json!(cell_para_idx),
        );
        obj.insert(
            "tableControl".to_string(),
            serde_json::json!(table_control_idx),
        );
    }
    Ok(result)
}

fn delete_hwp_shape_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
    control_idx: usize,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "delete-shape", |core| {
        core.delete_shape_control_native(section_idx, para_idx, control_idx)
            .map_err(|e| format!("도형 삭제 실패: {}", e))
    })
}

fn delete_hwp_cell_shape_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
    cell_path_json: &str,
    control_idx: usize,
) -> Result<HwpEditCliResult, String> {
    let cell_path = parse_cell_path_for_cli(cell_path_json)?;
    if cell_path.is_empty() {
        return Err("cell-path는 비어 있을 수 없습니다.".to_string());
    }
    let mut core = rhwp::document_core::DocumentCore::from_bytes(data)
        .map_err(|e| format!("HWP 파싱 실패: {}", e))?;
    core.convert_to_editable_native()
        .map_err(|e| format!("편집 가능 변환 실패: {}", e))?;
    {
        let section = core
            .document_mut()
            .sections
            .get_mut(section_idx)
            .ok_or_else(|| format!("구역 인덱스 {} 범위 초과", section_idx))?;
        section.raw_stream = None;
        let paragraph = resolve_cell_paragraph_mut_for_cli(section, para_idx, &cell_path)?;
        remove_cell_shape_control_for_cli(paragraph, control_idx)?;
    }
    let (bytes, page_count_before, page_count_after) = serialize_hwp_verified_for_cli(&mut core)?;
    Ok(HwpEditCliResult {
        bytes,
        details: serde_json::json!({
            "ok": true,
            "operation": "delete-shape",
            "container": "cell",
            "tableControl": cell_path[0].0,
            "cellIndex": cell_path.last().map(|entry| entry.1).unwrap_or(0),
            "cellParaIndex": cell_path.last().map(|entry| entry.2).unwrap_or(0),
            "cellPath": parse_json_value(cell_path_json),
        }),
        page_count_before,
        page_count_after,
    })
}

#[allow(clippy::too_many_arguments)]
fn delete_hwp_cell_shape_at_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
    table_control_idx: usize,
    row: u16,
    col: u16,
    cell_para_idx: usize,
    shape_control_idx: usize,
) -> Result<HwpEditCliResult, String> {
    let (cell_path, cell_idx) = cell_path_from_row_col_for_cli(
        data,
        section_idx,
        para_idx,
        table_control_idx,
        row,
        col,
        cell_para_idx,
    )?;
    let mut result = delete_hwp_cell_shape_bytes_for_cli(
        data,
        section_idx,
        para_idx,
        &cell_path,
        shape_control_idx,
    )?;
    if let Some(obj) = result.details.as_object_mut() {
        obj.insert("row".to_string(), serde_json::json!(row));
        obj.insert("col".to_string(), serde_json::json!(col));
        obj.insert("cellIndex".to_string(), serde_json::json!(cell_idx));
        obj.insert(
            "cellParaIndex".to_string(),
            serde_json::json!(cell_para_idx),
        );
        obj.insert(
            "tableControl".to_string(),
            serde_json::json!(table_control_idx),
        );
    }
    Ok(result)
}

fn change_hwp_shape_z_order_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
    control_idx: usize,
    operation: &str,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "change-shape-z-order", |core| {
        core.change_shape_z_order_native(section_idx, para_idx, control_idx, operation)
            .map_err(|e| format!("도형 배치 순서 변경 실패: {}", e))
    })
}

fn group_hwp_shapes_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    targets_json: &str,
) -> Result<HwpTableCliResult, String> {
    let targets = parse_shape_targets_for_cli(targets_json)?;
    let mut core = rhwp::document_core::DocumentCore::from_bytes(data)
        .map_err(|e| format!("HWP 파싱 실패: {}", e))?;
    core.convert_to_editable_native()
        .map_err(|e| format!("편집 가능 변환 실패: {}", e))?;
    let details_json = core
        .group_shapes_native(section_idx, &targets)
        .map_err(|e| format!("도형 묶기 실패: {}", e))?;
    let mut details = parse_json_value(&details_json);
    if let Some(obj) = details.as_object_mut() {
        obj.insert("operation".to_string(), serde_json::json!("group-shapes"));
    }
    let para_idx = details
        .get("paraIdx")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| format!("도형 묶기 결과에 paraIdx가 없습니다: {}", details_json))?
        as usize;
    let control_idx = details
        .get("controlIdx")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| format!("도형 묶기 결과에 controlIdx가 없습니다: {}", details_json))?
        as usize;
    let (bytes, page_count_before, page_count_after) = serialize_hwp_verified_for_cli(&mut core)?;
    Ok(HwpTableCliResult {
        bytes,
        para_idx,
        control_idx,
        details,
        page_count_before,
        page_count_after,
    })
}

fn ungroup_hwp_shape_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    para_idx: usize,
    control_idx: usize,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "ungroup-shape", |core| {
        core.ungroup_shape_native(section_idx, para_idx, control_idx)
            .map_err(|e| format!("도형 풀기 실패: {}", e))
    })
}

fn read_text_argument(
    inline_text: Option<String>,
    text_file: Option<String>,
) -> Result<String, String> {
    match (inline_text, text_file) {
        (Some(text), None) => Ok(text),
        (None, Some(path)) => fs::read_to_string(&path)
            .map_err(|e| format!("텍스트 파일 읽기 실패 - {}: {}", path, e)),
        (Some(_), Some(_)) => Err("--text와 --text-file은 동시에 사용할 수 없습니다.".to_string()),
        (None, None) => Err("--text 또는 --text-file 중 하나가 필요합니다.".to_string()),
    }
}

fn read_optional_text_argument(
    inline_text: Option<String>,
    text_file: Option<String>,
) -> Result<Option<String>, String> {
    match (inline_text, text_file) {
        (Some(text), None) => Ok(Some(text)),
        (None, Some(path)) => fs::read_to_string(&path)
            .map(Some)
            .map_err(|e| format!("텍스트 파일 읽기 실패 - {}: {}", path, e)),
        (Some(_), Some(_)) => Err("--text와 --text-file은 동시에 사용할 수 없습니다.".to_string()),
        (None, None) => Ok(None),
    }
}

fn read_json_argument(
    inline_json: Option<String>,
    json_file: Option<String>,
) -> Result<String, String> {
    match (inline_json, json_file) {
        (Some(json), None) => Ok(json),
        (None, Some(path)) => {
            fs::read_to_string(&path).map_err(|e| format!("JSON 파일 읽기 실패 - {}: {}", path, e))
        }
        (Some(_), Some(_)) => Err("--json과 --json-file은 동시에 사용할 수 없습니다.".to_string()),
        (None, None) => Err("--json 또는 --json-file 중 하나가 필요합니다.".to_string()),
    }
}

fn write_hwp_cli_output(path: &str, bytes: &[u8]) -> Result<(), String> {
    let out = Path::new(path);
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("출력 디렉토리 생성 실패: {}", e))?;
    }
    fs::write(out, bytes).map_err(|e| format!("HWP 파일 저장 실패 - {}: {}", path, e))
}

fn exit_cli_error(message: &str) -> ! {
    eprintln!("오류: {}", message);
    std::process::exit(1);
}

fn create_hwp(args: &[String]) {
    let mut inline_text: Option<String> = None;
    let mut text_file: Option<String> = None;
    let mut output_path: Option<String> = None;
    let mut template_path: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--text" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--text 뒤에 텍스트가 필요합니다.");
                }
                inline_text = Some(args[i].clone());
            }
            "--text-file" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--text-file 뒤에 경로가 필요합니다.");
                }
                text_file = Some(args[i].clone());
            }
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("-o/--output 뒤에 경로가 필요합니다.");
                }
                output_path = Some(args[i].clone());
            }
            "--template" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--template 뒤에 경로가 필요합니다.");
                }
                template_path = Some(args[i].clone());
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }

    let text = read_text_argument(inline_text, text_file).unwrap_or_else(|e| exit_cli_error(&e));
    let output = output_path.unwrap_or_else(|| exit_cli_error("-o <출력.hwp>가 필요합니다."));
    let result = create_hwp_bytes_from_text_for_cli(&text, template_path.as_deref())
        .unwrap_or_else(|e| exit_cli_error(&e));
    write_hwp_cli_output(&output, &result.bytes).unwrap_or_else(|e| exit_cli_error(&e));
    println!(
        "{}",
        serde_json::json!({
            "ok": true,
            "path": output,
            "bytes": result.bytes.len(),
            "paragraphCount": result.paragraph_count,
            "pageCountBefore": result.page_count_before,
            "pageCountAfter": result.page_count_after,
        })
    );
}

fn replace_text_cli(args: &[String]) {
    if args.is_empty() {
        exit_cli_error(
            "사용법: rhwp replace-text <파일.hwp> --old <검색어> --new <대체문구> -o <출력.hwp>",
        );
    }
    let input = args[0].clone();
    let mut old: Option<String> = None;
    let mut new: Option<String> = None;
    let mut output_path: Option<String> = None;
    let mut replace_all = false;
    let mut case_sensitive = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--old" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--old 뒤에 검색어가 필요합니다.");
                }
                old = Some(args[i].clone());
            }
            "--new" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--new 뒤에 대체문구가 필요합니다.");
                }
                new = Some(args[i].clone());
            }
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("-o/--output 뒤에 경로가 필요합니다.");
                }
                output_path = Some(args[i].clone());
            }
            "--all" => replace_all = true,
            "--case-sensitive" => case_sensitive = true,
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }

    let old = old.unwrap_or_else(|| exit_cli_error("--old <검색어>가 필요합니다."));
    let new = new.unwrap_or_else(|| exit_cli_error("--new <대체문구>가 필요합니다."));
    let output = output_path.unwrap_or_else(|| input.clone());
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let result = replace_hwp_text_bytes_for_cli(&data, &old, &new, replace_all, case_sensitive)
        .unwrap_or_else(|e| exit_cli_error(&e));
    write_hwp_cli_output(&output, &result.bytes).unwrap_or_else(|e| exit_cli_error(&e));
    println!(
        "{}",
        serde_json::json!({
            "ok": true,
            "path": output,
            "bytes": result.bytes.len(),
            "count": result.count,
            "details": result.details,
            "pageCountBefore": result.page_count_before,
            "pageCountAfter": result.page_count_after,
        })
    );
}

fn list_fields_cli(args: &[String]) {
    if args.len() != 1 {
        exit_cli_error("사용법: rhwp list-fields <파일.hwp>");
    }
    let input = &args[0];
    let data = fs::read(input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let mut result = list_hwp_fields_json_for_cli(&data).unwrap_or_else(|e| exit_cli_error(&e));
    if let serde_json::Value::Object(ref mut obj) = result {
        obj.insert("path".to_string(), serde_json::Value::String(input.clone()));
    }
    println!("{}", result);
}

fn list_forms_cli(args: &[String]) {
    if args.len() != 1 {
        exit_cli_error("사용법: rhwp list-forms <파일.hwp>");
    }
    let input = &args[0];
    let data = fs::read(input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let mut result = list_hwp_forms_json_for_cli(&data).unwrap_or_else(|e| exit_cli_error(&e));
    if let serde_json::Value::Object(ref mut obj) = result {
        obj.insert("path".to_string(), serde_json::Value::String(input.clone()));
    }
    println!("{}", result);
}

fn list_objects_cli(args: &[String]) {
    if args.len() != 1 {
        exit_cli_error("사용법: rhwp list-objects <파일.hwp>");
    }
    let input = &args[0];
    let data = fs::read(input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let mut result = list_hwp_objects_json_for_cli(&data).unwrap_or_else(|e| exit_cli_error(&e));
    if let serde_json::Value::Object(ref mut obj) = result {
        obj.insert("path".to_string(), serde_json::Value::String(input.clone()));
    }
    println!("{}", result);
}

fn insert_clickhere_field_cli(args: &[String]) {
    if args.is_empty() {
        exit_cli_error("사용법: rhwp insert-clickhere-field <파일.hwp> --section N --para N --offset N --name <필드명> [--guide <안내문>] [--memo <메모>] [--value <초기값>] [--cell-path JSON|--table-ctrl N --row R --col C [--cell-para N]] [--ctrl N --textbox-para N] -o <출력.hwp>");
    }
    let input = args[0].clone();
    let mut section: Option<String> = None;
    let mut para: Option<String> = None;
    let mut ctrl: Option<String> = None;
    let mut cell: Option<String> = None;
    let mut cell_path: Option<String> = None;
    let mut table_ctrl: Option<String> = None;
    let mut row: Option<String> = None;
    let mut col: Option<String> = None;
    let mut cell_para: Option<String> = None;
    let mut nested_para: Option<String> = None;
    let mut textbox = false;
    let mut offset: Option<String> = None;
    let mut name: Option<String> = None;
    let mut guide: Option<String> = None;
    let mut memo: Option<String> = None;
    let mut value: Option<String> = None;
    let mut output_path: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--section" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--section 뒤에 번호가 필요합니다.");
                }
                section = Some(args[i].clone());
            }
            "--para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--para 뒤에 번호가 필요합니다.");
                }
                para = Some(args[i].clone());
            }
            "--ctrl" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--ctrl 뒤에 번호가 필요합니다.");
                }
                ctrl = Some(args[i].clone());
            }
            "--cell" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--cell 뒤에 번호가 필요합니다.");
                }
                cell = Some(args[i].clone());
            }
            "--cell-path" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--cell-path 뒤에 JSON 문자열이 필요합니다.");
                }
                cell_path = Some(args[i].clone());
            }
            "--cell-path-file" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--cell-path-file 뒤에 경로가 필요합니다.");
                }
                cell_path = Some(fs::read_to_string(&args[i]).unwrap_or_else(|e| {
                    exit_cli_error(&format!("cellPath 파일 읽기 실패: {}", e))
                }));
            }
            "--table-ctrl" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--table-ctrl 뒤에 값이 필요합니다.");
                }
                table_ctrl = Some(args[i].clone());
            }
            "--row" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--row 뒤에 값이 필요합니다.");
                }
                row = Some(args[i].clone());
            }
            "--col" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--col 뒤에 값이 필요합니다.");
                }
                col = Some(args[i].clone());
            }
            "--cell-para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--cell-para 뒤에 값이 필요합니다.");
                }
                cell_para = Some(args[i].clone());
            }
            "--textbox-para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--textbox-para 뒤에 번호가 필요합니다.");
                }
                nested_para = Some(args[i].clone());
            }
            "--textbox" => {
                textbox = true;
            }
            "--offset" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--offset 뒤에 번호가 필요합니다.");
                }
                offset = Some(args[i].clone());
            }
            "--name" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--name 뒤에 필드명이 필요합니다.");
                }
                name = Some(args[i].clone());
            }
            "--guide" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--guide 뒤에 안내문이 필요합니다.");
                }
                guide = Some(args[i].clone());
            }
            "--memo" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--memo 뒤에 메모가 필요합니다.");
                }
                memo = Some(args[i].clone());
            }
            "--value" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--value 뒤에 초기값이 필요합니다.");
                }
                value = Some(args[i].clone());
            }
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("-o/--output 뒤에 경로가 필요합니다.");
                }
                output_path = Some(args[i].clone());
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }

    let section = parse_usize_cli(section, "--section");
    let para = parse_usize_cli(para, "--para");
    let offset = parse_usize_cli(offset, "--offset");
    let name = name.unwrap_or_else(|| exit_cli_error("--name <필드명>이 필요합니다."));
    let guide = guide.unwrap_or_else(|| name.clone());
    let memo = memo.unwrap_or_default();
    let value = value.unwrap_or_default();
    let output = output_path.unwrap_or_else(|| input.clone());
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let cell_location =
        parse_table_cell_location_cli(&cell_path, table_ctrl, row, col, cell_para.clone());
    if let Some((table_ctrl, row, col, cell_para)) = cell_location {
        if cell.is_some() || textbox {
            exit_cli_error(
                "--table-ctrl/--row/--col은 --cell 또는 --textbox와 함께 사용할 수 없습니다.",
            );
        }
        let ctrl = parse_usize_cli(ctrl, "--ctrl");
        let textbox_para = parse_usize_cli(
            nested_para.or_else(|| Some("0".to_string())),
            "--textbox-para",
        );
        let result = insert_hwp_cell_shape_clickhere_field_at_bytes_for_cli(
            &data,
            section,
            para,
            table_ctrl,
            row,
            col,
            cell_para,
            ctrl,
            textbox_para,
            offset,
            &name,
            &guide,
            &memo,
            &value,
        )
        .unwrap_or_else(|e| exit_cli_error(&e));
        write_hwp_cli_output(&output, &result.bytes).unwrap_or_else(|e| exit_cli_error(&e));
        println!(
            "{}",
            serde_json::json!({
                "ok": true,
                "path": output,
                "bytes": result.bytes.len(),
                "field": name,
                "guide": guide,
                "value": value,
                "details": result.details,
                "pageCountBefore": result.page_count_before,
                "pageCountAfter": result.page_count_after,
            })
        );
        return;
    }
    if let Some(cell_path) = cell_path {
        if cell.is_some() || textbox {
            exit_cli_error("--cell-path는 --cell 또는 --textbox와 함께 사용할 수 없습니다.");
        }
        let ctrl = parse_usize_cli(ctrl, "--ctrl");
        let textbox_para = parse_usize_cli(
            nested_para.or_else(|| Some("0".to_string())),
            "--textbox-para",
        );
        let result = insert_hwp_clickhere_field_by_path_bytes_for_cli(
            &data,
            section,
            para,
            &cell_path,
            ctrl,
            textbox_para,
            offset,
            &name,
            &guide,
            &memo,
            &value,
        )
        .unwrap_or_else(|e| exit_cli_error(&e));
        write_hwp_cli_output(&output, &result.bytes).unwrap_or_else(|e| exit_cli_error(&e));
        println!(
            "{}",
            serde_json::json!({
                "ok": true,
                "path": output,
                "bytes": result.bytes.len(),
                "field": name,
                "guide": guide,
                "value": value,
                "details": result.details,
                "pageCountBefore": result.page_count_before,
                "pageCountAfter": result.page_count_after,
            })
        );
        return;
    }
    if ctrl.is_some() || cell.is_some() || nested_para.is_some() || textbox {
        let ctrl = parse_usize_cli(ctrl, "--ctrl");
        let cell_idx = if textbox {
            if cell.is_some() {
                exit_cli_error("--textbox와 --cell은 함께 사용할 수 없습니다.");
            }
            0
        } else {
            parse_usize_cli(cell, "--cell")
        };
        let cell_para = parse_usize_cli(
            nested_para.or(cell_para).or_else(|| Some("0".to_string())),
            if textbox {
                "--textbox-para"
            } else {
                "--cell-para"
            },
        );
        let result = insert_hwp_nested_clickhere_field_bytes_for_cli(
            &data, section, para, ctrl, cell_idx, cell_para, offset, textbox, &name, &guide, &memo,
            &value,
        )
        .unwrap_or_else(|e| exit_cli_error(&e));
        write_hwp_cli_output(&output, &result.bytes).unwrap_or_else(|e| exit_cli_error(&e));
        println!(
            "{}",
            serde_json::json!({
                "ok": true,
                "path": output,
                "bytes": result.bytes.len(),
                "field": name,
                "guide": guide,
                "value": value,
                "details": result.details,
                "pageCountBefore": result.page_count_before,
                "pageCountAfter": result.page_count_after,
            })
        );
        return;
    }
    let result = insert_hwp_clickhere_field_bytes_for_cli(
        &data, section, para, offset, &name, &guide, &memo, &value,
    )
    .unwrap_or_else(|e| exit_cli_error(&e));
    write_hwp_cli_output(&output, &result.bytes).unwrap_or_else(|e| exit_cli_error(&e));
    println!(
        "{}",
        serde_json::json!({
            "ok": true,
            "path": output,
            "bytes": result.bytes.len(),
            "field": name,
            "guide": guide,
            "value": value,
            "details": result.details,
            "pageCountBefore": result.page_count_before,
            "pageCountAfter": result.page_count_after,
        })
    );
}

fn get_field_info_cli(args: &[String]) {
    if args.is_empty() {
        exit_cli_error("사용법: rhwp get-field-info <파일.hwp> --section N --para N [--cell-path JSON|--table-ctrl N --row R --col C [--cell-para N]] [--ctrl N --textbox-para N] --offset N");
    }
    let input = args[0].clone();
    let mut section: Option<String> = None;
    let mut para: Option<String> = None;
    let mut ctrl: Option<String> = None;
    let mut cell: Option<String> = None;
    let mut cell_path: Option<String> = None;
    let mut table_ctrl: Option<String> = None;
    let mut row: Option<String> = None;
    let mut col: Option<String> = None;
    let mut cell_para: Option<String> = None;
    let mut nested_para: Option<String> = None;
    let mut textbox = false;
    let mut offset: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--section" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--section 뒤에 번호가 필요합니다.");
                }
                section = Some(args[i].clone());
            }
            "--para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--para 뒤에 번호가 필요합니다.");
                }
                para = Some(args[i].clone());
            }
            "--ctrl" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--ctrl 뒤에 번호가 필요합니다.");
                }
                ctrl = Some(args[i].clone());
            }
            "--cell" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--cell 뒤에 번호가 필요합니다.");
                }
                cell = Some(args[i].clone());
            }
            "--cell-path" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--cell-path 뒤에 JSON 문자열이 필요합니다.");
                }
                cell_path = Some(args[i].clone());
            }
            "--cell-path-file" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--cell-path-file 뒤에 경로가 필요합니다.");
                }
                cell_path = Some(fs::read_to_string(&args[i]).unwrap_or_else(|e| {
                    exit_cli_error(&format!("cellPath 파일 읽기 실패: {}", e))
                }));
            }
            "--table-ctrl" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--table-ctrl 뒤에 값이 필요합니다.");
                }
                table_ctrl = Some(args[i].clone());
            }
            "--row" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--row 뒤에 값이 필요합니다.");
                }
                row = Some(args[i].clone());
            }
            "--col" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--col 뒤에 값이 필요합니다.");
                }
                col = Some(args[i].clone());
            }
            "--cell-para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--cell-para 뒤에 값이 필요합니다.");
                }
                cell_para = Some(args[i].clone());
            }
            "--textbox-para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--textbox-para 뒤에 번호가 필요합니다.");
                }
                nested_para = Some(args[i].clone());
            }
            "--textbox" => {
                textbox = true;
            }
            "--offset" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--offset 뒤에 번호가 필요합니다.");
                }
                offset = Some(args[i].clone());
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }

    let section = parse_usize_cli(section, "--section");
    let para = parse_usize_cli(para, "--para");
    let offset = parse_usize_cli(offset, "--offset");
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let cell_location =
        parse_table_cell_location_cli(&cell_path, table_ctrl, row, col, cell_para.clone());
    let mut result = if let Some((table_ctrl, row, col, cell_para)) = cell_location {
        if cell.is_some() || textbox {
            exit_cli_error(
                "--table-ctrl/--row/--col은 --cell 또는 --textbox와 함께 사용할 수 없습니다.",
            );
        }
        let ctrl = parse_usize_cli(ctrl, "--ctrl");
        let textbox_para = parse_usize_cli(
            nested_para.or_else(|| Some("0".to_string())),
            "--textbox-para",
        );
        get_hwp_cell_shape_field_info_at_json_for_cli(
            &data,
            section,
            para,
            table_ctrl,
            row,
            col,
            cell_para,
            ctrl,
            textbox_para,
            offset,
        )
        .unwrap_or_else(|e| exit_cli_error(&e))
    } else if let Some(cell_path) = cell_path {
        if cell.is_some() || textbox {
            exit_cli_error("--cell-path는 --cell 또는 --textbox와 함께 사용할 수 없습니다.");
        }
        let ctrl = parse_usize_cli(ctrl, "--ctrl");
        let textbox_para = parse_usize_cli(
            nested_para.or_else(|| Some("0".to_string())),
            "--textbox-para",
        );
        get_hwp_field_info_by_path_json_for_cli(
            &data,
            section,
            para,
            &cell_path,
            ctrl,
            textbox_para,
            offset,
        )
        .unwrap_or_else(|e| exit_cli_error(&e))
    } else if ctrl.is_some() || cell.is_some() || nested_para.is_some() || textbox {
        let ctrl = parse_usize_cli(ctrl, "--ctrl");
        let cell_idx = if textbox {
            if cell.is_some() {
                exit_cli_error("--textbox와 --cell은 함께 사용할 수 없습니다.");
            }
            0
        } else {
            parse_usize_cli(cell, "--cell")
        };
        let cell_para = parse_usize_cli(
            nested_para.or(cell_para).or_else(|| Some("0".to_string())),
            if textbox {
                "--textbox-para"
            } else {
                "--cell-para"
            },
        );
        get_hwp_nested_field_info_json_for_cli(
            &data, section, para, ctrl, cell_idx, cell_para, offset, textbox,
        )
        .unwrap_or_else(|e| exit_cli_error(&e))
    } else {
        get_hwp_field_info_json_for_cli(&data, section, para, offset)
            .unwrap_or_else(|e| exit_cli_error(&e))
    };
    if let serde_json::Value::Object(ref mut obj) = result {
        obj.insert("path".to_string(), serde_json::Value::String(input));
    }
    println!("{}", result);
}

fn remove_field_cli(args: &[String]) {
    if args.is_empty() {
        exit_cli_error(
            "사용법: rhwp remove-field <파일.hwp> --section N --para N [--cell-path JSON|--table-ctrl N --row R --col C [--cell-para N]] [--ctrl N --textbox-para N] --offset N -o <출력.hwp>",
        );
    }
    let input = args[0].clone();
    let mut section: Option<String> = None;
    let mut para: Option<String> = None;
    let mut ctrl: Option<String> = None;
    let mut cell: Option<String> = None;
    let mut cell_path: Option<String> = None;
    let mut table_ctrl: Option<String> = None;
    let mut row: Option<String> = None;
    let mut col: Option<String> = None;
    let mut cell_para: Option<String> = None;
    let mut nested_para: Option<String> = None;
    let mut textbox = false;
    let mut offset: Option<String> = None;
    let mut output_path: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--section" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--section 뒤에 번호가 필요합니다.");
                }
                section = Some(args[i].clone());
            }
            "--para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--para 뒤에 번호가 필요합니다.");
                }
                para = Some(args[i].clone());
            }
            "--ctrl" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--ctrl 뒤에 번호가 필요합니다.");
                }
                ctrl = Some(args[i].clone());
            }
            "--cell" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--cell 뒤에 번호가 필요합니다.");
                }
                cell = Some(args[i].clone());
            }
            "--cell-path" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--cell-path 뒤에 JSON 문자열이 필요합니다.");
                }
                cell_path = Some(args[i].clone());
            }
            "--cell-path-file" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--cell-path-file 뒤에 경로가 필요합니다.");
                }
                cell_path = Some(fs::read_to_string(&args[i]).unwrap_or_else(|e| {
                    exit_cli_error(&format!("cellPath 파일 읽기 실패: {}", e))
                }));
            }
            "--table-ctrl" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--table-ctrl 뒤에 값이 필요합니다.");
                }
                table_ctrl = Some(args[i].clone());
            }
            "--row" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--row 뒤에 값이 필요합니다.");
                }
                row = Some(args[i].clone());
            }
            "--col" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--col 뒤에 값이 필요합니다.");
                }
                col = Some(args[i].clone());
            }
            "--cell-para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--cell-para 뒤에 값이 필요합니다.");
                }
                cell_para = Some(args[i].clone());
            }
            "--textbox-para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--textbox-para 뒤에 번호가 필요합니다.");
                }
                nested_para = Some(args[i].clone());
            }
            "--textbox" => {
                textbox = true;
            }
            "--offset" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--offset 뒤에 번호가 필요합니다.");
                }
                offset = Some(args[i].clone());
            }
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("-o/--output 뒤에 경로가 필요합니다.");
                }
                output_path = Some(args[i].clone());
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }

    let section = parse_usize_cli(section, "--section");
    let para = parse_usize_cli(para, "--para");
    let offset = parse_usize_cli(offset, "--offset");
    let output = output_path.unwrap_or_else(|| input.clone());
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let cell_location =
        parse_table_cell_location_cli(&cell_path, table_ctrl, row, col, cell_para.clone());
    if let Some((table_ctrl, row, col, cell_para)) = cell_location {
        if cell.is_some() || textbox {
            exit_cli_error(
                "--table-ctrl/--row/--col은 --cell 또는 --textbox와 함께 사용할 수 없습니다.",
            );
        }
        let ctrl = parse_usize_cli(ctrl, "--ctrl");
        let textbox_para = parse_usize_cli(
            nested_para.or_else(|| Some("0".to_string())),
            "--textbox-para",
        );
        let result = remove_hwp_cell_shape_field_at_bytes_for_cli(
            &data,
            section,
            para,
            table_ctrl,
            row,
            col,
            cell_para,
            ctrl,
            textbox_para,
            offset,
        )
        .unwrap_or_else(|e| exit_cli_error(&e));
        write_hwp_cli_output(&output, &result.bytes).unwrap_or_else(|e| exit_cli_error(&e));
        print_hwp_edit_cli_result(output, result);
    } else if let Some(cell_path) = cell_path {
        if cell.is_some() || textbox {
            exit_cli_error("--cell-path는 --cell 또는 --textbox와 함께 사용할 수 없습니다.");
        }
        let ctrl = parse_usize_cli(ctrl, "--ctrl");
        let textbox_para = parse_usize_cli(
            nested_para.or_else(|| Some("0".to_string())),
            "--textbox-para",
        );
        let result = remove_hwp_field_by_path_bytes_for_cli(
            &data,
            section,
            para,
            &cell_path,
            ctrl,
            textbox_para,
            offset,
        )
        .unwrap_or_else(|e| exit_cli_error(&e));
        write_hwp_cli_output(&output, &result.bytes).unwrap_or_else(|e| exit_cli_error(&e));
        print_hwp_edit_cli_result(output, result);
    } else if ctrl.is_some() || cell.is_some() || nested_para.is_some() || textbox {
        let ctrl = parse_usize_cli(ctrl, "--ctrl");
        let cell_idx = if textbox {
            if cell.is_some() {
                exit_cli_error("--textbox와 --cell은 함께 사용할 수 없습니다.");
            }
            0
        } else {
            parse_usize_cli(cell, "--cell")
        };
        let cell_para = parse_usize_cli(
            nested_para.or(cell_para).or_else(|| Some("0".to_string())),
            if textbox {
                "--textbox-para"
            } else {
                "--cell-para"
            },
        );
        let result = remove_hwp_nested_field_bytes_for_cli(
            &data, section, para, ctrl, cell_idx, cell_para, offset, textbox,
        )
        .unwrap_or_else(|e| exit_cli_error(&e));
        write_hwp_cli_output(&output, &result.bytes).unwrap_or_else(|e| exit_cli_error(&e));
        print_hwp_edit_cli_result(output, result);
    } else {
        let result = remove_hwp_field_bytes_for_cli(&data, section, para, offset)
            .unwrap_or_else(|e| exit_cli_error(&e));
        write_hwp_cli_output(&output, &result.bytes).unwrap_or_else(|e| exit_cli_error(&e));
        println!(
            "{}",
            serde_json::json!({
                "ok": true,
                "path": output,
                "bytes": result.bytes.len(),
                "details": result.details,
                "pageCountBefore": result.page_count_before,
                "pageCountAfter": result.page_count_after,
            })
        );
    }
}

fn set_field_cli(args: &[String]) {
    if args.is_empty() {
        exit_cli_error(
            "사용법: rhwp set-field <파일.hwp> --name <필드명> --value <값> -o <출력.hwp>",
        );
    }
    let input = args[0].clone();
    let mut name: Option<String> = None;
    let mut value: Option<String> = None;
    let mut output_path: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--name" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--name 뒤에 필드명이 필요합니다.");
                }
                name = Some(args[i].clone());
            }
            "--value" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--value 뒤에 값이 필요합니다.");
                }
                value = Some(args[i].clone());
            }
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("-o/--output 뒤에 경로가 필요합니다.");
                }
                output_path = Some(args[i].clone());
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }

    let name = name.unwrap_or_else(|| exit_cli_error("--name <필드명>이 필요합니다."));
    let value = value.unwrap_or_else(|| exit_cli_error("--value <값>이 필요합니다."));
    let output = output_path.unwrap_or_else(|| input.clone());
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let result =
        set_hwp_field_bytes_for_cli(&data, &name, &value).unwrap_or_else(|e| exit_cli_error(&e));
    write_hwp_cli_output(&output, &result.bytes).unwrap_or_else(|e| exit_cli_error(&e));
    println!(
        "{}",
        serde_json::json!({
            "ok": true,
            "path": output,
            "bytes": result.bytes.len(),
            "field": name,
            "value": value,
            "details": result.details,
            "pageCountBefore": result.page_count_before,
            "pageCountAfter": result.page_count_after,
        })
    );
}

fn create_form_cli(args: &[String]) {
    if args.is_empty() {
        exit_cli_error("사용법: rhwp create-form <파일.hwp> --section N --para N [--cell-path JSON|--table-ctrl N --row R --col C [--cell-para N]] --offset N --form-type checkbox|radio|edit|button|combo --name <이름> [--caption <캡션>] [--text <텍스트>] [--value N] [--width N] [--height N] -o <출력.hwp>");
    }
    let input = args[0].clone();
    let mut section: Option<String> = Some("0".to_string());
    let mut para: Option<String> = None;
    let mut offset: Option<String> = None;
    let mut form_type: Option<String> = None;
    let mut name: Option<String> = None;
    let mut cell_path: Option<String> = None;
    let mut table_ctrl: Option<String> = None;
    let mut row: Option<String> = None;
    let mut col: Option<String> = None;
    let mut cell_para: Option<String> = None;
    let mut caption: Option<String> = None;
    let mut text: Option<String> = None;
    let mut value: Option<String> = Some("0".to_string());
    let mut width: Option<String> = Some("1200".to_string());
    let mut height: Option<String> = Some("900".to_string());
    let mut enabled = true;
    let mut properties_json: Option<String> = Some("{}".to_string());
    let mut output_path: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--section" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--section 뒤에 번호가 필요합니다.");
                }
                section = Some(args[i].clone());
            }
            "--para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--para 뒤에 번호가 필요합니다.");
                }
                para = Some(args[i].clone());
            }
            "--offset" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--offset 뒤에 번호가 필요합니다.");
                }
                offset = Some(args[i].clone());
            }
            "--form-type" | "--type" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--form-type 뒤에 값이 필요합니다.");
                }
                form_type = Some(args[i].clone());
            }
            "--name" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--name 뒤에 이름이 필요합니다.");
                }
                name = Some(args[i].clone());
            }
            "--cell-path" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--cell-path 뒤에 JSON 문자열이 필요합니다.");
                }
                cell_path = Some(args[i].clone());
            }
            "--cell-path-file" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--cell-path-file 뒤에 경로가 필요합니다.");
                }
                cell_path = Some(fs::read_to_string(&args[i]).unwrap_or_else(|e| {
                    exit_cli_error(&format!("cell-path 파일 읽기 실패: {}", e))
                }));
            }
            "--table-ctrl" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--table-ctrl 뒤에 값이 필요합니다.");
                }
                table_ctrl = Some(args[i].clone());
            }
            "--row" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--row 뒤에 값이 필요합니다.");
                }
                row = Some(args[i].clone());
            }
            "--col" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--col 뒤에 값이 필요합니다.");
                }
                col = Some(args[i].clone());
            }
            "--cell-para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--cell-para 뒤에 값이 필요합니다.");
                }
                cell_para = Some(args[i].clone());
            }
            "--caption" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--caption 뒤에 값이 필요합니다.");
                }
                caption = Some(args[i].clone());
            }
            "--text" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--text 뒤에 값이 필요합니다.");
                }
                text = Some(args[i].clone());
            }
            "--value" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--value 뒤에 값이 필요합니다.");
                }
                value = Some(args[i].clone());
            }
            "--width" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--width 뒤에 값이 필요합니다.");
                }
                width = Some(args[i].clone());
            }
            "--height" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--height 뒤에 값이 필요합니다.");
                }
                height = Some(args[i].clone());
            }
            "--disabled" => enabled = false,
            "--enabled" => enabled = true,
            "--properties-json" | "--json" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--properties-json 뒤에 JSON 값이 필요합니다.");
                }
                properties_json = Some(args[i].clone());
            }
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("-o/--output 뒤에 경로가 필요합니다.");
                }
                output_path = Some(args[i].clone());
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }

    let section = parse_usize_cli(section, "--section");
    let para = parse_usize_cli(para, "--para");
    let offset = parse_usize_cli(offset, "--offset");
    let form_type = form_type.unwrap_or_else(|| exit_cli_error("--form-type 값이 필요합니다."));
    let name = name.unwrap_or_else(|| exit_cli_error("--name 값이 필요합니다."));
    let caption = caption.unwrap_or_default();
    let text = text.unwrap_or_default();
    let value = value
        .unwrap_or_default()
        .parse::<i32>()
        .unwrap_or_else(|_| exit_cli_error("--value 값이 정수가 아닙니다."));
    let width = parse_u32_cli(width, "--width");
    let height = parse_u32_cli(height, "--height");
    let properties_json = properties_json.unwrap_or_else(|| "{}".to_string());
    let cell_location = parse_table_cell_location_cli(&cell_path, table_ctrl, row, col, cell_para);
    let output = output_path.unwrap_or_else(|| input.clone());
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let result = if let Some((table_ctrl, row, col, cell_para)) = cell_location {
        create_hwp_cell_form_object_at_bytes_for_cli(
            &data,
            section,
            para,
            table_ctrl,
            row,
            col,
            cell_para,
            offset,
            &form_type,
            &name,
            &caption,
            &text,
            width,
            height,
            value,
            enabled,
            &properties_json,
        )
    } else if let Some(cell_path) = cell_path {
        create_hwp_cell_form_object_bytes_for_cli(
            &data,
            section,
            para,
            &cell_path,
            offset,
            &form_type,
            &name,
            &caption,
            &text,
            width,
            height,
            value,
            enabled,
            &properties_json,
        )
    } else {
        create_hwp_form_object_bytes_for_cli(
            &data,
            section,
            para,
            offset,
            &form_type,
            &name,
            &caption,
            &text,
            width,
            height,
            value,
            enabled,
            &properties_json,
        )
    }
    .unwrap_or_else(|e| exit_cli_error(&e));
    write_hwp_cli_output(&output, &result.bytes).unwrap_or_else(|e| exit_cli_error(&e));
    println!(
        "{}",
        serde_json::json!({
            "ok": true,
            "path": output,
            "bytes": result.bytes.len(),
            "paraIdx": result.para_idx,
            "controlIdx": result.control_idx,
            "details": result.details,
            "pageCountBefore": result.page_count_before,
            "pageCountAfter": result.page_count_after,
        })
    );
}

fn get_form_cli(args: &[String]) {
    if args.is_empty() {
        exit_cli_error("사용법: rhwp get-form <파일.hwp> --section N --para N [--cell-path JSON|--table-ctrl N --row R --col C [--cell-para N]] --ctrl N");
    }
    let input = args[0].clone();
    let mut section: Option<String> = Some("0".to_string());
    let mut para: Option<String> = None;
    let mut ctrl: Option<String> = None;
    let mut cell_path: Option<String> = None;
    let mut table_ctrl: Option<String> = None;
    let mut row: Option<String> = None;
    let mut col: Option<String> = None;
    let mut cell_para: Option<String> = None;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--section" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--section 뒤에 번호가 필요합니다.");
                }
                section = Some(args[i].clone());
            }
            "--para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--para 뒤에 번호가 필요합니다.");
                }
                para = Some(args[i].clone());
            }
            "--ctrl" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--ctrl 뒤에 번호가 필요합니다.");
                }
                ctrl = Some(args[i].clone());
            }
            "--cell-path" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--cell-path 뒤에 JSON 문자열이 필요합니다.");
                }
                cell_path = Some(args[i].clone());
            }
            "--cell-path-file" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--cell-path-file 뒤에 경로가 필요합니다.");
                }
                cell_path = Some(fs::read_to_string(&args[i]).unwrap_or_else(|e| {
                    exit_cli_error(&format!("cell-path 파일 읽기 실패: {}", e))
                }));
            }
            "--table-ctrl" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--table-ctrl 뒤에 값이 필요합니다.");
                }
                table_ctrl = Some(args[i].clone());
            }
            "--row" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--row 뒤에 값이 필요합니다.");
                }
                row = Some(args[i].clone());
            }
            "--col" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--col 뒤에 값이 필요합니다.");
                }
                col = Some(args[i].clone());
            }
            "--cell-para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--cell-para 뒤에 값이 필요합니다.");
                }
                cell_para = Some(args[i].clone());
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }
    let section = parse_usize_cli(section, "--section");
    let para = parse_usize_cli(para, "--para");
    let ctrl = parse_usize_cli(ctrl, "--ctrl");
    let cell_location = parse_table_cell_location_cli(&cell_path, table_ctrl, row, col, cell_para);
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let mut result = if let Some((table_ctrl, row, col, cell_para)) = cell_location {
        get_hwp_cell_form_info_at_json_for_cli(
            &data, section, para, table_ctrl, row, col, cell_para, ctrl,
        )
    } else if let Some(cell_path) = cell_path {
        get_hwp_cell_form_info_json_for_cli(&data, section, para, &cell_path, ctrl)
    } else {
        get_hwp_form_info_json_for_cli(&data, section, para, ctrl)
    }
    .unwrap_or_else(|e| exit_cli_error(&e));
    if let serde_json::Value::Object(ref mut obj) = result {
        obj.insert("path".to_string(), serde_json::Value::String(input));
    }
    println!("{}", result);
}

fn set_form_cli(args: &[String]) {
    if args.is_empty() {
        exit_cli_error("사용법: rhwp set-form <파일.hwp> --section N --para N [--cell-path JSON|--table-ctrl N --row R --col C [--cell-para N]] --ctrl N --json <값JSON> -o <출력.hwp>");
    }
    let input = args[0].clone();
    let mut section: Option<String> = Some("0".to_string());
    let mut para: Option<String> = None;
    let mut ctrl: Option<String> = None;
    let mut cell_path: Option<String> = None;
    let mut table_ctrl: Option<String> = None;
    let mut row: Option<String> = None;
    let mut col: Option<String> = None;
    let mut cell_para: Option<String> = None;
    let mut json_value: Option<String> = None;
    let mut output_path: Option<String> = None;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--section" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--section 뒤에 번호가 필요합니다.");
                }
                section = Some(args[i].clone());
            }
            "--para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--para 뒤에 번호가 필요합니다.");
                }
                para = Some(args[i].clone());
            }
            "--ctrl" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--ctrl 뒤에 번호가 필요합니다.");
                }
                ctrl = Some(args[i].clone());
            }
            "--cell-path" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--cell-path 뒤에 JSON 문자열이 필요합니다.");
                }
                cell_path = Some(args[i].clone());
            }
            "--cell-path-file" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--cell-path-file 뒤에 경로가 필요합니다.");
                }
                cell_path = Some(fs::read_to_string(&args[i]).unwrap_or_else(|e| {
                    exit_cli_error(&format!("cell-path 파일 읽기 실패: {}", e))
                }));
            }
            "--table-ctrl" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--table-ctrl 뒤에 값이 필요합니다.");
                }
                table_ctrl = Some(args[i].clone());
            }
            "--row" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--row 뒤에 값이 필요합니다.");
                }
                row = Some(args[i].clone());
            }
            "--col" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--col 뒤에 값이 필요합니다.");
                }
                col = Some(args[i].clone());
            }
            "--cell-para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--cell-para 뒤에 값이 필요합니다.");
                }
                cell_para = Some(args[i].clone());
            }
            "--json" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--json 뒤에 값이 필요합니다.");
                }
                json_value = Some(args[i].clone());
            }
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("-o/--output 뒤에 경로가 필요합니다.");
                }
                output_path = Some(args[i].clone());
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }
    let section = parse_usize_cli(section, "--section");
    let para = parse_usize_cli(para, "--para");
    let ctrl = parse_usize_cli(ctrl, "--ctrl");
    let json_value = json_value.unwrap_or_else(|| exit_cli_error("--json 값이 필요합니다."));
    let cell_location = parse_table_cell_location_cli(&cell_path, table_ctrl, row, col, cell_para);
    let output = output_path.unwrap_or_else(|| input.clone());
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let result = if let Some((table_ctrl, row, col, cell_para)) = cell_location {
        set_hwp_cell_form_value_at_bytes_for_cli(
            &data,
            section,
            para,
            table_ctrl,
            row,
            col,
            cell_para,
            ctrl,
            &json_value,
        )
    } else if let Some(cell_path) = cell_path {
        set_hwp_cell_form_value_bytes_for_cli(&data, section, para, &cell_path, ctrl, &json_value)
    } else {
        set_hwp_form_value_bytes_for_cli(&data, section, para, ctrl, &json_value)
    }
    .unwrap_or_else(|e| exit_cli_error(&e));
    write_hwp_cli_output(&output, &result.bytes).unwrap_or_else(|e| exit_cli_error(&e));
    print_hwp_edit_cli_result(output, result);
}

fn delete_form_cli(args: &[String]) {
    if args.is_empty() {
        exit_cli_error("사용법: rhwp delete-form <파일.hwp> --section N --para N [--cell-path JSON|--table-ctrl N --row R --col C [--cell-para N]] --ctrl N -o <출력.hwp>");
    }
    let input = args[0].clone();
    let mut section: Option<String> = Some("0".to_string());
    let mut para: Option<String> = None;
    let mut ctrl: Option<String> = None;
    let mut cell_path: Option<String> = None;
    let mut table_ctrl: Option<String> = None;
    let mut row: Option<String> = None;
    let mut col: Option<String> = None;
    let mut cell_para: Option<String> = None;
    let mut output_path: Option<String> = None;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--section" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--section 뒤에 번호가 필요합니다.");
                }
                section = Some(args[i].clone());
            }
            "--para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--para 뒤에 번호가 필요합니다.");
                }
                para = Some(args[i].clone());
            }
            "--ctrl" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--ctrl 뒤에 번호가 필요합니다.");
                }
                ctrl = Some(args[i].clone());
            }
            "--cell-path" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--cell-path 뒤에 JSON 문자열이 필요합니다.");
                }
                cell_path = Some(args[i].clone());
            }
            "--cell-path-file" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--cell-path-file 뒤에 경로가 필요합니다.");
                }
                cell_path = Some(fs::read_to_string(&args[i]).unwrap_or_else(|e| {
                    exit_cli_error(&format!("cell-path 파일 읽기 실패: {}", e))
                }));
            }
            "--table-ctrl" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--table-ctrl 뒤에 값이 필요합니다.");
                }
                table_ctrl = Some(args[i].clone());
            }
            "--row" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--row 뒤에 값이 필요합니다.");
                }
                row = Some(args[i].clone());
            }
            "--col" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--col 뒤에 값이 필요합니다.");
                }
                col = Some(args[i].clone());
            }
            "--cell-para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--cell-para 뒤에 값이 필요합니다.");
                }
                cell_para = Some(args[i].clone());
            }
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("-o/--output 뒤에 경로가 필요합니다.");
                }
                output_path = Some(args[i].clone());
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }
    let section = parse_usize_cli(section, "--section");
    let para = parse_usize_cli(para, "--para");
    let ctrl = parse_usize_cli(ctrl, "--ctrl");
    let cell_location = parse_table_cell_location_cli(&cell_path, table_ctrl, row, col, cell_para);
    let output = output_path.unwrap_or_else(|| input.clone());
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let result = if let Some((table_ctrl, row, col, cell_para)) = cell_location {
        delete_hwp_cell_form_object_at_bytes_for_cli(
            &data, section, para, table_ctrl, row, col, cell_para, ctrl,
        )
    } else if let Some(cell_path) = cell_path {
        delete_hwp_cell_form_object_bytes_for_cli(&data, section, para, &cell_path, ctrl)
    } else {
        delete_hwp_form_object_bytes_for_cli(&data, section, para, ctrl)
    }
    .unwrap_or_else(|e| exit_cli_error(&e));
    write_hwp_cli_output(&output, &result.bytes).unwrap_or_else(|e| exit_cli_error(&e));
    print_hwp_edit_cli_result(output, result);
}

fn parse_usize_cli(value: Option<String>, name: &str) -> usize {
    value
        .unwrap_or_else(|| exit_cli_error(&format!("{} 값이 필요합니다.", name)))
        .parse::<usize>()
        .unwrap_or_else(|_| exit_cli_error(&format!("{} 값이 정수가 아닙니다.", name)))
}

fn parse_u16_cli(value: Option<String>, name: &str) -> u16 {
    value
        .unwrap_or_else(|| exit_cli_error(&format!("{} 값이 필요합니다.", name)))
        .parse::<u16>()
        .unwrap_or_else(|_| exit_cli_error(&format!("{} 값이 0~65535 정수가 아닙니다.", name)))
}

fn parse_table_cell_location_cli(
    cell_path: &Option<String>,
    table_ctrl: Option<String>,
    row: Option<String>,
    col: Option<String>,
    cell_para: Option<String>,
) -> Option<(usize, u16, u16, usize)> {
    let row_col = match (row, col) {
        (Some(row), Some(col)) => Some((
            parse_u16_cli(Some(row), "--row"),
            parse_u16_cli(Some(col), "--col"),
        )),
        (None, None) => None,
        _ => exit_cli_error("--row와 --col은 함께 지정해야 합니다."),
    };
    if cell_path.is_some() && row_col.is_some() {
        exit_cli_error("--cell-path와 --row/--col은 함께 사용할 수 없습니다.");
    }
    if let Some((row, col)) = row_col {
        let table_ctrl = parse_usize_cli(table_ctrl, "--table-ctrl");
        let cell_para = cell_para
            .map(|value| parse_usize_cli(Some(value), "--cell-para"))
            .unwrap_or(0);
        return Some((table_ctrl, row, col, cell_para));
    }
    if table_ctrl.is_some() || cell_para.is_some() {
        exit_cli_error(
            "--table-ctrl/--cell-para는 --row와 --col을 함께 지정할 때만 사용할 수 있습니다.",
        );
    }
    None
}

fn parse_u32_cli(value: Option<String>, name: &str) -> u32 {
    value
        .unwrap_or_else(|| exit_cli_error(&format!("{} 값이 필요합니다.", name)))
        .parse::<u32>()
        .unwrap_or_else(|_| exit_cli_error(&format!("{} 값이 0~4294967295 정수가 아닙니다.", name)))
}

fn parse_i32_cli(value: Option<String>, name: &str) -> i32 {
    value
        .unwrap_or_else(|| exit_cli_error(&format!("{} 값이 필요합니다.", name)))
        .parse::<i32>()
        .unwrap_or_else(|_| exit_cli_error(&format!("{} 값이 정수가 아닙니다.", name)))
}

fn extract_structure_cli(args: &[String]) {
    if args.len() != 1 {
        exit_cli_error("사용법: rhwp extract-structure <파일.hwp>");
    }
    let input = &args[0];
    let data = fs::read(input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let mut result =
        extract_hwp_structure_json_for_cli(&data).unwrap_or_else(|e| exit_cli_error(&e));
    if let serde_json::Value::Object(ref mut obj) = result {
        obj.insert("path".to_string(), serde_json::Value::String(input.clone()));
    }
    println!("{}", result);
}

fn text_edit_cli(args: &[String], delete: bool) {
    if args.is_empty() {
        if delete {
            exit_cli_error("사용법: rhwp delete-text <파일.hwp> --section N --para N --offset N --count N -o <출력.hwp>");
        } else {
            exit_cli_error("사용법: rhwp insert-text <파일.hwp> --section N --para N --offset N --text <텍스트> -o <출력.hwp>");
        }
    }
    let input = args[0].clone();
    let mut section: Option<String> = None;
    let mut para: Option<String> = None;
    let mut offset: Option<String> = None;
    let mut count: Option<String> = None;
    let mut inline_text: Option<String> = None;
    let mut text_file: Option<String> = None;
    let mut output_path: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--section" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--section 뒤에 값이 필요합니다.");
                }
                section = Some(args[i].clone());
            }
            "--para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--para 뒤에 값이 필요합니다.");
                }
                para = Some(args[i].clone());
            }
            "--offset" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--offset 뒤에 값이 필요합니다.");
                }
                offset = Some(args[i].clone());
            }
            "--count" if delete => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--count 뒤에 값이 필요합니다.");
                }
                count = Some(args[i].clone());
            }
            "--text" if !delete => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--text 뒤에 텍스트가 필요합니다.");
                }
                inline_text = Some(args[i].clone());
            }
            "--text-file" if !delete => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--text-file 뒤에 경로가 필요합니다.");
                }
                text_file = Some(args[i].clone());
            }
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("-o/--output 뒤에 경로가 필요합니다.");
                }
                output_path = Some(args[i].clone());
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }

    let section = parse_usize_cli(section, "--section");
    let para = parse_usize_cli(para, "--para");
    let offset = parse_usize_cli(offset, "--offset");
    let output = output_path.unwrap_or_else(|| input.clone());
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let result = if delete {
        let count = parse_usize_cli(count, "--count");
        delete_hwp_text_bytes_for_cli(&data, section, para, offset, count)
    } else {
        let text = read_optional_text_argument(inline_text, text_file)
            .unwrap_or_else(|e| exit_cli_error(&e))
            .unwrap_or_else(|| exit_cli_error("--text 또는 --text-file 값이 필요합니다."));
        insert_hwp_text_bytes_for_cli(&data, section, para, offset, &text)
    }
    .unwrap_or_else(|e| exit_cli_error(&e));
    write_hwp_cli_output(&output, &result.bytes).unwrap_or_else(|e| exit_cli_error(&e));
    print_hwp_edit_cli_result(output, result);
}

fn set_paragraph_cli(args: &[String]) {
    if args.is_empty() {
        exit_cli_error(
            "사용법: rhwp set-paragraph <파일.hwp> --section N --para N --text <텍스트> -o <출력.hwp>",
        );
    }
    let input = args[0].clone();
    let mut section: Option<String> = None;
    let mut para: Option<String> = None;
    let mut text: Option<String> = None;
    let mut output_path: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--section" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--section 뒤에 값이 필요합니다.");
                }
                section = Some(args[i].clone());
            }
            "--para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--para 뒤에 값이 필요합니다.");
                }
                para = Some(args[i].clone());
            }
            "--text" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--text 뒤에 텍스트가 필요합니다.");
                }
                text = Some(args[i].clone());
            }
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("-o/--output 뒤에 경로가 필요합니다.");
                }
                output_path = Some(args[i].clone());
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }

    let section = parse_usize_cli(section, "--section");
    let para = parse_usize_cli(para, "--para");
    let text = text.unwrap_or_else(|| exit_cli_error("--text <텍스트>가 필요합니다."));
    let output = output_path.unwrap_or_else(|| input.clone());
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let result = set_hwp_paragraph_text_bytes_for_cli(&data, section, para, &text)
        .unwrap_or_else(|e| exit_cli_error(&e));
    write_hwp_cli_output(&output, &result.bytes).unwrap_or_else(|e| exit_cli_error(&e));
    println!(
        "{}",
        serde_json::json!({
            "ok": true,
            "path": output,
            "bytes": result.bytes.len(),
            "details": result.details,
            "pageCountBefore": result.page_count_before,
            "pageCountAfter": result.page_count_after,
        })
    );
}

fn paragraph_insert_cli(args: &[String]) {
    if args.is_empty() {
        exit_cli_error("사용법: rhwp insert-paragraph <파일.hwp> --section N --para N [--text <텍스트>] -o <출력.hwp>");
    }
    let input = args[0].clone();
    let mut section: Option<String> = None;
    let mut para: Option<String> = None;
    let mut inline_text: Option<String> = None;
    let mut text_file: Option<String> = None;
    let mut output_path: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--section" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--section 뒤에 값이 필요합니다.");
                }
                section = Some(args[i].clone());
            }
            "--para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--para 뒤에 값이 필요합니다.");
                }
                para = Some(args[i].clone());
            }
            "--text" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--text 뒤에 텍스트가 필요합니다.");
                }
                inline_text = Some(args[i].clone());
            }
            "--text-file" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--text-file 뒤에 경로가 필요합니다.");
                }
                text_file = Some(args[i].clone());
            }
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("-o/--output 뒤에 경로가 필요합니다.");
                }
                output_path = Some(args[i].clone());
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }

    let section = parse_usize_cli(section, "--section");
    let para = parse_usize_cli(para, "--para");
    let text =
        read_optional_text_argument(inline_text, text_file).unwrap_or_else(|e| exit_cli_error(&e));
    let output = output_path.unwrap_or_else(|| input.clone());
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let result = insert_hwp_paragraph_bytes_for_cli(&data, section, para, text.as_deref())
        .unwrap_or_else(|e| exit_cli_error(&e));
    write_hwp_cli_output(&output, &result.bytes).unwrap_or_else(|e| exit_cli_error(&e));
    print_hwp_edit_cli_result(output, result);
}

fn paragraph_copy_cli(args: &[String]) {
    if args.is_empty() {
        exit_cli_error(
            "사용법: rhwp copy-paragraph <파일.hwp> --section N --para N [--before|--after] -o <출력.hwp>",
        );
    }
    let input = args[0].clone();
    let mut section: Option<String> = None;
    let mut para: Option<String> = None;
    let mut after = true;
    let mut output_path: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--section" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--section 뒤에 값이 필요합니다.");
                }
                section = Some(args[i].clone());
            }
            "--para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--para 뒤에 값이 필요합니다.");
                }
                para = Some(args[i].clone());
            }
            "--before" => after = false,
            "--after" => after = true,
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("-o/--output 뒤에 경로가 필요합니다.");
                }
                output_path = Some(args[i].clone());
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }

    let section = parse_usize_cli(section, "--section");
    let para = parse_usize_cli(para, "--para");
    let output = output_path.unwrap_or_else(|| input.clone());
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let result = copy_hwp_paragraph_bytes_for_cli(&data, section, para, after)
        .unwrap_or_else(|e| exit_cli_error(&e));
    write_hwp_cli_output(&output, &result.bytes).unwrap_or_else(|e| exit_cli_error(&e));
    print_hwp_edit_cli_result(output, result);
}

fn paragraph_range_copy_cli(args: &[String]) {
    if args.is_empty() {
        exit_cli_error(
            "사용법: rhwp copy-paragraph-range <파일.hwp> --section N --start N --end N [--before|--after] [--replace OLD NEW]... -o <출력.hwp>",
        );
    }
    let input = args[0].clone();
    let mut section: Option<String> = None;
    let mut start: Option<String> = None;
    let mut end: Option<String> = None;
    let mut after = true;
    let mut replacements: Vec<(String, String)> = Vec::new();
    let mut output_path: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--section" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--section 뒤에 값이 필요합니다.");
                }
                section = Some(args[i].clone());
            }
            "--start" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--start 뒤에 값이 필요합니다.");
                }
                start = Some(args[i].clone());
            }
            "--end" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--end 뒤에 값이 필요합니다.");
                }
                end = Some(args[i].clone());
            }
            "--before" => after = false,
            "--after" => after = true,
            "--replace" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--replace 뒤에 검색어가 필요합니다.");
                }
                let old = args[i].clone();
                if old.is_empty() {
                    exit_cli_error("--replace 검색어는 비어 있을 수 없습니다.");
                }
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--replace 뒤에 대체문구가 필요합니다.");
                }
                replacements.push((old, args[i].clone()));
            }
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("-o/--output 뒤에 경로가 필요합니다.");
                }
                output_path = Some(args[i].clone());
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }

    let section = parse_usize_cli(section, "--section");
    let start = parse_usize_cli(start, "--start");
    let end = parse_usize_cli(end, "--end");
    let output = output_path.unwrap_or_else(|| input.clone());
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let result = if replacements.is_empty() {
        copy_hwp_paragraph_range_bytes_for_cli(&data, section, start, end, after)
    } else {
        copy_hwp_paragraph_range_with_replacements_bytes_for_cli(
            &data,
            section,
            start,
            end,
            after,
            &replacements,
        )
    }
    .unwrap_or_else(|e| exit_cli_error(&e));
    write_hwp_cli_output(&output, &result.bytes).unwrap_or_else(|e| exit_cli_error(&e));
    print_hwp_edit_cli_result(output, result);
}

fn paragraph_split_cli(args: &[String]) {
    if args.is_empty() {
        exit_cli_error(
            "사용법: rhwp split-paragraph <파일.hwp> --section N --para N --offset N -o <출력.hwp>",
        );
    }
    let input = args[0].clone();
    let mut section: Option<String> = None;
    let mut para: Option<String> = None;
    let mut offset: Option<String> = None;
    let mut output_path: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--section" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--section 뒤에 값이 필요합니다.");
                }
                section = Some(args[i].clone());
            }
            "--para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--para 뒤에 값이 필요합니다.");
                }
                para = Some(args[i].clone());
            }
            "--offset" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--offset 뒤에 값이 필요합니다.");
                }
                offset = Some(args[i].clone());
            }
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("-o/--output 뒤에 경로가 필요합니다.");
                }
                output_path = Some(args[i].clone());
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }

    let section = parse_usize_cli(section, "--section");
    let para = parse_usize_cli(para, "--para");
    let offset = parse_usize_cli(offset, "--offset");
    let output = output_path.unwrap_or_else(|| input.clone());
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let result = split_hwp_paragraph_bytes_for_cli(&data, section, para, offset)
        .unwrap_or_else(|e| exit_cli_error(&e));
    write_hwp_cli_output(&output, &result.bytes).unwrap_or_else(|e| exit_cli_error(&e));
    print_hwp_edit_cli_result(output, result);
}

fn paragraph_merge_delete_cli(args: &[String], action: &str) {
    if args.is_empty() {
        exit_cli_error(
            "사용법: rhwp merge|delete-paragraph <파일.hwp> --section N --para N -o <출력.hwp>",
        );
    }
    let input = args[0].clone();
    let mut section: Option<String> = None;
    let mut para: Option<String> = None;
    let mut output_path: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--section" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--section 뒤에 값이 필요합니다.");
                }
                section = Some(args[i].clone());
            }
            "--para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--para 뒤에 값이 필요합니다.");
                }
                para = Some(args[i].clone());
            }
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("-o/--output 뒤에 경로가 필요합니다.");
                }
                output_path = Some(args[i].clone());
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }

    let section = parse_usize_cli(section, "--section");
    let para = parse_usize_cli(para, "--para");
    let output = output_path.unwrap_or_else(|| input.clone());
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let result = match action {
        "merge" => merge_hwp_paragraph_bytes_for_cli(&data, section, para),
        "delete" => delete_hwp_paragraph_bytes_for_cli(&data, section, para),
        _ => Err(format!("지원하지 않는 문단 구조 작업: {}", action)),
    }
    .unwrap_or_else(|e| exit_cli_error(&e));
    write_hwp_cli_output(&output, &result.bytes).unwrap_or_else(|e| exit_cli_error(&e));
    print_hwp_edit_cli_result(output, result);
}

fn layout_break_cli(args: &[String], kind: &str) {
    if args.is_empty() {
        exit_cli_error("사용법: rhwp insert-page|column-break <파일.hwp> --section N --para N --offset N -o <출력.hwp>");
    }
    let input = args[0].clone();
    let mut section: Option<String> = None;
    let mut para: Option<String> = None;
    let mut offset: Option<String> = None;
    let mut output_path: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--section" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--section 뒤에 값이 필요합니다.");
                }
                section = Some(args[i].clone());
            }
            "--para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--para 뒤에 값이 필요합니다.");
                }
                para = Some(args[i].clone());
            }
            "--offset" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--offset 뒤에 값이 필요합니다.");
                }
                offset = Some(args[i].clone());
            }
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("-o/--output 뒤에 경로가 필요합니다.");
                }
                output_path = Some(args[i].clone());
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }

    let section = parse_usize_cli(section, "--section");
    let para = parse_usize_cli(para, "--para");
    let offset = parse_usize_cli(offset, "--offset");
    let output = output_path.unwrap_or_else(|| input.clone());
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let result = match kind {
        "page" => insert_hwp_page_break_bytes_for_cli(&data, section, para, offset),
        "column" => insert_hwp_column_break_bytes_for_cli(&data, section, para, offset),
        _ => Err(format!("지원하지 않는 나누기 작업: {}", kind)),
    }
    .unwrap_or_else(|e| exit_cli_error(&e));
    write_hwp_cli_output(&output, &result.bytes).unwrap_or_else(|e| exit_cli_error(&e));
    print_hwp_edit_cli_result(output, result);
}

fn set_column_def_cli(args: &[String]) {
    if args.is_empty() {
        exit_cli_error("사용법: rhwp set-column-def <파일.hwp> --section N --count N [--type normal|distribute|parallel] [--spacing HWPUNIT] [--same-width|--variable-width] -o <출력.hwp>");
    }
    let input = args[0].clone();
    let mut section: Option<String> = None;
    let mut count: Option<String> = None;
    let mut column_type: Option<String> = Some("normal".to_string());
    let mut spacing: Option<String> = Some("0".to_string());
    let mut same_width = true;
    let mut output_path: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--section" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--section 뒤에 값이 필요합니다.");
                }
                section = Some(args[i].clone());
            }
            "--count" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--count 뒤에 값이 필요합니다.");
                }
                count = Some(args[i].clone());
            }
            "--type" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--type 뒤에 normal/distribute/parallel 값이 필요합니다.");
                }
                column_type = Some(args[i].clone());
            }
            "--spacing" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--spacing 뒤에 HWPUNIT 정수 값이 필요합니다.");
                }
                spacing = Some(args[i].clone());
            }
            "--same-width" => same_width = true,
            "--variable-width" => same_width = false,
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("-o/--output 뒤에 경로가 필요합니다.");
                }
                output_path = Some(args[i].clone());
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }

    let section = parse_usize_cli(section, "--section");
    let count = parse_u16_cli(count, "--count");
    let column_type = parse_column_type_for_cli(
        &column_type.unwrap_or_else(|| exit_cli_error("--type 값이 필요합니다.")),
    )
    .unwrap_or_else(|e| exit_cli_error(&e));
    let spacing = parse_i32_cli(spacing, "--spacing");
    if spacing < i16::MIN as i32 || spacing > i16::MAX as i32 {
        exit_cli_error("--spacing 값은 -32768~32767 범위여야 합니다.");
    }
    let output = output_path.unwrap_or_else(|| input.clone());
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let result = set_hwp_column_def_bytes_for_cli(
        &data,
        section,
        count,
        column_type,
        same_width,
        spacing as i16,
    )
    .unwrap_or_else(|e| exit_cli_error(&e));
    write_hwp_cli_output(&output, &result.bytes).unwrap_or_else(|e| exit_cli_error(&e));
    print_hwp_edit_cli_result(output, result);
}

fn new_number_cli(args: &[String]) {
    if args.is_empty() {
        exit_cli_error("사용법: rhwp insert-new-number <파일.hwp> --section N --para N --offset N --start N -o <출력.hwp>");
    }
    let input = args[0].clone();
    let mut section: Option<String> = None;
    let mut para: Option<String> = None;
    let mut offset: Option<String> = None;
    let mut start: Option<String> = None;
    let mut output_path: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--section" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--section 뒤에 값이 필요합니다.");
                }
                section = Some(args[i].clone());
            }
            "--para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--para 뒤에 값이 필요합니다.");
                }
                para = Some(args[i].clone());
            }
            "--offset" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--offset 뒤에 값이 필요합니다.");
                }
                offset = Some(args[i].clone());
            }
            "--start" | "--start-number" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--start 뒤에 값이 필요합니다.");
                }
                start = Some(args[i].clone());
            }
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("-o/--output 뒤에 경로가 필요합니다.");
                }
                output_path = Some(args[i].clone());
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }

    let section = parse_usize_cli(section, "--section");
    let para = parse_usize_cli(para, "--para");
    let offset = parse_usize_cli(offset, "--offset");
    let start = parse_u16_cli(start, "--start");
    if start == 0 {
        exit_cli_error("--start 값은 1 이상이어야 합니다.");
    }
    let output = output_path.unwrap_or_else(|| input.clone());
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let result = insert_hwp_new_number_bytes_for_cli(&data, section, para, offset, start)
        .unwrap_or_else(|e| exit_cli_error(&e));
    write_hwp_cli_output(&output, &result.bytes).unwrap_or_else(|e| exit_cli_error(&e));
    print_hwp_edit_cli_result(output, result);
}

fn get_page_hide_cli(args: &[String]) {
    if args.is_empty() {
        exit_cli_error("사용법: rhwp get-page-hide <파일.hwp> --section N --para N");
    }
    let input = args[0].clone();
    let mut section: Option<String> = None;
    let mut para: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--section" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--section 뒤에 값이 필요합니다.");
                }
                section = Some(args[i].clone());
            }
            "--para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--para 뒤에 값이 필요합니다.");
                }
                para = Some(args[i].clone());
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }

    let section = parse_usize_cli(section, "--section");
    let para = parse_usize_cli(para, "--para");
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let result =
        get_hwp_page_hide_json_for_cli(&data, section, para).unwrap_or_else(|e| exit_cli_error(&e));
    println!("{}", result);
}

fn set_page_hide_cli(args: &[String]) {
    if args.is_empty() {
        exit_cli_error("사용법: rhwp set-page-hide <파일.hwp> --section N --para N [--hide-header] [--hide-footer] [--hide-master-page] [--hide-border] [--hide-fill] [--hide-page-num] -o <출력.hwp>");
    }
    let input = args[0].clone();
    let mut section: Option<String> = None;
    let mut para: Option<String> = None;
    let mut output_path: Option<String> = None;
    let mut hide_header = false;
    let mut hide_footer = false;
    let mut hide_master_page = false;
    let mut hide_border = false;
    let mut hide_fill = false;
    let mut hide_page_num = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--section" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--section 뒤에 값이 필요합니다.");
                }
                section = Some(args[i].clone());
            }
            "--para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--para 뒤에 값이 필요합니다.");
                }
                para = Some(args[i].clone());
            }
            "--hide-header" => hide_header = true,
            "--hide-footer" => hide_footer = true,
            "--hide-master-page" => hide_master_page = true,
            "--hide-border" => hide_border = true,
            "--hide-fill" => hide_fill = true,
            "--hide-page-num" => hide_page_num = true,
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("-o/--output 뒤에 경로가 필요합니다.");
                }
                output_path = Some(args[i].clone());
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }

    let section = parse_usize_cli(section, "--section");
    let para = parse_usize_cli(para, "--para");
    let output = output_path.unwrap_or_else(|| input.clone());
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let result = set_hwp_page_hide_bytes_for_cli(
        &data,
        section,
        para,
        hide_header,
        hide_footer,
        hide_master_page,
        hide_border,
        hide_fill,
        hide_page_num,
    )
    .unwrap_or_else(|e| exit_cli_error(&e));
    write_hwp_cli_output(&output, &result.bytes).unwrap_or_else(|e| exit_cli_error(&e));
    print_hwp_edit_cli_result(output, result);
}

fn list_bookmarks_cli(args: &[String]) {
    if args.is_empty() {
        exit_cli_error("사용법: rhwp list-bookmarks <파일.hwp>");
    }
    let input = args[0].clone();
    if args.len() > 1 {
        exit_cli_error(&format!("알 수 없는 옵션: {}", args[1]));
    }
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let result = get_hwp_bookmarks_json_for_cli(&data).unwrap_or_else(|e| exit_cli_error(&e));
    println!("{}", result);
}

fn add_bookmark_cli(args: &[String]) {
    if args.is_empty() {
        exit_cli_error("사용법: rhwp add-bookmark <파일.hwp> --section N --para N --offset N --name <이름> -o <출력.hwp>");
    }
    let input = args[0].clone();
    let mut section: Option<String> = None;
    let mut para: Option<String> = None;
    let mut offset: Option<String> = None;
    let mut name: Option<String> = None;
    let mut output_path: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--section" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--section 뒤에 값이 필요합니다.");
                }
                section = Some(args[i].clone());
            }
            "--para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--para 뒤에 값이 필요합니다.");
                }
                para = Some(args[i].clone());
            }
            "--offset" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--offset 뒤에 값이 필요합니다.");
                }
                offset = Some(args[i].clone());
            }
            "--name" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--name 뒤에 값이 필요합니다.");
                }
                name = Some(args[i].clone());
            }
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("-o/--output 뒤에 경로가 필요합니다.");
                }
                output_path = Some(args[i].clone());
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }

    let section = parse_usize_cli(section, "--section");
    let para = parse_usize_cli(para, "--para");
    let offset = parse_usize_cli(offset, "--offset");
    let name = name.unwrap_or_else(|| exit_cli_error("--name 값이 필요합니다."));
    let output = output_path.unwrap_or_else(|| input.clone());
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let result = add_hwp_bookmark_bytes_for_cli(&data, section, para, offset, &name)
        .unwrap_or_else(|e| exit_cli_error(&e));
    write_hwp_cli_output(&output, &result.bytes).unwrap_or_else(|e| exit_cli_error(&e));
    print_hwp_edit_cli_result(output, result);
}

fn rename_bookmark_cli(args: &[String]) {
    if args.is_empty() {
        exit_cli_error("사용법: rhwp rename-bookmark <파일.hwp> --section N --para N --ctrl N --name <새이름> -o <출력.hwp>");
    }
    let input = args[0].clone();
    let mut section: Option<String> = None;
    let mut para: Option<String> = None;
    let mut ctrl: Option<String> = None;
    let mut name: Option<String> = None;
    let mut output_path: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--section" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--section 뒤에 값이 필요합니다.");
                }
                section = Some(args[i].clone());
            }
            "--para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--para 뒤에 값이 필요합니다.");
                }
                para = Some(args[i].clone());
            }
            "--ctrl" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--ctrl 뒤에 값이 필요합니다.");
                }
                ctrl = Some(args[i].clone());
            }
            "--name" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--name 뒤에 값이 필요합니다.");
                }
                name = Some(args[i].clone());
            }
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("-o/--output 뒤에 경로가 필요합니다.");
                }
                output_path = Some(args[i].clone());
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }

    let section = parse_usize_cli(section, "--section");
    let para = parse_usize_cli(para, "--para");
    let ctrl = parse_usize_cli(ctrl, "--ctrl");
    let name = name.unwrap_or_else(|| exit_cli_error("--name 값이 필요합니다."));
    let output = output_path.unwrap_or_else(|| input.clone());
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let result = rename_hwp_bookmark_bytes_for_cli(&data, section, para, ctrl, &name)
        .unwrap_or_else(|e| exit_cli_error(&e));
    write_hwp_cli_output(&output, &result.bytes).unwrap_or_else(|e| exit_cli_error(&e));
    print_hwp_edit_cli_result(output, result);
}

fn delete_bookmark_cli(args: &[String]) {
    if args.is_empty() {
        exit_cli_error(
            "사용법: rhwp delete-bookmark <파일.hwp> --section N --para N --ctrl N -o <출력.hwp>",
        );
    }
    let input = args[0].clone();
    let mut section: Option<String> = None;
    let mut para: Option<String> = None;
    let mut ctrl: Option<String> = None;
    let mut output_path: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--section" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--section 뒤에 값이 필요합니다.");
                }
                section = Some(args[i].clone());
            }
            "--para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--para 뒤에 값이 필요합니다.");
                }
                para = Some(args[i].clone());
            }
            "--ctrl" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--ctrl 뒤에 값이 필요합니다.");
                }
                ctrl = Some(args[i].clone());
            }
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("-o/--output 뒤에 경로가 필요합니다.");
                }
                output_path = Some(args[i].clone());
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }

    let section = parse_usize_cli(section, "--section");
    let para = parse_usize_cli(para, "--para");
    let ctrl = parse_usize_cli(ctrl, "--ctrl");
    let output = output_path.unwrap_or_else(|| input.clone());
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let result = delete_hwp_bookmark_bytes_for_cli(&data, section, para, ctrl)
        .unwrap_or_else(|e| exit_cli_error(&e));
    write_hwp_cli_output(&output, &result.bytes).unwrap_or_else(|e| exit_cli_error(&e));
    print_hwp_edit_cli_result(output, result);
}

fn note_create_cli(args: &[String], is_endnote: bool) {
    if args.is_empty() {
        exit_cli_error("사용법: rhwp create-footnote|create-endnote <파일.hwp> --section N --para N --offset N [--text <텍스트>|--text-file <파일>] -o <출력.hwp>");
    }
    let input = args[0].clone();
    let mut section: Option<String> = None;
    let mut para: Option<String> = None;
    let mut offset: Option<String> = None;
    let mut inline_text: Option<String> = None;
    let mut text_file: Option<String> = None;
    let mut output_path: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--section" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--section 뒤에 값이 필요합니다.");
                }
                section = Some(args[i].clone());
            }
            "--para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--para 뒤에 값이 필요합니다.");
                }
                para = Some(args[i].clone());
            }
            "--offset" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--offset 뒤에 값이 필요합니다.");
                }
                offset = Some(args[i].clone());
            }
            "--text" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--text 뒤에 텍스트가 필요합니다.");
                }
                inline_text = Some(args[i].clone());
            }
            "--text-file" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--text-file 뒤에 경로가 필요합니다.");
                }
                text_file = Some(args[i].clone());
            }
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("-o/--output 뒤에 경로가 필요합니다.");
                }
                output_path = Some(args[i].clone());
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }

    let section = parse_usize_cli(section, "--section");
    let para = parse_usize_cli(para, "--para");
    let offset = parse_usize_cli(offset, "--offset");
    let text =
        read_optional_text_argument(inline_text, text_file).unwrap_or_else(|e| exit_cli_error(&e));
    let output = output_path.unwrap_or_else(|| input.clone());
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let result =
        create_hwp_note_bytes_for_cli(&data, section, para, offset, is_endnote, text.as_deref())
            .unwrap_or_else(|e| exit_cli_error(&e));
    write_hwp_cli_output(&output, &result.bytes).unwrap_or_else(|e| exit_cli_error(&e));
    print_hwp_edit_cli_result(output, result);
}

fn note_get_cli(args: &[String]) {
    if args.is_empty() {
        exit_cli_error("사용법: rhwp get-footnote <파일.hwp> --section N --para N --ctrl N");
    }
    let input = args[0].clone();
    let mut section: Option<String> = None;
    let mut para: Option<String> = None;
    let mut ctrl: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--section" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--section 뒤에 값이 필요합니다.");
                }
                section = Some(args[i].clone());
            }
            "--para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--para 뒤에 값이 필요합니다.");
                }
                para = Some(args[i].clone());
            }
            "--ctrl" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--ctrl 뒤에 값이 필요합니다.");
                }
                ctrl = Some(args[i].clone());
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }

    let section = parse_usize_cli(section, "--section");
    let para = parse_usize_cli(para, "--para");
    let ctrl = parse_usize_cli(ctrl, "--ctrl");
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let result = get_hwp_footnote_info_json_for_cli(&data, section, para, ctrl)
        .unwrap_or_else(|e| exit_cli_error(&e));
    println!("{}", result);
}

fn note_text_cli(args: &[String], action: &str) {
    if args.is_empty() {
        exit_cli_error("사용법: rhwp insert|delete-footnote-text <파일.hwp> --section N --para N --ctrl N --note-para N --offset N (--text <텍스트>|--count N) -o <출력.hwp>");
    }
    let input = args[0].clone();
    let mut section: Option<String> = None;
    let mut para: Option<String> = None;
    let mut ctrl: Option<String> = None;
    let mut note_para: Option<String> = None;
    let mut offset: Option<String> = None;
    let mut count: Option<String> = None;
    let mut inline_text: Option<String> = None;
    let mut text_file: Option<String> = None;
    let mut output_path: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--section" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--section 뒤에 값이 필요합니다.");
                }
                section = Some(args[i].clone());
            }
            "--para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--para 뒤에 값이 필요합니다.");
                }
                para = Some(args[i].clone());
            }
            "--ctrl" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--ctrl 뒤에 값이 필요합니다.");
                }
                ctrl = Some(args[i].clone());
            }
            "--note-para" | "--fn-para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--note-para 뒤에 값이 필요합니다.");
                }
                note_para = Some(args[i].clone());
            }
            "--offset" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--offset 뒤에 값이 필요합니다.");
                }
                offset = Some(args[i].clone());
            }
            "--count" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--count 뒤에 값이 필요합니다.");
                }
                count = Some(args[i].clone());
            }
            "--text" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--text 뒤에 텍스트가 필요합니다.");
                }
                inline_text = Some(args[i].clone());
            }
            "--text-file" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--text-file 뒤에 경로가 필요합니다.");
                }
                text_file = Some(args[i].clone());
            }
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("-o/--output 뒤에 경로가 필요합니다.");
                }
                output_path = Some(args[i].clone());
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }

    let section = parse_usize_cli(section, "--section");
    let para = parse_usize_cli(para, "--para");
    let ctrl = parse_usize_cli(ctrl, "--ctrl");
    let note_para = parse_usize_cli(note_para, "--note-para");
    let offset = parse_usize_cli(offset, "--offset");
    let output = output_path.unwrap_or_else(|| input.clone());
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let result = match action {
        "insert" => {
            let text = read_optional_text_argument(inline_text, text_file)
                .unwrap_or_else(|e| exit_cli_error(&e))
                .unwrap_or_else(|| exit_cli_error("--text 또는 --text-file 값이 필요합니다."));
            insert_hwp_footnote_text_bytes_for_cli(
                &data, section, para, ctrl, note_para, offset, &text,
            )
        }
        "delete" => {
            let count = parse_usize_cli(count, "--count");
            delete_hwp_footnote_text_bytes_for_cli(
                &data, section, para, ctrl, note_para, offset, count,
            )
        }
        _ => Err(format!("지원하지 않는 각주/미주 텍스트 작업: {}", action)),
    }
    .unwrap_or_else(|e| exit_cli_error(&e));
    write_hwp_cli_output(&output, &result.bytes).unwrap_or_else(|e| exit_cli_error(&e));
    print_hwp_edit_cli_result(output, result);
}

fn note_paragraph_cli(args: &[String], action: &str) {
    if args.is_empty() {
        exit_cli_error("사용법: rhwp split|merge-footnote-paragraph <파일.hwp> --section N --para N --ctrl N --note-para N [--offset N] -o <출력.hwp>");
    }
    let input = args[0].clone();
    let mut section: Option<String> = None;
    let mut para: Option<String> = None;
    let mut ctrl: Option<String> = None;
    let mut note_para: Option<String> = None;
    let mut offset: Option<String> = None;
    let mut output_path: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--section" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--section 뒤에 값이 필요합니다.");
                }
                section = Some(args[i].clone());
            }
            "--para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--para 뒤에 값이 필요합니다.");
                }
                para = Some(args[i].clone());
            }
            "--ctrl" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--ctrl 뒤에 값이 필요합니다.");
                }
                ctrl = Some(args[i].clone());
            }
            "--note-para" | "--fn-para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--note-para 뒤에 값이 필요합니다.");
                }
                note_para = Some(args[i].clone());
            }
            "--offset" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--offset 뒤에 값이 필요합니다.");
                }
                offset = Some(args[i].clone());
            }
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("-o/--output 뒤에 경로가 필요합니다.");
                }
                output_path = Some(args[i].clone());
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }

    let section = parse_usize_cli(section, "--section");
    let para = parse_usize_cli(para, "--para");
    let ctrl = parse_usize_cli(ctrl, "--ctrl");
    let note_para = parse_usize_cli(note_para, "--note-para");
    let output = output_path.unwrap_or_else(|| input.clone());
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let result = match action {
        "split" => {
            let offset = parse_usize_cli(offset, "--offset");
            split_hwp_footnote_paragraph_bytes_for_cli(
                &data, section, para, ctrl, note_para, offset,
            )
        }
        "merge" => {
            merge_hwp_footnote_paragraph_bytes_for_cli(&data, section, para, ctrl, note_para)
        }
        _ => Err(format!("지원하지 않는 각주/미주 문단 작업: {}", action)),
    }
    .unwrap_or_else(|e| exit_cli_error(&e));
    write_hwp_cli_output(&output, &result.bytes).unwrap_or_else(|e| exit_cli_error(&e));
    print_hwp_edit_cli_result(output, result);
}

fn note_delete_cli(args: &[String]) {
    if args.is_empty() {
        exit_cli_error(
            "사용법: rhwp delete-footnote <파일.hwp> --section N --para N --ctrl N -o <출력.hwp>",
        );
    }
    let input = args[0].clone();
    let mut section: Option<String> = None;
    let mut para: Option<String> = None;
    let mut ctrl: Option<String> = None;
    let mut output_path: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--section" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--section 뒤에 값이 필요합니다.");
                }
                section = Some(args[i].clone());
            }
            "--para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--para 뒤에 값이 필요합니다.");
                }
                para = Some(args[i].clone());
            }
            "--ctrl" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--ctrl 뒤에 값이 필요합니다.");
                }
                ctrl = Some(args[i].clone());
            }
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("-o/--output 뒤에 경로가 필요합니다.");
                }
                output_path = Some(args[i].clone());
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }

    let section = parse_usize_cli(section, "--section");
    let para = parse_usize_cli(para, "--para");
    let ctrl = parse_usize_cli(ctrl, "--ctrl");
    let output = output_path.unwrap_or_else(|| input.clone());
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let result = delete_hwp_footnote_bytes_for_cli(&data, section, para, ctrl)
        .unwrap_or_else(|e| exit_cli_error(&e));
    write_hwp_cli_output(&output, &result.bytes).unwrap_or_else(|e| exit_cli_error(&e));
    print_hwp_edit_cli_result(output, result);
}

fn create_table_cli(args: &[String]) {
    if args.is_empty() {
        exit_cli_error("사용법: rhwp create-table <파일.hwp> --section N --para N --offset N --rows N --cols N -o <출력.hwp>");
    }
    let input = args[0].clone();
    let mut section: Option<String> = None;
    let mut para: Option<String> = None;
    let mut offset: Option<String> = None;
    let mut rows: Option<String> = None;
    let mut cols: Option<String> = None;
    let mut output_path: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--section" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--section 뒤에 값이 필요합니다.");
                }
                section = Some(args[i].clone());
            }
            "--para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--para 뒤에 값이 필요합니다.");
                }
                para = Some(args[i].clone());
            }
            "--offset" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--offset 뒤에 값이 필요합니다.");
                }
                offset = Some(args[i].clone());
            }
            "--rows" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--rows 뒤에 값이 필요합니다.");
                }
                rows = Some(args[i].clone());
            }
            "--cols" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--cols 뒤에 값이 필요합니다.");
                }
                cols = Some(args[i].clone());
            }
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("-o/--output 뒤에 경로가 필요합니다.");
                }
                output_path = Some(args[i].clone());
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }

    let section = parse_usize_cli(section, "--section");
    let para = parse_usize_cli(para, "--para");
    let offset = parse_usize_cli(offset, "--offset");
    let rows = parse_u16_cli(rows, "--rows");
    let cols = parse_u16_cli(cols, "--cols");
    let output = output_path.unwrap_or_else(|| input.clone());
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let result = create_hwp_table_bytes_for_cli(&data, section, para, offset, rows, cols)
        .unwrap_or_else(|e| exit_cli_error(&e));
    write_hwp_cli_output(&output, &result.bytes).unwrap_or_else(|e| exit_cli_error(&e));
    println!(
        "{}",
        serde_json::json!({
            "ok": true,
            "path": output,
            "bytes": result.bytes.len(),
            "paraIdx": result.para_idx,
            "controlIdx": result.control_idx,
            "details": result.details,
            "pageCountBefore": result.page_count_before,
            "pageCountAfter": result.page_count_after,
        })
    );
}

fn set_cell_text_cli(args: &[String]) {
    if args.is_empty() {
        exit_cli_error("사용법: rhwp set-cell-text <파일.hwp> --para N --ctrl N --cell N [--cell-para N] --text <텍스트> -o <출력.hwp>");
    }
    let input = args[0].clone();
    let mut para: Option<String> = None;
    let mut ctrl: Option<String> = None;
    let mut cell: Option<String> = None;
    let mut row: Option<String> = None;
    let mut col: Option<String> = None;
    let mut cell_para: Option<String> = Some("0".to_string());
    let mut text: Option<String> = None;
    let mut output_path: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--para 뒤에 값이 필요합니다.");
                }
                para = Some(args[i].clone());
            }
            "--ctrl" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--ctrl 뒤에 값이 필요합니다.");
                }
                ctrl = Some(args[i].clone());
            }
            "--cell" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--cell 뒤에 값이 필요합니다.");
                }
                cell = Some(args[i].clone());
            }
            "--row" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--row 뒤에 값이 필요합니다.");
                }
                row = Some(args[i].clone());
            }
            "--col" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--col 뒤에 값이 필요합니다.");
                }
                col = Some(args[i].clone());
            }
            "--cell-para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--cell-para 뒤에 값이 필요합니다.");
                }
                cell_para = Some(args[i].clone());
            }
            "--text" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--text 뒤에 텍스트가 필요합니다.");
                }
                text = Some(args[i].clone());
            }
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("-o/--output 뒤에 경로가 필요합니다.");
                }
                output_path = Some(args[i].clone());
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }

    let para = parse_usize_cli(para, "--para");
    let ctrl = parse_usize_cli(ctrl, "--ctrl");
    let cell_para = parse_usize_cli(cell_para, "--cell-para");
    let text = text.unwrap_or_else(|| exit_cli_error("--text <텍스트>가 필요합니다."));
    let output = output_path.unwrap_or_else(|| input.clone());
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let result = if row.is_some() || col.is_some() {
        if cell.is_some() {
            exit_cli_error("--cell과 --row/--col은 함께 사용할 수 없습니다.");
        }
        let row = parse_u16_cli(row, "--row");
        let col = parse_u16_cli(col, "--col");
        set_hwp_cell_text_by_position_bytes_for_cli(&data, para, ctrl, row, col, cell_para, &text)
    } else {
        let cell = parse_usize_cli(cell, "--cell");
        set_hwp_cell_text_bytes_for_cli(&data, para, ctrl, cell, cell_para, &text)
    }
    .unwrap_or_else(|e| exit_cli_error(&e));
    write_hwp_cli_output(&output, &result.bytes).unwrap_or_else(|e| exit_cli_error(&e));
    println!(
        "{}",
        serde_json::json!({
            "ok": true,
            "path": output,
            "bytes": result.bytes.len(),
            "details": result.details,
            "pageCountBefore": result.page_count_before,
            "pageCountAfter": result.page_count_after,
        })
    );
}

fn cell_text_edit_cli(args: &[String], delete: bool) {
    if args.is_empty() {
        if delete {
            exit_cli_error("사용법: rhwp delete-cell-text <파일.hwp> --para N --ctrl N (--cell N|--row N --col N) [--cell-para N] --offset N --count N -o <출력.hwp>");
        } else {
            exit_cli_error("사용법: rhwp insert-cell-text <파일.hwp> --para N --ctrl N (--cell N|--row N --col N) [--cell-para N] --offset N --text <텍스트> -o <출력.hwp>");
        }
    }
    let input = args[0].clone();
    let mut para: Option<String> = None;
    let mut ctrl: Option<String> = None;
    let mut cell: Option<String> = None;
    let mut row: Option<String> = None;
    let mut col: Option<String> = None;
    let mut cell_para: Option<String> = Some("0".to_string());
    let mut offset: Option<String> = None;
    let mut count: Option<String> = None;
    let mut inline_text: Option<String> = None;
    let mut text_file: Option<String> = None;
    let mut output_path: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--para 뒤에 값이 필요합니다.");
                }
                para = Some(args[i].clone());
            }
            "--ctrl" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--ctrl 뒤에 값이 필요합니다.");
                }
                ctrl = Some(args[i].clone());
            }
            "--cell" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--cell 뒤에 값이 필요합니다.");
                }
                cell = Some(args[i].clone());
            }
            "--row" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--row 뒤에 값이 필요합니다.");
                }
                row = Some(args[i].clone());
            }
            "--col" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--col 뒤에 값이 필요합니다.");
                }
                col = Some(args[i].clone());
            }
            "--cell-para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--cell-para 뒤에 값이 필요합니다.");
                }
                cell_para = Some(args[i].clone());
            }
            "--offset" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--offset 뒤에 값이 필요합니다.");
                }
                offset = Some(args[i].clone());
            }
            "--count" if delete => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--count 뒤에 값이 필요합니다.");
                }
                count = Some(args[i].clone());
            }
            "--text" if !delete => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--text 뒤에 텍스트가 필요합니다.");
                }
                inline_text = Some(args[i].clone());
            }
            "--text-file" if !delete => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--text-file 뒤에 경로가 필요합니다.");
                }
                text_file = Some(args[i].clone());
            }
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("-o/--output 뒤에 경로가 필요합니다.");
                }
                output_path = Some(args[i].clone());
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }

    let para = parse_usize_cli(para, "--para");
    let ctrl = parse_usize_cli(ctrl, "--ctrl");
    let cell_para = parse_usize_cli(cell_para, "--cell-para");
    let offset = parse_usize_cli(offset, "--offset");
    let output = output_path.unwrap_or_else(|| input.clone());
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));

    let result = if row.is_some() || col.is_some() {
        if cell.is_some() {
            exit_cli_error("--cell과 --row/--col은 함께 사용할 수 없습니다.");
        }
        let row = parse_u16_cli(row, "--row");
        let col = parse_u16_cli(col, "--col");
        if delete {
            let count = parse_usize_cli(count, "--count");
            delete_hwp_cell_text_by_position_bytes_for_cli(
                &data, para, ctrl, row, col, cell_para, offset, count,
            )
        } else {
            let text = read_optional_text_argument(inline_text, text_file)
                .unwrap_or_else(|e| exit_cli_error(&e))
                .unwrap_or_else(|| exit_cli_error("--text 또는 --text-file 값이 필요합니다."));
            insert_hwp_cell_text_by_position_bytes_for_cli(
                &data, para, ctrl, row, col, cell_para, offset, &text,
            )
        }
    } else {
        let cell = parse_usize_cli(cell, "--cell");
        if delete {
            let count = parse_usize_cli(count, "--count");
            delete_hwp_cell_text_bytes_for_cli(&data, para, ctrl, cell, cell_para, offset, count)
        } else {
            let text = read_optional_text_argument(inline_text, text_file)
                .unwrap_or_else(|e| exit_cli_error(&e))
                .unwrap_or_else(|| exit_cli_error("--text 또는 --text-file 값이 필요합니다."));
            insert_hwp_cell_text_bytes_for_cli(&data, para, ctrl, cell, cell_para, offset, &text)
        }
    }
    .unwrap_or_else(|e| exit_cli_error(&e));
    write_hwp_cli_output(&output, &result.bytes).unwrap_or_else(|e| exit_cli_error(&e));
    print_hwp_edit_cli_result(output, result);
}

fn cell_paragraph_cli(args: &[String], merge: bool) {
    if args.is_empty() {
        if merge {
            exit_cli_error("사용법: rhwp merge-cell-paragraph <파일.hwp> --para N --ctrl N (--cell N|--row N --col N) --cell-para N -o <출력.hwp>");
        } else {
            exit_cli_error("사용법: rhwp split-cell-paragraph <파일.hwp> --para N --ctrl N (--cell N|--row N --col N) [--cell-para N] --offset N -o <출력.hwp>");
        }
    }
    let input = args[0].clone();
    let mut para: Option<String> = None;
    let mut ctrl: Option<String> = None;
    let mut cell: Option<String> = None;
    let mut row: Option<String> = None;
    let mut col: Option<String> = None;
    let mut cell_para: Option<String> = if merge { None } else { Some("0".to_string()) };
    let mut offset: Option<String> = None;
    let mut output_path: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--para 뒤에 값이 필요합니다.");
                }
                para = Some(args[i].clone());
            }
            "--ctrl" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--ctrl 뒤에 값이 필요합니다.");
                }
                ctrl = Some(args[i].clone());
            }
            "--cell" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--cell 뒤에 값이 필요합니다.");
                }
                cell = Some(args[i].clone());
            }
            "--row" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--row 뒤에 값이 필요합니다.");
                }
                row = Some(args[i].clone());
            }
            "--col" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--col 뒤에 값이 필요합니다.");
                }
                col = Some(args[i].clone());
            }
            "--cell-para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--cell-para 뒤에 값이 필요합니다.");
                }
                cell_para = Some(args[i].clone());
            }
            "--offset" if !merge => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--offset 뒤에 값이 필요합니다.");
                }
                offset = Some(args[i].clone());
            }
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("-o/--output 뒤에 경로가 필요합니다.");
                }
                output_path = Some(args[i].clone());
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }

    let para = parse_usize_cli(para, "--para");
    let ctrl = parse_usize_cli(ctrl, "--ctrl");
    let cell_para = parse_usize_cli(cell_para, "--cell-para");
    let output = output_path.unwrap_or_else(|| input.clone());
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));

    let result = if row.is_some() || col.is_some() {
        if cell.is_some() {
            exit_cli_error("--cell과 --row/--col은 함께 사용할 수 없습니다.");
        }
        let row = parse_u16_cli(row, "--row");
        let col = parse_u16_cli(col, "--col");
        if merge {
            merge_hwp_cell_paragraph_by_position_bytes_for_cli(
                &data, para, ctrl, row, col, cell_para,
            )
        } else {
            let offset = parse_usize_cli(offset, "--offset");
            split_hwp_cell_paragraph_by_position_bytes_for_cli(
                &data, para, ctrl, row, col, cell_para, offset,
            )
        }
    } else {
        let cell = parse_usize_cli(cell, "--cell");
        if merge {
            merge_hwp_cell_paragraph_bytes_for_cli(&data, para, ctrl, cell, cell_para)
        } else {
            let offset = parse_usize_cli(offset, "--offset");
            split_hwp_cell_paragraph_bytes_for_cli(&data, para, ctrl, cell, cell_para, offset)
        }
    }
    .unwrap_or_else(|e| exit_cli_error(&e));
    write_hwp_cli_output(&output, &result.bytes).unwrap_or_else(|e| exit_cli_error(&e));
    print_hwp_edit_cli_result(output, result);
}

fn cell_paragraph_edit_cli(args: &[String], delete: bool) {
    if args.is_empty() {
        if delete {
            exit_cli_error("사용법: rhwp delete-cell-paragraph <파일.hwp> --para N --ctrl N (--cell N|--row N --col N) --cell-para N -o <출력.hwp>");
        } else {
            exit_cli_error("사용법: rhwp insert-cell-paragraph <파일.hwp> --para N --ctrl N (--cell N|--row N --col N) --cell-para N [--text <텍스트>|--text-file <경로>] -o <출력.hwp>");
        }
    }
    let input = args[0].clone();
    let mut para: Option<String> = None;
    let mut ctrl: Option<String> = None;
    let mut cell: Option<String> = None;
    let mut row: Option<String> = None;
    let mut col: Option<String> = None;
    let mut cell_para: Option<String> = None;
    let mut inline_text: Option<String> = None;
    let mut text_file: Option<String> = None;
    let mut output_path: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--para 뒤에 값이 필요합니다.");
                }
                para = Some(args[i].clone());
            }
            "--ctrl" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--ctrl 뒤에 값이 필요합니다.");
                }
                ctrl = Some(args[i].clone());
            }
            "--cell" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--cell 뒤에 값이 필요합니다.");
                }
                cell = Some(args[i].clone());
            }
            "--row" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--row 뒤에 값이 필요합니다.");
                }
                row = Some(args[i].clone());
            }
            "--col" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--col 뒤에 값이 필요합니다.");
                }
                col = Some(args[i].clone());
            }
            "--cell-para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--cell-para 뒤에 값이 필요합니다.");
                }
                cell_para = Some(args[i].clone());
            }
            "--text" if !delete => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--text 뒤에 텍스트가 필요합니다.");
                }
                inline_text = Some(args[i].clone());
            }
            "--text-file" if !delete => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--text-file 뒤에 경로가 필요합니다.");
                }
                text_file = Some(args[i].clone());
            }
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("-o/--output 뒤에 경로가 필요합니다.");
                }
                output_path = Some(args[i].clone());
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }

    let para = parse_usize_cli(para, "--para");
    let ctrl = parse_usize_cli(ctrl, "--ctrl");
    let cell_para = parse_usize_cli(cell_para, "--cell-para");
    let text = if delete {
        None
    } else {
        read_optional_text_argument(inline_text, text_file).unwrap_or_else(|e| exit_cli_error(&e))
    };
    let output = output_path.unwrap_or_else(|| input.clone());
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));

    let result = if row.is_some() || col.is_some() {
        if cell.is_some() {
            exit_cli_error("--cell과 --row/--col은 함께 사용할 수 없습니다.");
        }
        let row = parse_u16_cli(row, "--row");
        let col = parse_u16_cli(col, "--col");
        if delete {
            delete_hwp_cell_paragraph_by_position_bytes_for_cli(
                &data, para, ctrl, row, col, cell_para,
            )
        } else {
            insert_hwp_cell_paragraph_by_position_bytes_for_cli(
                &data,
                para,
                ctrl,
                row,
                col,
                cell_para,
                text.as_deref(),
            )
        }
    } else {
        let cell = parse_usize_cli(cell, "--cell");
        if delete {
            delete_hwp_cell_paragraph_bytes_for_cli(&data, para, ctrl, cell, cell_para)
        } else {
            insert_hwp_cell_paragraph_bytes_for_cli(
                &data,
                para,
                ctrl,
                cell,
                cell_para,
                text.as_deref(),
            )
        }
    }
    .unwrap_or_else(|e| exit_cli_error(&e));
    write_hwp_cli_output(&output, &result.bytes).unwrap_or_else(|e| exit_cli_error(&e));
    print_hwp_edit_cli_result(output, result);
}

fn cell_field_cli(args: &[String], clear: bool) {
    if args.is_empty() {
        if clear {
            exit_cli_error(
                "사용법: rhwp clear-cell-field <파일.hwp> --para N --ctrl N --cell N -o <출력.hwp>",
            );
        } else {
            exit_cli_error("사용법: rhwp set-cell-field <파일.hwp> --para N --ctrl N --cell N --name <필드명> -o <출력.hwp>");
        }
    }
    let input = args[0].clone();
    let mut para: Option<String> = None;
    let mut ctrl: Option<String> = None;
    let mut cell: Option<String> = None;
    let mut row: Option<String> = None;
    let mut col: Option<String> = None;
    let mut name: Option<String> = None;
    let mut output_path: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--para 뒤에 값이 필요합니다.");
                }
                para = Some(args[i].clone());
            }
            "--ctrl" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--ctrl 뒤에 값이 필요합니다.");
                }
                ctrl = Some(args[i].clone());
            }
            "--cell" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--cell 뒤에 값이 필요합니다.");
                }
                cell = Some(args[i].clone());
            }
            "--row" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--row 뒤에 값이 필요합니다.");
                }
                row = Some(args[i].clone());
            }
            "--col" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--col 뒤에 값이 필요합니다.");
                }
                col = Some(args[i].clone());
            }
            "--name" if !clear => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--name 뒤에 필드명이 필요합니다.");
                }
                name = Some(args[i].clone());
            }
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("-o/--output 뒤에 경로가 필요합니다.");
                }
                output_path = Some(args[i].clone());
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }

    let para = parse_usize_cli(para, "--para");
    let ctrl = parse_usize_cli(ctrl, "--ctrl");
    let name = if clear {
        None
    } else {
        let name = name.unwrap_or_else(|| exit_cli_error("--name <필드명>이 필요합니다."));
        if name.trim().is_empty() {
            exit_cli_error("필드명은 비어 있을 수 없습니다.");
        }
        Some(name)
    };
    let output = output_path.unwrap_or_else(|| input.clone());
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let result = if row.is_some() || col.is_some() {
        if cell.is_some() {
            exit_cli_error("--cell과 --row/--col은 함께 사용할 수 없습니다.");
        }
        let row = parse_u16_cli(row, "--row");
        let col = parse_u16_cli(col, "--col");
        set_hwp_cell_field_by_position_bytes_for_cli(&data, para, ctrl, row, col, name.as_deref())
    } else {
        let cell = parse_usize_cli(cell, "--cell");
        set_hwp_cell_field_bytes_for_cli(&data, para, ctrl, cell, name.as_deref())
    }
    .unwrap_or_else(|e| exit_cli_error(&e));
    write_hwp_cli_output(&output, &result.bytes).unwrap_or_else(|e| exit_cli_error(&e));
    print_hwp_edit_cli_result(output, result);
}

fn table_structure_usage(operation: &str) -> &'static str {
    match operation {
        "copy-table" => {
            "사용법: rhwp copy-table <파일.hwp> --section N --para N --ctrl N [--before|--after] [--replace OLD NEW]... -o <출력.hwp>"
        }
        "delete-table" => {
            "사용법: rhwp delete-table <파일.hwp> --section N --para N --ctrl N -o <출력.hwp>"
        }
        "insert-table-row" => "사용법: rhwp insert-table-row <파일.hwp> --section N --para N --ctrl N --row N [--above|--below] -o <출력.hwp>",
        "copy-table-row" => "사용법: rhwp copy-table-row <파일.hwp> --section N --para N --ctrl N --row N [--above|--below] [--replace OLD NEW]... -o <출력.hwp>",
        "delete-table-row" => {
            "사용법: rhwp delete-table-row <파일.hwp> --section N --para N --ctrl N --row N -o <출력.hwp>"
        }
        "insert-table-column" => "사용법: rhwp insert-table-column <파일.hwp> --section N --para N --ctrl N --col N [--left|--right] -o <출력.hwp>",
        "copy-table-column" => "사용법: rhwp copy-table-column <파일.hwp> --section N --para N --ctrl N --col N [--left|--right] [--replace OLD NEW]... -o <출력.hwp>",
        "delete-table-column" => {
            "사용법: rhwp delete-table-column <파일.hwp> --section N --para N --ctrl N --col N -o <출력.hwp>"
        }
        "merge-table-cells" => "사용법: rhwp merge-table-cells <파일.hwp> --section N --para N --ctrl N --start-row N --start-col N --end-row N --end-col N -o <출력.hwp>",
        "split-table-cell" => "사용법: rhwp split-table-cell <파일.hwp> --section N --para N --ctrl N --row N --col N -o <출력.hwp>",
        _ => "사용법: rhwp <table-command> <파일.hwp> [옵션] -o <출력.hwp>",
    }
}

fn table_structure_cli(args: &[String], operation: &str) {
    if args.is_empty() {
        exit_cli_error(table_structure_usage(operation));
    }
    let input = args[0].clone();
    let mut section: Option<String> = None;
    let mut para: Option<String> = None;
    let mut ctrl: Option<String> = None;
    let mut row: Option<String> = None;
    let mut col: Option<String> = None;
    let mut start_row: Option<String> = None;
    let mut start_col: Option<String> = None;
    let mut end_row: Option<String> = None;
    let mut end_col: Option<String> = None;
    let mut below = true;
    let mut right = true;
    let mut replacements: Vec<(String, String)> = Vec::new();
    let mut output_path: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--section" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--section 뒤에 값이 필요합니다.");
                }
                section = Some(args[i].clone());
            }
            "--para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--para 뒤에 값이 필요합니다.");
                }
                para = Some(args[i].clone());
            }
            "--ctrl" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--ctrl 뒤에 값이 필요합니다.");
                }
                ctrl = Some(args[i].clone());
            }
            "--row" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--row 뒤에 값이 필요합니다.");
                }
                row = Some(args[i].clone());
            }
            "--col" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--col 뒤에 값이 필요합니다.");
                }
                col = Some(args[i].clone());
            }
            "--start-row" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--start-row 뒤에 값이 필요합니다.");
                }
                start_row = Some(args[i].clone());
            }
            "--start-col" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--start-col 뒤에 값이 필요합니다.");
                }
                start_col = Some(args[i].clone());
            }
            "--end-row" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--end-row 뒤에 값이 필요합니다.");
                }
                end_row = Some(args[i].clone());
            }
            "--end-col" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--end-col 뒤에 값이 필요합니다.");
                }
                end_col = Some(args[i].clone());
            }
            "--above" => below = false,
            "--below" => below = true,
            "--before" => below = false,
            "--after" => below = true,
            "--left" => right = false,
            "--right" => right = true,
            "--replace" => {
                if operation != "copy-table"
                    && operation != "copy-table-row"
                    && operation != "copy-table-column"
                {
                    exit_cli_error(
                        "--replace는 copy-table/copy-table-row/copy-table-column에서만 지원합니다.",
                    );
                }
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--replace 뒤에 검색어가 필요합니다.");
                }
                let old = args[i].clone();
                if old.is_empty() {
                    exit_cli_error("--replace 검색어는 비어 있을 수 없습니다.");
                }
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--replace 뒤에 대체문구가 필요합니다.");
                }
                replacements.push((old, args[i].clone()));
            }
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("-o/--output 뒤에 경로가 필요합니다.");
                }
                output_path = Some(args[i].clone());
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }

    let section = parse_usize_cli(section, "--section");
    let para = parse_usize_cli(para, "--para");
    let ctrl = parse_usize_cli(ctrl, "--ctrl");
    let output = output_path.unwrap_or_else(|| input.clone());
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));

    let result = match operation {
        "copy-table" => {
            if replacements.is_empty() {
                copy_hwp_table_bytes_for_cli(&data, section, para, ctrl, below)
            } else {
                copy_hwp_table_with_replacements_bytes_for_cli(
                    &data,
                    section,
                    para,
                    ctrl,
                    below,
                    &replacements,
                )
            }
        }
        "delete-table" => delete_hwp_table_bytes_for_cli(&data, section, para, ctrl),
        "insert-table-row" => {
            let row = parse_u16_cli(row, "--row");
            insert_hwp_table_row_bytes_for_cli(&data, section, para, ctrl, row, below)
        }
        "copy-table-row" => {
            let row = parse_u16_cli(row, "--row");
            if replacements.is_empty() {
                copy_hwp_table_row_bytes_for_cli(&data, section, para, ctrl, row, below)
            } else {
                copy_hwp_table_row_with_replacements_bytes_for_cli(
                    &data,
                    section,
                    para,
                    ctrl,
                    row,
                    below,
                    &replacements,
                )
            }
        }
        "delete-table-row" => {
            let row = parse_u16_cli(row, "--row");
            delete_hwp_table_row_bytes_for_cli(&data, section, para, ctrl, row)
        }
        "insert-table-column" => {
            let col = parse_u16_cli(col, "--col");
            insert_hwp_table_column_bytes_for_cli(&data, section, para, ctrl, col, right)
        }
        "copy-table-column" => {
            let col = parse_u16_cli(col, "--col");
            if replacements.is_empty() {
                copy_hwp_table_column_bytes_for_cli(&data, section, para, ctrl, col, right)
            } else {
                copy_hwp_table_column_with_replacements_bytes_for_cli(
                    &data,
                    section,
                    para,
                    ctrl,
                    col,
                    right,
                    &replacements,
                )
            }
        }
        "delete-table-column" => {
            let col = parse_u16_cli(col, "--col");
            delete_hwp_table_column_bytes_for_cli(&data, section, para, ctrl, col)
        }
        "merge-table-cells" => {
            let start_row = parse_u16_cli(start_row, "--start-row");
            let start_col = parse_u16_cli(start_col, "--start-col");
            let end_row = parse_u16_cli(end_row, "--end-row");
            let end_col = parse_u16_cli(end_col, "--end-col");
            merge_hwp_table_cells_bytes_for_cli(
                &data, section, para, ctrl, start_row, start_col, end_row, end_col,
            )
        }
        "split-table-cell" => {
            let row = parse_u16_cli(row, "--row");
            let col = parse_u16_cli(col, "--col");
            split_hwp_table_cell_bytes_for_cli(&data, section, para, ctrl, row, col)
        }
        _ => Err(format!("지원하지 않는 표 구조 명령: {}", operation)),
    }
    .unwrap_or_else(|e| exit_cli_error(&e));

    write_hwp_cli_output(&output, &result.bytes).unwrap_or_else(|e| exit_cli_error(&e));
    println!(
        "{}",
        serde_json::json!({
            "ok": true,
            "path": output,
            "bytes": result.bytes.len(),
            "details": result.details,
            "pageCountBefore": result.page_count_before,
            "pageCountAfter": result.page_count_after,
        })
    );
}

fn get_table_properties_cli(args: &[String], is_cell: bool) {
    if args.is_empty() {
        let usage = if is_cell {
            "사용법: rhwp get-cell-properties <파일.hwp> --section N --para N --ctrl N (--cell N|--row N --col N)"
        } else {
            "사용법: rhwp get-table-properties <파일.hwp> --section N --para N --ctrl N"
        };
        exit_cli_error(usage);
    }
    let input = args[0].clone();
    let mut section: Option<String> = None;
    let mut para: Option<String> = None;
    let mut ctrl: Option<String> = None;
    let mut cell: Option<String> = None;
    let mut row: Option<String> = None;
    let mut col: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--section" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--section 뒤에 값이 필요합니다.");
                }
                section = Some(args[i].clone());
            }
            "--para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--para 뒤에 값이 필요합니다.");
                }
                para = Some(args[i].clone());
            }
            "--ctrl" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--ctrl 뒤에 값이 필요합니다.");
                }
                ctrl = Some(args[i].clone());
            }
            "--cell" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--cell 뒤에 값이 필요합니다.");
                }
                cell = Some(args[i].clone());
            }
            "--row" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--row 뒤에 값이 필요합니다.");
                }
                row = Some(args[i].clone());
            }
            "--col" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--col 뒤에 값이 필요합니다.");
                }
                col = Some(args[i].clone());
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }

    let section = parse_usize_cli(section, "--section");
    let para = parse_usize_cli(para, "--para");
    let ctrl = parse_usize_cli(ctrl, "--ctrl");
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));

    let result = if is_cell {
        if cell.is_some() && (row.is_some() || col.is_some()) {
            exit_cli_error("--cell과 --row/--col은 함께 사용할 수 없습니다.");
        }
        if let Some(cell) = cell {
            let cell = parse_usize_cli(Some(cell), "--cell");
            get_hwp_cell_properties_json_for_cli(&data, section, para, ctrl, cell)
        } else {
            let row = parse_u16_cli(row, "--row");
            let col = parse_u16_cli(col, "--col");
            get_hwp_cell_properties_at_json_for_cli(&data, section, para, ctrl, row, col)
        }
    } else {
        get_hwp_table_properties_json_for_cli(&data, section, para, ctrl)
    }
    .unwrap_or_else(|e| exit_cli_error(&e));

    println!("{}", result);
}

fn set_table_properties_cli(args: &[String], is_cell: bool) {
    if args.is_empty() {
        let usage = if is_cell {
            "사용법: rhwp set-cell-properties <파일.hwp> --section N --para N --ctrl N (--cell N|--row N --col N) --json <속성JSON> -o <출력.hwp>"
        } else {
            "사용법: rhwp set-table-properties <파일.hwp> --section N --para N --ctrl N --json <속성JSON> -o <출력.hwp>"
        };
        exit_cli_error(usage);
    }
    let input = args[0].clone();
    let mut section: Option<String> = None;
    let mut para: Option<String> = None;
    let mut ctrl: Option<String> = None;
    let mut cell: Option<String> = None;
    let mut row: Option<String> = None;
    let mut col: Option<String> = None;
    let mut inline_json: Option<String> = None;
    let mut json_file: Option<String> = None;
    let mut output_path: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--section" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--section 뒤에 값이 필요합니다.");
                }
                section = Some(args[i].clone());
            }
            "--para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--para 뒤에 값이 필요합니다.");
                }
                para = Some(args[i].clone());
            }
            "--ctrl" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--ctrl 뒤에 값이 필요합니다.");
                }
                ctrl = Some(args[i].clone());
            }
            "--cell" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--cell 뒤에 값이 필요합니다.");
                }
                cell = Some(args[i].clone());
            }
            "--row" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--row 뒤에 값이 필요합니다.");
                }
                row = Some(args[i].clone());
            }
            "--col" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--col 뒤에 값이 필요합니다.");
                }
                col = Some(args[i].clone());
            }
            "--json" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--json 뒤에 JSON 문자열이 필요합니다.");
                }
                inline_json = Some(args[i].clone());
            }
            "--json-file" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--json-file 뒤에 경로가 필요합니다.");
                }
                json_file = Some(args[i].clone());
            }
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("-o/--output 뒤에 경로가 필요합니다.");
                }
                output_path = Some(args[i].clone());
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }

    let section = parse_usize_cli(section, "--section");
    let para = parse_usize_cli(para, "--para");
    let ctrl = parse_usize_cli(ctrl, "--ctrl");
    let props_json =
        read_json_argument(inline_json, json_file).unwrap_or_else(|e| exit_cli_error(&e));
    let output = output_path.unwrap_or_else(|| input.clone());
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));

    let result = if is_cell {
        if cell.is_some() && (row.is_some() || col.is_some()) {
            exit_cli_error("--cell과 --row/--col은 함께 사용할 수 없습니다.");
        }
        if let Some(cell) = cell {
            let cell = parse_usize_cli(Some(cell), "--cell");
            set_hwp_cell_properties_bytes_for_cli(&data, section, para, ctrl, cell, &props_json)
        } else {
            let row = parse_u16_cli(row, "--row");
            let col = parse_u16_cli(col, "--col");
            set_hwp_cell_properties_at_bytes_for_cli(
                &data,
                section,
                para,
                ctrl,
                row,
                col,
                &props_json,
            )
        }
    } else {
        set_hwp_table_properties_bytes_for_cli(&data, section, para, ctrl, &props_json)
    }
    .unwrap_or_else(|e| exit_cli_error(&e));

    write_hwp_cli_output(&output, &result.bytes).unwrap_or_else(|e| exit_cli_error(&e));
    println!(
        "{}",
        serde_json::json!({
            "ok": true,
            "path": output,
            "bytes": result.bytes.len(),
            "details": result.details,
            "pageCountBefore": result.page_count_before,
            "pageCountAfter": result.page_count_after,
        })
    );
}

fn resize_table_cells_cli(args: &[String]) {
    if args.is_empty() {
        exit_cli_error("사용법: rhwp resize-table-cells <파일.hwp> --section N --para N --ctrl N --json <변경배열JSON> -o <출력.hwp>");
    }
    let input = args[0].clone();
    let mut section: Option<String> = None;
    let mut para: Option<String> = None;
    let mut ctrl: Option<String> = None;
    let mut inline_json: Option<String> = None;
    let mut json_file: Option<String> = None;
    let mut output_path: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--section" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--section 뒤에 값이 필요합니다.");
                }
                section = Some(args[i].clone());
            }
            "--para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--para 뒤에 값이 필요합니다.");
                }
                para = Some(args[i].clone());
            }
            "--ctrl" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--ctrl 뒤에 값이 필요합니다.");
                }
                ctrl = Some(args[i].clone());
            }
            "--json" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--json 뒤에 JSON 문자열이 필요합니다.");
                }
                inline_json = Some(args[i].clone());
            }
            "--json-file" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--json-file 뒤에 경로가 필요합니다.");
                }
                json_file = Some(args[i].clone());
            }
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("-o/--output 뒤에 경로가 필요합니다.");
                }
                output_path = Some(args[i].clone());
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }

    let section = parse_usize_cli(section, "--section");
    let para = parse_usize_cli(para, "--para");
    let ctrl = parse_usize_cli(ctrl, "--ctrl");
    let updates_json =
        read_json_argument(inline_json, json_file).unwrap_or_else(|e| exit_cli_error(&e));
    let output = output_path.unwrap_or_else(|| input.clone());
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let result = resize_hwp_table_cells_bytes_for_cli(&data, section, para, ctrl, &updates_json)
        .unwrap_or_else(|e| exit_cli_error(&e));

    write_hwp_cli_output(&output, &result.bytes).unwrap_or_else(|e| exit_cli_error(&e));
    println!(
        "{}",
        serde_json::json!({
            "ok": true,
            "path": output,
            "bytes": result.bytes.len(),
            "details": result.details,
            "pageCountBefore": result.page_count_before,
            "pageCountAfter": result.page_count_after,
        })
    );
}

fn get_format_properties_cli(args: &[String], kind: &str) {
    if args.is_empty() {
        exit_cli_error("사용법: rhwp get-*-properties <파일.hwp> --section N --para N [--ctrl N (--cell N|--row R --col C) --cell-para N] [--offset N]");
    }
    let input = args[0].clone();
    let mut section: Option<String> = None;
    let mut para: Option<String> = None;
    let mut ctrl: Option<String> = None;
    let mut cell: Option<String> = None;
    let mut row: Option<String> = None;
    let mut col: Option<String> = None;
    let mut cell_para: Option<String> = Some("0".to_string());
    let mut offset: Option<String> = Some("0".to_string());

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--section" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--section 뒤에 값이 필요합니다.");
                }
                section = Some(args[i].clone());
            }
            "--para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--para 뒤에 값이 필요합니다.");
                }
                para = Some(args[i].clone());
            }
            "--ctrl" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--ctrl 뒤에 값이 필요합니다.");
                }
                ctrl = Some(args[i].clone());
            }
            "--cell" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--cell 뒤에 값이 필요합니다.");
                }
                cell = Some(args[i].clone());
            }
            "--row" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--row 뒤에 값이 필요합니다.");
                }
                row = Some(args[i].clone());
            }
            "--col" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--col 뒤에 값이 필요합니다.");
                }
                col = Some(args[i].clone());
            }
            "--cell-para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--cell-para 뒤에 값이 필요합니다.");
                }
                cell_para = Some(args[i].clone());
            }
            "--offset" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--offset 뒤에 값이 필요합니다.");
                }
                offset = Some(args[i].clone());
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }

    let section = parse_usize_cli(section, "--section");
    let para = parse_usize_cli(para, "--para");
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));

    let result = match kind {
        "char" => {
            let offset = parse_usize_cli(offset, "--offset");
            get_hwp_char_properties_json_for_cli(&data, section, para, offset)
        }
        "para" => get_hwp_para_properties_json_for_cli(&data, section, para),
        "cell-char" => {
            let ctrl = parse_usize_cli(ctrl, "--ctrl");
            let cell_para = parse_usize_cli(cell_para, "--cell-para");
            let offset = parse_usize_cli(offset, "--offset");
            if row.is_some() || col.is_some() {
                if cell.is_some() {
                    exit_cli_error("--cell과 --row/--col은 함께 사용할 수 없습니다.");
                }
                let row = parse_u16_cli(row, "--row");
                let col = parse_u16_cli(col, "--col");
                get_hwp_cell_char_properties_at_json_for_cli(
                    &data, section, para, ctrl, row, col, cell_para, offset,
                )
            } else {
                let cell = parse_usize_cli(cell, "--cell");
                get_hwp_cell_char_properties_json_for_cli(
                    &data, section, para, ctrl, cell, cell_para, offset,
                )
            }
        }
        "cell-para" => {
            let ctrl = parse_usize_cli(ctrl, "--ctrl");
            let cell_para = parse_usize_cli(cell_para, "--cell-para");
            if row.is_some() || col.is_some() {
                if cell.is_some() {
                    exit_cli_error("--cell과 --row/--col은 함께 사용할 수 없습니다.");
                }
                let row = parse_u16_cli(row, "--row");
                let col = parse_u16_cli(col, "--col");
                get_hwp_cell_para_properties_at_json_for_cli(
                    &data, section, para, ctrl, row, col, cell_para,
                )
            } else {
                let cell = parse_usize_cli(cell, "--cell");
                get_hwp_cell_para_properties_json_for_cli(
                    &data, section, para, ctrl, cell, cell_para,
                )
            }
        }
        _ => Err(format!("지원하지 않는 서식 조회 명령: {}", kind)),
    }
    .unwrap_or_else(|e| exit_cli_error(&e));

    println!("{}", result);
}

fn set_format_cli(args: &[String], kind: &str) {
    if args.is_empty() {
        exit_cli_error("사용법: rhwp set-*-format <파일.hwp> --section N --para N [--ctrl N (--cell N|--row R --col C) --cell-para N] [--start N --end N] --json <서식JSON> -o <출력.hwp>");
    }
    let input = args[0].clone();
    let mut section: Option<String> = None;
    let mut para: Option<String> = None;
    let mut ctrl: Option<String> = None;
    let mut cell: Option<String> = None;
    let mut row: Option<String> = None;
    let mut col: Option<String> = None;
    let mut cell_para: Option<String> = Some("0".to_string());
    let mut start: Option<String> = None;
    let mut end: Option<String> = None;
    let mut inline_json: Option<String> = None;
    let mut json_file: Option<String> = None;
    let mut output_path: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--section" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--section 뒤에 값이 필요합니다.");
                }
                section = Some(args[i].clone());
            }
            "--para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--para 뒤에 값이 필요합니다.");
                }
                para = Some(args[i].clone());
            }
            "--ctrl" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--ctrl 뒤에 값이 필요합니다.");
                }
                ctrl = Some(args[i].clone());
            }
            "--cell" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--cell 뒤에 값이 필요합니다.");
                }
                cell = Some(args[i].clone());
            }
            "--row" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--row 뒤에 값이 필요합니다.");
                }
                row = Some(args[i].clone());
            }
            "--col" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--col 뒤에 값이 필요합니다.");
                }
                col = Some(args[i].clone());
            }
            "--cell-para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--cell-para 뒤에 값이 필요합니다.");
                }
                cell_para = Some(args[i].clone());
            }
            "--start" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--start 뒤에 값이 필요합니다.");
                }
                start = Some(args[i].clone());
            }
            "--end" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--end 뒤에 값이 필요합니다.");
                }
                end = Some(args[i].clone());
            }
            "--json" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--json 뒤에 JSON 문자열이 필요합니다.");
                }
                inline_json = Some(args[i].clone());
            }
            "--json-file" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--json-file 뒤에 경로가 필요합니다.");
                }
                json_file = Some(args[i].clone());
            }
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("-o/--output 뒤에 경로가 필요합니다.");
                }
                output_path = Some(args[i].clone());
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }

    let section = parse_usize_cli(section, "--section");
    let para = parse_usize_cli(para, "--para");
    let props_json =
        read_json_argument(inline_json, json_file).unwrap_or_else(|e| exit_cli_error(&e));
    let output = output_path.unwrap_or_else(|| input.clone());
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));

    let result = match kind {
        "char" => {
            let start = parse_usize_cli(start, "--start");
            let end = parse_usize_cli(end, "--end");
            set_hwp_char_format_bytes_for_cli(&data, section, para, start, end, &props_json)
        }
        "para" => set_hwp_para_format_bytes_for_cli(&data, section, para, &props_json),
        "cell-char" => {
            let ctrl = parse_usize_cli(ctrl, "--ctrl");
            let cell_para = parse_usize_cli(cell_para, "--cell-para");
            let start = parse_usize_cli(start, "--start");
            let end = parse_usize_cli(end, "--end");
            if row.is_some() || col.is_some() {
                if cell.is_some() {
                    exit_cli_error("--cell과 --row/--col은 함께 사용할 수 없습니다.");
                }
                let row = parse_u16_cli(row, "--row");
                let col = parse_u16_cli(col, "--col");
                set_hwp_cell_char_format_at_bytes_for_cli(
                    &data,
                    section,
                    para,
                    ctrl,
                    row,
                    col,
                    cell_para,
                    start,
                    end,
                    &props_json,
                )
            } else {
                let cell = parse_usize_cli(cell, "--cell");
                set_hwp_cell_char_format_bytes_for_cli(
                    &data,
                    section,
                    para,
                    ctrl,
                    cell,
                    cell_para,
                    start,
                    end,
                    &props_json,
                )
            }
        }
        "cell-para" => {
            let ctrl = parse_usize_cli(ctrl, "--ctrl");
            let cell_para = parse_usize_cli(cell_para, "--cell-para");
            if row.is_some() || col.is_some() {
                if cell.is_some() {
                    exit_cli_error("--cell과 --row/--col은 함께 사용할 수 없습니다.");
                }
                let row = parse_u16_cli(row, "--row");
                let col = parse_u16_cli(col, "--col");
                set_hwp_cell_para_format_at_bytes_for_cli(
                    &data,
                    section,
                    para,
                    ctrl,
                    row,
                    col,
                    cell_para,
                    &props_json,
                )
            } else {
                let cell = parse_usize_cli(cell, "--cell");
                set_hwp_cell_para_format_bytes_for_cli(
                    &data,
                    section,
                    para,
                    ctrl,
                    cell,
                    cell_para,
                    &props_json,
                )
            }
        }
        _ => Err(format!("지원하지 않는 서식 설정 명령: {}", kind)),
    }
    .unwrap_or_else(|e| exit_cli_error(&e));

    write_hwp_cli_output(&output, &result.bytes).unwrap_or_else(|e| exit_cli_error(&e));
    println!(
        "{}",
        serde_json::json!({
            "ok": true,
            "path": output,
            "bytes": result.bytes.len(),
            "details": result.details,
            "pageCountBefore": result.page_count_before,
            "pageCountAfter": result.page_count_after,
        })
    );
}

fn list_styles_cli(args: &[String]) {
    if args.is_empty() {
        exit_cli_error("사용법: rhwp list-styles <파일.hwp>");
    }
    let input = args[0].clone();
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let result = list_hwp_styles_json_for_cli(&data).unwrap_or_else(|e| exit_cli_error(&e));
    println!("{}", result);
}

fn apply_style_cli(args: &[String], cell_style: bool) {
    if args.is_empty() {
        exit_cli_error("사용법: rhwp apply-style|apply-cell-style <파일.hwp> --section N --para N [--ctrl N (--cell N|--row R --col C) --cell-para N] (--style-id N|--style-name <이름>) -o <출력.hwp>");
    }
    let input = args[0].clone();
    let mut section: Option<String> = None;
    let mut para: Option<String> = None;
    let mut ctrl: Option<String> = None;
    let mut cell: Option<String> = None;
    let mut row: Option<String> = None;
    let mut col: Option<String> = None;
    let mut cell_para: Option<String> = Some("0".to_string());
    let mut style_id: Option<String> = None;
    let mut style_name: Option<String> = None;
    let mut output_path: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--section" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--section 뒤에 값이 필요합니다.");
                }
                section = Some(args[i].clone());
            }
            "--para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--para 뒤에 값이 필요합니다.");
                }
                para = Some(args[i].clone());
            }
            "--ctrl" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--ctrl 뒤에 값이 필요합니다.");
                }
                ctrl = Some(args[i].clone());
            }
            "--cell" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--cell 뒤에 값이 필요합니다.");
                }
                cell = Some(args[i].clone());
            }
            "--row" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--row 뒤에 값이 필요합니다.");
                }
                row = Some(args[i].clone());
            }
            "--col" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--col 뒤에 값이 필요합니다.");
                }
                col = Some(args[i].clone());
            }
            "--cell-para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--cell-para 뒤에 값이 필요합니다.");
                }
                cell_para = Some(args[i].clone());
            }
            "--style-id" | "--style" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--style-id 뒤에 값이 필요합니다.");
                }
                style_id = Some(args[i].clone());
            }
            "--style-name" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--style-name 뒤에 값이 필요합니다.");
                }
                style_name = Some(args[i].clone());
            }
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("-o/--output 뒤에 경로가 필요합니다.");
                }
                output_path = Some(args[i].clone());
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }

    let section = parse_usize_cli(section, "--section");
    let para = parse_usize_cli(para, "--para");
    let output = output_path.unwrap_or_else(|| input.clone());
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let style_id = match style_id {
        Some(raw) => raw
            .parse::<usize>()
            .or_else(|_| resolve_hwp_style_id_for_cli(&data, None, Some(&raw)))
            .unwrap_or_else(|e| exit_cli_error(&e)),
        None => resolve_hwp_style_id_for_cli(&data, None, style_name.as_deref())
            .unwrap_or_else(|e| exit_cli_error(&e)),
    };

    let result = if cell_style {
        let ctrl = parse_usize_cli(ctrl, "--ctrl");
        let cell_para = parse_usize_cli(cell_para, "--cell-para");
        if row.is_some() || col.is_some() {
            if cell.is_some() {
                exit_cli_error("--cell과 --row/--col은 함께 사용할 수 없습니다.");
            }
            let row = parse_u16_cli(row, "--row");
            let col = parse_u16_cli(col, "--col");
            apply_hwp_cell_style_at_bytes_for_cli(
                &data, section, para, ctrl, row, col, cell_para, style_id,
            )
        } else {
            let cell = parse_usize_cli(cell, "--cell");
            apply_hwp_cell_style_bytes_for_cli(
                &data, section, para, ctrl, cell, cell_para, style_id,
            )
        }
    } else {
        apply_hwp_style_bytes_for_cli(&data, section, para, style_id)
    }
    .unwrap_or_else(|e| exit_cli_error(&e));

    write_hwp_cli_output(&output, &result.bytes).unwrap_or_else(|e| exit_cli_error(&e));
    println!(
        "{}",
        serde_json::json!({
            "ok": true,
            "path": output,
            "bytes": result.bytes.len(),
            "details": result.details,
            "pageCountBefore": result.page_count_before,
            "pageCountAfter": result.page_count_after,
        })
    );
}

fn get_page_settings_cli(args: &[String], kind: &str) {
    if args.is_empty() {
        exit_cli_error("사용법: rhwp get-page|section-* <파일.hwp> --section N");
    }
    let input = args[0].clone();
    let mut section: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--section" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--section 뒤에 값이 필요합니다.");
                }
                section = Some(args[i].clone());
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }

    let section = parse_usize_cli(section, "--section");
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let result = match kind {
        "page-def" => get_hwp_page_def_json_for_cli(&data, section),
        "section-def" => get_hwp_section_def_json_for_cli(&data, section),
        "page-border-fill" => get_hwp_page_border_fill_json_for_cli(&data, section),
        _ => Err(format!("지원하지 않는 페이지 설정 조회 명령: {}", kind)),
    }
    .unwrap_or_else(|e| exit_cli_error(&e));

    println!("{}", result);
}

fn set_page_settings_cli(args: &[String], kind: &str) {
    if args.is_empty() {
        exit_cli_error("사용법: rhwp set-page|section-* <파일.hwp> --section N --json <설정JSON> -o <출력.hwp>");
    }
    let input = args[0].clone();
    let mut section: Option<String> = None;
    let mut inline_json: Option<String> = None;
    let mut json_file: Option<String> = None;
    let mut output_path: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--section" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--section 뒤에 값이 필요합니다.");
                }
                section = Some(args[i].clone());
            }
            "--json" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--json 뒤에 JSON 문자열이 필요합니다.");
                }
                inline_json = Some(args[i].clone());
            }
            "--json-file" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--json-file 뒤에 경로가 필요합니다.");
                }
                json_file = Some(args[i].clone());
            }
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("-o/--output 뒤에 경로가 필요합니다.");
                }
                output_path = Some(args[i].clone());
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }

    let section = parse_usize_cli(section, "--section");
    let props_json =
        read_json_argument(inline_json, json_file).unwrap_or_else(|e| exit_cli_error(&e));
    let output = output_path.unwrap_or_else(|| input.clone());
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let result = match kind {
        "page-def" => set_hwp_page_def_bytes_for_cli(&data, section, &props_json),
        "section-def" => set_hwp_section_def_bytes_for_cli(&data, section, &props_json),
        "page-border-fill" => set_hwp_page_border_fill_bytes_for_cli(&data, section, &props_json),
        _ => Err(format!("지원하지 않는 페이지 설정 명령: {}", kind)),
    }
    .unwrap_or_else(|e| exit_cli_error(&e));

    write_hwp_cli_output(&output, &result.bytes).unwrap_or_else(|e| exit_cli_error(&e));
    println!(
        "{}",
        serde_json::json!({
            "ok": true,
            "path": output,
            "bytes": result.bytes.len(),
            "details": result.details,
            "pageCountBefore": result.page_count_before,
            "pageCountAfter": result.page_count_after,
        })
    );
}

fn insert_picture_cli(args: &[String]) {
    if args.is_empty() {
        exit_cli_error("사용법: rhwp insert-picture <파일.hwp> --section N --para N --offset N --image <이미지> --width N --height N [--cell-path JSON|--table-ctrl N --row R --col C [--cell-para N]] -o <출력.hwp>");
    }
    let input = args[0].clone();
    let mut section: Option<String> = None;
    let mut para: Option<String> = None;
    let mut offset: Option<String> = None;
    let mut image_path: Option<String> = None;
    let mut width: Option<String> = None;
    let mut height: Option<String> = None;
    let mut natural_width: Option<String> = None;
    let mut natural_height: Option<String> = None;
    let mut extension: Option<String> = None;
    let mut description = String::new();
    let mut cell_path: Option<String> = None;
    let mut table_ctrl: Option<String> = None;
    let mut row: Option<String> = None;
    let mut col: Option<String> = None;
    let mut cell_para: Option<String> = None;
    let mut paper_x: Option<String> = None;
    let mut paper_y: Option<String> = None;
    let mut inline_in_cell = false;
    let mut output_path: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--inline" => {
                inline_in_cell = true;
            }
            "--section" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--section 뒤에 값이 필요합니다.");
                }
                section = Some(args[i].clone());
            }
            "--para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--para 뒤에 값이 필요합니다.");
                }
                para = Some(args[i].clone());
            }
            "--offset" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--offset 뒤에 값이 필요합니다.");
                }
                offset = Some(args[i].clone());
            }
            "--image" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--image 뒤에 경로가 필요합니다.");
                }
                image_path = Some(args[i].clone());
            }
            "--width" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--width 뒤에 값이 필요합니다.");
                }
                width = Some(args[i].clone());
            }
            "--height" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--height 뒤에 값이 필요합니다.");
                }
                height = Some(args[i].clone());
            }
            "--natural-width" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--natural-width 뒤에 값이 필요합니다.");
                }
                natural_width = Some(args[i].clone());
            }
            "--natural-height" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--natural-height 뒤에 값이 필요합니다.");
                }
                natural_height = Some(args[i].clone());
            }
            "--extension" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--extension 뒤에 값이 필요합니다.");
                }
                extension = Some(args[i].clone());
            }
            "--description" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--description 뒤에 텍스트가 필요합니다.");
                }
                description = args[i].clone();
            }
            "--cell-path" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--cell-path 뒤에 JSON 문자열이 필요합니다.");
                }
                cell_path = Some(args[i].clone());
            }
            "--cell-path-file" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--cell-path-file 뒤에 경로가 필요합니다.");
                }
                cell_path = Some(fs::read_to_string(&args[i]).unwrap_or_else(|e| {
                    exit_cli_error(&format!("cellPath 파일 읽기 실패: {}", e))
                }));
            }
            "--table-ctrl" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--table-ctrl 뒤에 값이 필요합니다.");
                }
                table_ctrl = Some(args[i].clone());
            }
            "--row" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--row 뒤에 값이 필요합니다.");
                }
                row = Some(args[i].clone());
            }
            "--col" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--col 뒤에 값이 필요합니다.");
                }
                col = Some(args[i].clone());
            }
            "--cell-para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--cell-para 뒤에 값이 필요합니다.");
                }
                cell_para = Some(args[i].clone());
            }
            "--paper-x" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--paper-x 뒤에 값이 필요합니다.");
                }
                paper_x = Some(args[i].clone());
            }
            "--paper-y" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--paper-y 뒤에 값이 필요합니다.");
                }
                paper_y = Some(args[i].clone());
            }
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("-o/--output 뒤에 경로가 필요합니다.");
                }
                output_path = Some(args[i].clone());
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }

    let section = parse_usize_cli(section, "--section");
    let para = parse_usize_cli(para, "--para");
    let offset = parse_usize_cli(offset, "--offset");
    let image_path = image_path.unwrap_or_else(|| exit_cli_error("--image <이미지>가 필요합니다."));
    let width = parse_u32_cli(width, "--width");
    let height = parse_u32_cli(height, "--height");
    let natural_width = natural_width
        .map(|v| parse_u32_cli(Some(v), "--natural-width"))
        .unwrap_or_else(|| (width / 75).max(1));
    let natural_height = natural_height
        .map(|v| parse_u32_cli(Some(v), "--natural-height"))
        .unwrap_or_else(|| (height / 75).max(1));
    let extension = extension.unwrap_or_else(|| {
        Path::new(&image_path)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("png")
            .to_ascii_lowercase()
    });
    let paper_x = paper_x.map(|v| parse_i32_cli(Some(v), "--paper-x"));
    let paper_y = paper_y.map(|v| parse_i32_cli(Some(v), "--paper-y"));
    let cell_location = parse_table_cell_location_cli(&cell_path, table_ctrl, row, col, cell_para);
    let output = output_path.unwrap_or_else(|| input.clone());
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let image_data = fs::read(&image_path).unwrap_or_else(|e| {
        exit_cli_error(&format!("이미지 파일 읽기 실패 - {}: {}", image_path, e))
    });

    let result = if let Some((table_ctrl, row, col, cell_para)) = cell_location {
        if inline_in_cell {
            insert_hwp_cell_picture_inline_bytes_for_cli(
                &data,
                section,
                para,
                table_ctrl,
                row,
                col,
                cell_para,
                offset,
                &image_data,
                width,
                height,
                natural_width,
                natural_height,
                &extension,
                &description,
            )
        } else {
            insert_hwp_cell_picture_at_bytes_for_cli(
                &data,
                section,
                para,
                table_ctrl,
                row,
                col,
                cell_para,
                offset,
                &image_data,
                width,
                height,
                natural_width,
                natural_height,
                &extension,
                &description,
                paper_x,
                paper_y,
            )
        }
    } else if inline_in_cell {
        exit_cli_error("--inline은 --table-ctrl/--row/--col 셀 지정과 함께 사용해야 합니다.");
    } else {
        let cell_path = cell_path.unwrap_or_else(|| "[]".to_string());
        insert_hwp_picture_bytes_for_cli(
            &data,
            section,
            para,
            offset,
            &cell_path,
            &image_data,
            width,
            height,
            natural_width,
            natural_height,
            &extension,
            &description,
            paper_x,
            paper_y,
        )
    }
    .unwrap_or_else(|e| exit_cli_error(&e));

    write_hwp_cli_output(&output, &result.bytes).unwrap_or_else(|e| exit_cli_error(&e));
    println!(
        "{}",
        serde_json::json!({
            "ok": true,
            "path": output,
            "bytes": result.bytes.len(),
            "paraIdx": result.para_idx,
            "controlIdx": result.control_idx,
            "details": result.details,
            "pageCountBefore": result.page_count_before,
            "pageCountAfter": result.page_count_after,
        })
    );
}

fn create_shape_cli(args: &[String]) {
    if args.is_empty() {
        exit_cli_error("사용법: rhwp create-shape <파일.hwp> --section N --para N [--cell-path JSON|--table-ctrl N --row R --col C [--cell-para N]] --offset N --width N --height N [--shape-type TYPE] -o <출력.hwp>");
    }
    let input = args[0].clone();
    let mut section: Option<String> = None;
    let mut para: Option<String> = None;
    let mut offset: Option<String> = None;
    let mut width: Option<String> = None;
    let mut height: Option<String> = None;
    let mut horz_offset: Option<String> = Some("0".to_string());
    let mut vert_offset: Option<String> = Some("0".to_string());
    let mut treat_as_char = false;
    let mut text_wrap = "Square".to_string();
    let mut shape_type = "rectangle".to_string();
    let mut line_flip_x = false;
    let mut line_flip_y = false;
    let mut polygon_points = "[]".to_string();
    let mut cell_path: Option<String> = None;
    let mut table_ctrl: Option<String> = None;
    let mut row: Option<String> = None;
    let mut col: Option<String> = None;
    let mut cell_para: Option<String> = None;
    let mut output_path: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--section" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--section 뒤에 값이 필요합니다.");
                }
                section = Some(args[i].clone());
            }
            "--para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--para 뒤에 값이 필요합니다.");
                }
                para = Some(args[i].clone());
            }
            "--offset" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--offset 뒤에 값이 필요합니다.");
                }
                offset = Some(args[i].clone());
            }
            "--width" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--width 뒤에 값이 필요합니다.");
                }
                width = Some(args[i].clone());
            }
            "--height" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--height 뒤에 값이 필요합니다.");
                }
                height = Some(args[i].clone());
            }
            "--horz-offset" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--horz-offset 뒤에 값이 필요합니다.");
                }
                horz_offset = Some(args[i].clone());
            }
            "--vert-offset" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--vert-offset 뒤에 값이 필요합니다.");
                }
                vert_offset = Some(args[i].clone());
            }
            "--treat-as-char" => treat_as_char = true,
            "--floating" => treat_as_char = false,
            "--text-wrap" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--text-wrap 뒤에 값이 필요합니다.");
                }
                text_wrap = args[i].clone();
            }
            "--shape-type" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--shape-type 뒤에 값이 필요합니다.");
                }
                shape_type = args[i].clone();
            }
            "--line-flip-x" => line_flip_x = true,
            "--line-flip-y" => line_flip_y = true,
            "--polygon-points" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--polygon-points 뒤에 JSON 문자열이 필요합니다.");
                }
                polygon_points = args[i].clone();
            }
            "--polygon-points-file" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--polygon-points-file 뒤에 경로가 필요합니다.");
                }
                polygon_points = fs::read_to_string(&args[i]).unwrap_or_else(|e| {
                    exit_cli_error(&format!("polygonPoints 파일 읽기 실패: {}", e))
                });
            }
            "--cell-path" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--cell-path 뒤에 JSON 문자열이 필요합니다.");
                }
                cell_path = Some(args[i].clone());
            }
            "--cell-path-file" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--cell-path-file 뒤에 경로가 필요합니다.");
                }
                cell_path = Some(fs::read_to_string(&args[i]).unwrap_or_else(|e| {
                    exit_cli_error(&format!("cell-path 파일 읽기 실패: {}", e))
                }));
            }
            "--table-ctrl" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--table-ctrl 뒤에 값이 필요합니다.");
                }
                table_ctrl = Some(args[i].clone());
            }
            "--row" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--row 뒤에 값이 필요합니다.");
                }
                row = Some(args[i].clone());
            }
            "--col" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--col 뒤에 값이 필요합니다.");
                }
                col = Some(args[i].clone());
            }
            "--cell-para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--cell-para 뒤에 값이 필요합니다.");
                }
                cell_para = Some(args[i].clone());
            }
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("-o/--output 뒤에 경로가 필요합니다.");
                }
                output_path = Some(args[i].clone());
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }

    let section = parse_usize_cli(section, "--section");
    let para = parse_usize_cli(para, "--para");
    let offset = parse_usize_cli(offset, "--offset");
    let width = parse_u32_cli(width, "--width");
    let height = parse_u32_cli(height, "--height");
    let horz_offset = parse_u32_cli(horz_offset, "--horz-offset");
    let vert_offset = parse_u32_cli(vert_offset, "--vert-offset");
    let cell_location = parse_table_cell_location_cli(&cell_path, table_ctrl, row, col, cell_para);
    let output = output_path.unwrap_or_else(|| input.clone());
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let result = if let Some((table_ctrl, row, col, cell_para)) = cell_location {
        create_hwp_cell_shape_at_bytes_for_cli(
            &data,
            section,
            para,
            table_ctrl,
            row,
            col,
            cell_para,
            offset,
            width,
            height,
            horz_offset,
            vert_offset,
            treat_as_char,
            &text_wrap,
            &shape_type,
            line_flip_x,
            line_flip_y,
            &polygon_points,
        )
    } else if let Some(cell_path) = cell_path {
        create_hwp_cell_shape_bytes_for_cli(
            &data,
            section,
            para,
            offset,
            &cell_path,
            width,
            height,
            horz_offset,
            vert_offset,
            treat_as_char,
            &text_wrap,
            &shape_type,
            line_flip_x,
            line_flip_y,
            &polygon_points,
        )
    } else {
        create_hwp_shape_bytes_for_cli(
            &data,
            section,
            para,
            offset,
            width,
            height,
            horz_offset,
            vert_offset,
            treat_as_char,
            &text_wrap,
            &shape_type,
            line_flip_x,
            line_flip_y,
            &polygon_points,
        )
    }
    .unwrap_or_else(|e| exit_cli_error(&e));

    write_hwp_cli_output(&output, &result.bytes).unwrap_or_else(|e| exit_cli_error(&e));
    println!(
        "{}",
        serde_json::json!({
            "ok": true,
            "path": output,
            "bytes": result.bytes.len(),
            "paraIdx": result.para_idx,
            "controlIdx": result.control_idx,
            "details": result.details,
            "pageCountBefore": result.page_count_before,
            "pageCountAfter": result.page_count_after,
        })
    );
}

fn set_cell_shape_text_cli(args: &[String]) {
    if args.is_empty() {
        exit_cli_error("사용법: rhwp set-cell-shape-text <파일.hwp> --section N --para N [--cell-path JSON|--table-ctrl N --row R --col C [--cell-para N]] --ctrl N [--textbox-para N] --text <텍스트> -o <출력.hwp>");
    }
    let input = args[0].clone();
    let mut section: Option<String> = None;
    let mut para: Option<String> = None;
    let mut ctrl: Option<String> = None;
    let mut cell_path: Option<String> = None;
    let mut table_ctrl: Option<String> = None;
    let mut row: Option<String> = None;
    let mut col: Option<String> = None;
    let mut cell_para: Option<String> = None;
    let mut textbox_para: Option<String> = Some("0".to_string());
    let mut text: Option<String> = None;
    let mut output_path: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--section" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--section 뒤에 값이 필요합니다.");
                }
                section = Some(args[i].clone());
            }
            "--para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--para 뒤에 값이 필요합니다.");
                }
                para = Some(args[i].clone());
            }
            "--ctrl" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--ctrl 뒤에 값이 필요합니다.");
                }
                ctrl = Some(args[i].clone());
            }
            "--cell-path" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--cell-path 뒤에 JSON 문자열이 필요합니다.");
                }
                cell_path = Some(args[i].clone());
            }
            "--cell-path-file" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--cell-path-file 뒤에 경로가 필요합니다.");
                }
                cell_path = Some(fs::read_to_string(&args[i]).unwrap_or_else(|e| {
                    exit_cli_error(&format!("cell-path 파일 읽기 실패: {}", e))
                }));
            }
            "--table-ctrl" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--table-ctrl 뒤에 값이 필요합니다.");
                }
                table_ctrl = Some(args[i].clone());
            }
            "--row" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--row 뒤에 값이 필요합니다.");
                }
                row = Some(args[i].clone());
            }
            "--col" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--col 뒤에 값이 필요합니다.");
                }
                col = Some(args[i].clone());
            }
            "--cell-para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--cell-para 뒤에 값이 필요합니다.");
                }
                cell_para = Some(args[i].clone());
            }
            "--textbox-para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--textbox-para 뒤에 값이 필요합니다.");
                }
                textbox_para = Some(args[i].clone());
            }
            "--text" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--text 뒤에 텍스트가 필요합니다.");
                }
                text = Some(args[i].clone());
            }
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("-o/--output 뒤에 경로가 필요합니다.");
                }
                output_path = Some(args[i].clone());
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }

    let section = parse_usize_cli(section, "--section");
    let para = parse_usize_cli(para, "--para");
    let ctrl = parse_usize_cli(ctrl, "--ctrl");
    let textbox_para = parse_usize_cli(textbox_para, "--textbox-para");
    let text = text.unwrap_or_else(|| exit_cli_error("--text <텍스트>가 필요합니다."));
    let cell_location = parse_table_cell_location_cli(&cell_path, table_ctrl, row, col, cell_para);
    let output = output_path.unwrap_or_else(|| input.clone());
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let result = if let Some((table_ctrl, row, col, cell_para)) = cell_location {
        set_hwp_cell_shape_text_at_bytes_for_cli(
            &data,
            section,
            para,
            table_ctrl,
            row,
            col,
            cell_para,
            ctrl,
            textbox_para,
            &text,
        )
    } else if let Some(cell_path) = cell_path {
        set_hwp_cell_shape_text_bytes_for_cli(
            &data,
            section,
            para,
            &cell_path,
            ctrl,
            textbox_para,
            &text,
        )
    } else {
        Err(
            "set-cell-shape-text에는 --cell-path 또는 --table-ctrl/--row/--col이 필요합니다."
                .to_string(),
        )
    }
    .unwrap_or_else(|e| exit_cli_error(&e));

    write_hwp_cli_output(&output, &result.bytes).unwrap_or_else(|e| exit_cli_error(&e));
    println!(
        "{}",
        serde_json::json!({
            "ok": true,
            "path": output,
            "bytes": result.bytes.len(),
            "details": result.details,
            "pageCountBefore": result.page_count_before,
            "pageCountAfter": result.page_count_after,
        })
    );
}

fn set_cell_shape_format_cli(args: &[String], char_format: bool) {
    if args.is_empty() {
        if char_format {
            exit_cli_error("사용법: rhwp set-cell-shape-char-format <파일.hwp> --section N --para N [--cell-path JSON|--table-ctrl N --row R --col C [--cell-para N]] --ctrl N --textbox-para N --start N --end N --json <서식JSON> -o <출력.hwp>");
        }
        exit_cli_error("사용법: rhwp set-cell-shape-para-format <파일.hwp> --section N --para N [--cell-path JSON|--table-ctrl N --row R --col C [--cell-para N]] --ctrl N --textbox-para N --json <서식JSON> -o <출력.hwp>");
    }
    let input = args[0].clone();
    let mut section: Option<String> = None;
    let mut para: Option<String> = None;
    let mut ctrl: Option<String> = None;
    let mut cell_path: Option<String> = None;
    let mut table_ctrl: Option<String> = None;
    let mut row: Option<String> = None;
    let mut col: Option<String> = None;
    let mut cell_para: Option<String> = None;
    let mut textbox_para: Option<String> = Some("0".to_string());
    let mut start: Option<String> = None;
    let mut end: Option<String> = None;
    let mut inline_json: Option<String> = None;
    let mut json_file: Option<String> = None;
    let mut output_path: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--section" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--section 뒤에 값이 필요합니다.");
                }
                section = Some(args[i].clone());
            }
            "--para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--para 뒤에 값이 필요합니다.");
                }
                para = Some(args[i].clone());
            }
            "--ctrl" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--ctrl 뒤에 값이 필요합니다.");
                }
                ctrl = Some(args[i].clone());
            }
            "--cell-path" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--cell-path 뒤에 JSON 문자열이 필요합니다.");
                }
                cell_path = Some(args[i].clone());
            }
            "--cell-path-file" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--cell-path-file 뒤에 경로가 필요합니다.");
                }
                cell_path = Some(fs::read_to_string(&args[i]).unwrap_or_else(|e| {
                    exit_cli_error(&format!("cell-path 파일 읽기 실패: {}", e))
                }));
            }
            "--table-ctrl" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--table-ctrl 뒤에 값이 필요합니다.");
                }
                table_ctrl = Some(args[i].clone());
            }
            "--row" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--row 뒤에 값이 필요합니다.");
                }
                row = Some(args[i].clone());
            }
            "--col" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--col 뒤에 값이 필요합니다.");
                }
                col = Some(args[i].clone());
            }
            "--cell-para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--cell-para 뒤에 값이 필요합니다.");
                }
                cell_para = Some(args[i].clone());
            }
            "--textbox-para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--textbox-para 뒤에 값이 필요합니다.");
                }
                textbox_para = Some(args[i].clone());
            }
            "--start" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--start 뒤에 값이 필요합니다.");
                }
                start = Some(args[i].clone());
            }
            "--end" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--end 뒤에 값이 필요합니다.");
                }
                end = Some(args[i].clone());
            }
            "--json" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--json 뒤에 JSON 문자열이 필요합니다.");
                }
                inline_json = Some(args[i].clone());
            }
            "--json-file" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--json-file 뒤에 경로가 필요합니다.");
                }
                json_file = Some(args[i].clone());
            }
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("-o/--output 뒤에 경로가 필요합니다.");
                }
                output_path = Some(args[i].clone());
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }

    let section = parse_usize_cli(section, "--section");
    let para = parse_usize_cli(para, "--para");
    let ctrl = parse_usize_cli(ctrl, "--ctrl");
    let textbox_para = parse_usize_cli(textbox_para, "--textbox-para");
    let start = if char_format {
        Some(parse_usize_cli(start, "--start"))
    } else {
        None
    };
    let end = if char_format {
        Some(parse_usize_cli(end, "--end"))
    } else {
        None
    };
    let props_json =
        read_json_argument(inline_json, json_file).unwrap_or_else(|e| exit_cli_error(&e));
    let cell_location = parse_table_cell_location_cli(&cell_path, table_ctrl, row, col, cell_para);
    let output = output_path.unwrap_or_else(|| input.clone());
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));

    let result = if char_format {
        let start = start.expect("start parsed");
        let end = end.expect("end parsed");
        if let Some((table_ctrl, row, col, cell_para)) = cell_location {
            set_hwp_cell_shape_char_format_at_bytes_for_cli(
                &data,
                section,
                para,
                table_ctrl,
                row,
                col,
                cell_para,
                ctrl,
                textbox_para,
                start,
                end,
                &props_json,
            )
        } else if let Some(cell_path) = cell_path {
            set_hwp_cell_shape_char_format_bytes_for_cli(
                &data,
                section,
                para,
                &cell_path,
                ctrl,
                textbox_para,
                start,
                end,
                &props_json,
            )
        } else {
            Err("set-cell-shape-char-format에는 --cell-path 또는 --table-ctrl/--row/--col이 필요합니다.".to_string())
        }
    } else if let Some((table_ctrl, row, col, cell_para)) = cell_location {
        set_hwp_cell_shape_para_format_at_bytes_for_cli(
            &data,
            section,
            para,
            table_ctrl,
            row,
            col,
            cell_para,
            ctrl,
            textbox_para,
            &props_json,
        )
    } else if let Some(cell_path) = cell_path {
        set_hwp_cell_shape_para_format_bytes_for_cli(
            &data,
            section,
            para,
            &cell_path,
            ctrl,
            textbox_para,
            &props_json,
        )
    } else {
        Err("set-cell-shape-para-format에는 --cell-path 또는 --table-ctrl/--row/--col이 필요합니다.".to_string())
    }
    .unwrap_or_else(|e| exit_cli_error(&e));

    write_hwp_cli_output(&output, &result.bytes).unwrap_or_else(|e| exit_cli_error(&e));
    println!(
        "{}",
        serde_json::json!({
            "ok": true,
            "path": output,
            "bytes": result.bytes.len(),
            "details": result.details,
            "pageCountBefore": result.page_count_before,
            "pageCountAfter": result.page_count_after,
        })
    );
}

fn get_object_properties_cli(args: &[String], kind: &str) {
    if args.is_empty() {
        exit_cli_error(
            "사용법: rhwp get-picture|shape-properties <파일.hwp> --section N --para N [--cell-path JSON|--table-ctrl N --row R --col C [--cell-para N]] --ctrl N",
        );
    }
    let input = args[0].clone();
    let mut section: Option<String> = None;
    let mut para: Option<String> = None;
    let mut ctrl: Option<String> = None;
    let mut cell_path: Option<String> = None;
    let mut table_ctrl: Option<String> = None;
    let mut row: Option<String> = None;
    let mut col: Option<String> = None;
    let mut cell_para: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--section" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--section 뒤에 값이 필요합니다.");
                }
                section = Some(args[i].clone());
            }
            "--para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--para 뒤에 값이 필요합니다.");
                }
                para = Some(args[i].clone());
            }
            "--ctrl" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--ctrl 뒤에 값이 필요합니다.");
                }
                ctrl = Some(args[i].clone());
            }
            "--table-ctrl" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--table-ctrl 뒤에 값이 필요합니다.");
                }
                table_ctrl = Some(args[i].clone());
            }
            "--row" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--row 뒤에 값이 필요합니다.");
                }
                row = Some(args[i].clone());
            }
            "--col" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--col 뒤에 값이 필요합니다.");
                }
                col = Some(args[i].clone());
            }
            "--cell-para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--cell-para 뒤에 값이 필요합니다.");
                }
                cell_para = Some(args[i].clone());
            }
            "--cell-path" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--cell-path 뒤에 JSON 문자열이 필요합니다.");
                }
                cell_path = Some(args[i].clone());
            }
            "--cell-path-file" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--cell-path-file 뒤에 경로가 필요합니다.");
                }
                cell_path = Some(fs::read_to_string(&args[i]).unwrap_or_else(|e| {
                    exit_cli_error(&format!("cell-path 파일 읽기 실패: {}", e))
                }));
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }

    let section = parse_usize_cli(section, "--section");
    let para = parse_usize_cli(para, "--para");
    let ctrl = parse_usize_cli(ctrl, "--ctrl");
    let cell_location = parse_table_cell_location_cli(&cell_path, table_ctrl, row, col, cell_para);
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let result = match kind {
        "picture" => {
            if let Some((table_ctrl, row, col, cell_para)) = cell_location {
                get_hwp_cell_picture_properties_at_json_for_cli(
                    &data, section, para, table_ctrl, row, col, cell_para, ctrl,
                )
            } else if let Some(cell_path) = cell_path {
                get_hwp_cell_picture_properties_json_for_cli(&data, section, para, &cell_path, ctrl)
            } else {
                get_hwp_picture_properties_json_for_cli(&data, section, para, ctrl)
            }
        }
        "shape" => {
            if let Some((table_ctrl, row, col, cell_para)) = cell_location {
                get_hwp_cell_shape_properties_at_json_for_cli(
                    &data, section, para, table_ctrl, row, col, cell_para, ctrl,
                )
            } else if let Some(cell_path) = cell_path {
                get_hwp_cell_shape_properties_json_for_cli(&data, section, para, &cell_path, ctrl)
            } else {
                get_hwp_shape_properties_json_for_cli(&data, section, para, ctrl)
            }
        }
        _ => Err(format!("지원하지 않는 객체 속성 조회 명령: {}", kind)),
    }
    .unwrap_or_else(|e| exit_cli_error(&e));
    println!("{}", result);
}

fn set_object_properties_cli(args: &[String], kind: &str) {
    if args.is_empty() {
        exit_cli_error("사용법: rhwp set-picture|shape-properties <파일.hwp> --section N --para N [--cell-path JSON|--table-ctrl N --row R --col C [--cell-para N]] --ctrl N --json <속성JSON> -o <출력.hwp>");
    }
    let input = args[0].clone();
    let mut section: Option<String> = None;
    let mut para: Option<String> = None;
    let mut ctrl: Option<String> = None;
    let mut cell_path: Option<String> = None;
    let mut table_ctrl: Option<String> = None;
    let mut row: Option<String> = None;
    let mut col: Option<String> = None;
    let mut cell_para: Option<String> = None;
    let mut inline_json: Option<String> = None;
    let mut json_file: Option<String> = None;
    let mut output_path: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--section" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--section 뒤에 값이 필요합니다.");
                }
                section = Some(args[i].clone());
            }
            "--para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--para 뒤에 값이 필요합니다.");
                }
                para = Some(args[i].clone());
            }
            "--ctrl" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--ctrl 뒤에 값이 필요합니다.");
                }
                ctrl = Some(args[i].clone());
            }
            "--table-ctrl" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--table-ctrl 뒤에 값이 필요합니다.");
                }
                table_ctrl = Some(args[i].clone());
            }
            "--row" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--row 뒤에 값이 필요합니다.");
                }
                row = Some(args[i].clone());
            }
            "--col" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--col 뒤에 값이 필요합니다.");
                }
                col = Some(args[i].clone());
            }
            "--cell-para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--cell-para 뒤에 값이 필요합니다.");
                }
                cell_para = Some(args[i].clone());
            }
            "--cell-path" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--cell-path 뒤에 JSON 문자열이 필요합니다.");
                }
                cell_path = Some(args[i].clone());
            }
            "--cell-path-file" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--cell-path-file 뒤에 경로가 필요합니다.");
                }
                cell_path = Some(fs::read_to_string(&args[i]).unwrap_or_else(|e| {
                    exit_cli_error(&format!("cell-path 파일 읽기 실패: {}", e))
                }));
            }
            "--json" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--json 뒤에 JSON 문자열이 필요합니다.");
                }
                inline_json = Some(args[i].clone());
            }
            "--json-file" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--json-file 뒤에 경로가 필요합니다.");
                }
                json_file = Some(args[i].clone());
            }
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("-o/--output 뒤에 경로가 필요합니다.");
                }
                output_path = Some(args[i].clone());
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }

    let section = parse_usize_cli(section, "--section");
    let para = parse_usize_cli(para, "--para");
    let ctrl = parse_usize_cli(ctrl, "--ctrl");
    let cell_location = parse_table_cell_location_cli(&cell_path, table_ctrl, row, col, cell_para);
    let props_json =
        read_json_argument(inline_json, json_file).unwrap_or_else(|e| exit_cli_error(&e));
    let output = output_path.unwrap_or_else(|| input.clone());
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let result = match kind {
        "picture" => {
            if let Some((table_ctrl, row, col, cell_para)) = cell_location {
                set_hwp_cell_picture_properties_at_bytes_for_cli(
                    &data,
                    section,
                    para,
                    table_ctrl,
                    row,
                    col,
                    cell_para,
                    ctrl,
                    &props_json,
                )
            } else if let Some(cell_path) = cell_path {
                set_hwp_cell_picture_properties_bytes_for_cli(
                    &data,
                    section,
                    para,
                    &cell_path,
                    ctrl,
                    &props_json,
                )
            } else {
                set_hwp_picture_properties_bytes_for_cli(&data, section, para, ctrl, &props_json)
            }
        }
        "shape" => {
            if let Some((table_ctrl, row, col, cell_para)) = cell_location {
                set_hwp_cell_shape_properties_at_bytes_for_cli(
                    &data,
                    section,
                    para,
                    table_ctrl,
                    row,
                    col,
                    cell_para,
                    ctrl,
                    &props_json,
                )
            } else if let Some(cell_path) = cell_path {
                set_hwp_cell_shape_properties_bytes_for_cli(
                    &data,
                    section,
                    para,
                    &cell_path,
                    ctrl,
                    &props_json,
                )
            } else {
                set_hwp_shape_properties_bytes_for_cli(&data, section, para, ctrl, &props_json)
            }
        }
        _ => Err(format!("지원하지 않는 객체 속성 설정 명령: {}", kind)),
    }
    .unwrap_or_else(|e| exit_cli_error(&e));

    write_hwp_cli_output(&output, &result.bytes).unwrap_or_else(|e| exit_cli_error(&e));
    println!(
        "{}",
        serde_json::json!({
            "ok": true,
            "path": output,
            "bytes": result.bytes.len(),
            "details": result.details,
            "pageCountBefore": result.page_count_before,
            "pageCountAfter": result.page_count_after,
        })
    );
}

fn delete_object_cli(args: &[String], kind: &str) {
    if args.is_empty() {
        exit_cli_error("사용법: rhwp delete-picture|shape <파일.hwp> --section N --para N [--cell-path JSON|--table-ctrl N --row R --col C [--cell-para N]] --ctrl N -o <출력.hwp>");
    }
    let input = args[0].clone();
    let mut section: Option<String> = None;
    let mut para: Option<String> = None;
    let mut ctrl: Option<String> = None;
    let mut cell_path: Option<String> = None;
    let mut table_ctrl: Option<String> = None;
    let mut row: Option<String> = None;
    let mut col: Option<String> = None;
    let mut cell_para: Option<String> = None;
    let mut output_path: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--section" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--section 뒤에 값이 필요합니다.");
                }
                section = Some(args[i].clone());
            }
            "--para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--para 뒤에 값이 필요합니다.");
                }
                para = Some(args[i].clone());
            }
            "--ctrl" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--ctrl 뒤에 값이 필요합니다.");
                }
                ctrl = Some(args[i].clone());
            }
            "--table-ctrl" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--table-ctrl 뒤에 값이 필요합니다.");
                }
                table_ctrl = Some(args[i].clone());
            }
            "--row" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--row 뒤에 값이 필요합니다.");
                }
                row = Some(args[i].clone());
            }
            "--col" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--col 뒤에 값이 필요합니다.");
                }
                col = Some(args[i].clone());
            }
            "--cell-para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--cell-para 뒤에 값이 필요합니다.");
                }
                cell_para = Some(args[i].clone());
            }
            "--cell-path" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--cell-path 뒤에 JSON 문자열이 필요합니다.");
                }
                cell_path = Some(args[i].clone());
            }
            "--cell-path-file" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--cell-path-file 뒤에 경로가 필요합니다.");
                }
                cell_path = Some(fs::read_to_string(&args[i]).unwrap_or_else(|e| {
                    exit_cli_error(&format!("cell-path 파일 읽기 실패: {}", e))
                }));
            }
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("-o/--output 뒤에 경로가 필요합니다.");
                }
                output_path = Some(args[i].clone());
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }

    let section = parse_usize_cli(section, "--section");
    let para = parse_usize_cli(para, "--para");
    let ctrl = parse_usize_cli(ctrl, "--ctrl");
    let cell_location = parse_table_cell_location_cli(&cell_path, table_ctrl, row, col, cell_para);
    let output = output_path.unwrap_or_else(|| input.clone());
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let result = match kind {
        "picture" => {
            if let Some((table_ctrl, row, col, cell_para)) = cell_location {
                delete_hwp_cell_picture_at_bytes_for_cli(
                    &data, section, para, table_ctrl, row, col, cell_para, ctrl,
                )
            } else if let Some(cell_path) = cell_path {
                delete_hwp_cell_picture_bytes_for_cli(&data, section, para, &cell_path, ctrl)
            } else {
                delete_hwp_picture_bytes_for_cli(&data, section, para, ctrl)
            }
        }
        "shape" => {
            if let Some((table_ctrl, row, col, cell_para)) = cell_location {
                delete_hwp_cell_shape_at_bytes_for_cli(
                    &data, section, para, table_ctrl, row, col, cell_para, ctrl,
                )
            } else if let Some(cell_path) = cell_path {
                delete_hwp_cell_shape_bytes_for_cli(&data, section, para, &cell_path, ctrl)
            } else {
                delete_hwp_shape_bytes_for_cli(&data, section, para, ctrl)
            }
        }
        _ => Err(format!("지원하지 않는 객체 삭제 명령: {}", kind)),
    }
    .unwrap_or_else(|e| exit_cli_error(&e));

    write_hwp_cli_output(&output, &result.bytes).unwrap_or_else(|e| exit_cli_error(&e));
    println!(
        "{}",
        serde_json::json!({
            "ok": true,
            "path": output,
            "bytes": result.bytes.len(),
            "details": result.details,
            "pageCountBefore": result.page_count_before,
            "pageCountAfter": result.page_count_after,
        })
    );
}

fn change_shape_z_order_cli(args: &[String]) {
    if args.is_empty() {
        exit_cli_error("사용법: rhwp change-shape-z-order <파일.hwp> --section N --para N --ctrl N --operation front|back|forward|backward -o <출력.hwp>");
    }
    let input = args[0].clone();
    let mut section: Option<String> = None;
    let mut para: Option<String> = None;
    let mut ctrl: Option<String> = None;
    let mut operation: Option<String> = None;
    let mut output_path: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--section" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--section 뒤에 값이 필요합니다.");
                }
                section = Some(args[i].clone());
            }
            "--para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--para 뒤에 값이 필요합니다.");
                }
                para = Some(args[i].clone());
            }
            "--ctrl" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--ctrl 뒤에 값이 필요합니다.");
                }
                ctrl = Some(args[i].clone());
            }
            "--operation" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--operation 뒤에 값이 필요합니다.");
                }
                operation = Some(args[i].clone());
            }
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("-o/--output 뒤에 경로가 필요합니다.");
                }
                output_path = Some(args[i].clone());
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }

    let section = parse_usize_cli(section, "--section");
    let para = parse_usize_cli(para, "--para");
    let ctrl = parse_usize_cli(ctrl, "--ctrl");
    let operation = operation.unwrap_or_else(|| exit_cli_error("--operation 값이 필요합니다."));
    let output = output_path.unwrap_or_else(|| input.clone());
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let result = change_hwp_shape_z_order_bytes_for_cli(&data, section, para, ctrl, &operation)
        .unwrap_or_else(|e| exit_cli_error(&e));

    write_hwp_cli_output(&output, &result.bytes).unwrap_or_else(|e| exit_cli_error(&e));
    println!(
        "{}",
        serde_json::json!({
            "ok": true,
            "path": output,
            "bytes": result.bytes.len(),
            "details": result.details,
            "pageCountBefore": result.page_count_before,
            "pageCountAfter": result.page_count_after,
        })
    );
}

fn group_shapes_cli(args: &[String]) {
    if args.is_empty() {
        exit_cli_error(
            "사용법: rhwp group-shapes <파일.hwp> --section N --targets <대상JSON> -o <출력.hwp>",
        );
    }
    let input = args[0].clone();
    let mut section: Option<String> = None;
    let mut inline_targets: Option<String> = None;
    let mut targets_file: Option<String> = None;
    let mut output_path: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--section" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--section 뒤에 값이 필요합니다.");
                }
                section = Some(args[i].clone());
            }
            "--targets" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--targets 뒤에 JSON 문자열이 필요합니다.");
                }
                inline_targets = Some(args[i].clone());
            }
            "--targets-file" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--targets-file 뒤에 경로가 필요합니다.");
                }
                targets_file = Some(args[i].clone());
            }
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("-o/--output 뒤에 경로가 필요합니다.");
                }
                output_path = Some(args[i].clone());
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }

    let section = parse_usize_cli(section, "--section");
    let targets_json =
        read_json_argument(inline_targets, targets_file).unwrap_or_else(|e| exit_cli_error(&e));
    let output = output_path.unwrap_or_else(|| input.clone());
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let result = group_hwp_shapes_bytes_for_cli(&data, section, &targets_json)
        .unwrap_or_else(|e| exit_cli_error(&e));

    write_hwp_cli_output(&output, &result.bytes).unwrap_or_else(|e| exit_cli_error(&e));
    println!(
        "{}",
        serde_json::json!({
            "ok": true,
            "path": output,
            "bytes": result.bytes.len(),
            "paraIdx": result.para_idx,
            "controlIdx": result.control_idx,
            "details": result.details,
            "pageCountBefore": result.page_count_before,
            "pageCountAfter": result.page_count_after,
        })
    );
}

fn ungroup_shape_cli(args: &[String]) {
    if args.is_empty() {
        exit_cli_error(
            "사용법: rhwp ungroup-shape <파일.hwp> --section N --para N --ctrl N -o <출력.hwp>",
        );
    }
    let input = args[0].clone();
    let mut section: Option<String> = None;
    let mut para: Option<String> = None;
    let mut ctrl: Option<String> = None;
    let mut output_path: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--section" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--section 뒤에 값이 필요합니다.");
                }
                section = Some(args[i].clone());
            }
            "--para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--para 뒤에 값이 필요합니다.");
                }
                para = Some(args[i].clone());
            }
            "--ctrl" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--ctrl 뒤에 값이 필요합니다.");
                }
                ctrl = Some(args[i].clone());
            }
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("-o/--output 뒤에 경로가 필요합니다.");
                }
                output_path = Some(args[i].clone());
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }

    let section = parse_usize_cli(section, "--section");
    let para = parse_usize_cli(para, "--para");
    let ctrl = parse_usize_cli(ctrl, "--ctrl");
    let output = output_path.unwrap_or_else(|| input.clone());
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let result = ungroup_hwp_shape_bytes_for_cli(&data, section, para, ctrl)
        .unwrap_or_else(|e| exit_cli_error(&e));

    write_hwp_cli_output(&output, &result.bytes).unwrap_or_else(|e| exit_cli_error(&e));
    println!(
        "{}",
        serde_json::json!({
            "ok": true,
            "path": output,
            "bytes": result.bytes.len(),
            "details": result.details,
            "pageCountBefore": result.page_count_before,
            "pageCountAfter": result.page_count_after,
        })
    );
}

fn parse_header_footer_common_cli(
    section: Option<String>,
    kind: Option<String>,
    apply_to: Option<String>,
) -> (usize, bool, u8) {
    let section = parse_usize_cli(section, "--section");
    let kind = kind.unwrap_or_else(|| exit_cli_error("--kind header|footer 값이 필요합니다."));
    let is_header = parse_header_footer_kind_for_cli(&kind).unwrap_or_else(|e| exit_cli_error(&e));
    let apply_to = apply_to
        .unwrap_or_else(|| exit_cli_error("--apply-to both|even|odd 또는 0|1|2 값이 필요합니다."));
    let apply_to =
        parse_header_footer_apply_to_for_cli(&apply_to).unwrap_or_else(|e| exit_cli_error(&e));
    (section, is_header, apply_to)
}

fn print_hwp_edit_cli_result(output: String, result: HwpEditCliResult) {
    println!(
        "{}",
        serde_json::json!({
            "ok": true,
            "path": output,
            "bytes": result.bytes.len(),
            "details": result.details,
            "pageCountBefore": result.page_count_before,
            "pageCountAfter": result.page_count_after,
        })
    );
}

fn get_header_footer_cli(args: &[String]) {
    if args.is_empty() {
        exit_cli_error("사용법: rhwp get-header-footer <파일.hwp> --section N --kind header|footer --apply-to both|even|odd");
    }
    let input = args[0].clone();
    let mut section: Option<String> = None;
    let mut kind: Option<String> = None;
    let mut apply_to: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--section" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--section 뒤에 값이 필요합니다.");
                }
                section = Some(args[i].clone());
            }
            "--kind" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--kind 뒤에 header 또는 footer가 필요합니다.");
                }
                kind = Some(args[i].clone());
            }
            "--apply-to" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--apply-to 뒤에 both/even/odd 값이 필요합니다.");
                }
                apply_to = Some(args[i].clone());
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }

    let (section, is_header, apply_to) = parse_header_footer_common_cli(section, kind, apply_to);
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let result = get_hwp_header_footer_json_for_cli(&data, section, is_header, apply_to)
        .unwrap_or_else(|e| exit_cli_error(&e));
    println!("{}", result);
}

fn list_header_footer_cli(args: &[String]) {
    if args.is_empty() {
        exit_cli_error("사용법: rhwp list-header-footer <파일.hwp> [--section N --kind header|footer --apply-to both|even|odd]");
    }
    let input = args[0].clone();
    let mut section: Option<String> = Some("0".to_string());
    let mut kind: Option<String> = Some("header".to_string());
    let mut apply_to: Option<String> = Some("both".to_string());

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--section" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--section 뒤에 값이 필요합니다.");
                }
                section = Some(args[i].clone());
            }
            "--kind" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--kind 뒤에 header 또는 footer가 필요합니다.");
                }
                kind = Some(args[i].clone());
            }
            "--apply-to" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--apply-to 뒤에 both/even/odd 값이 필요합니다.");
                }
                apply_to = Some(args[i].clone());
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }

    let (section, is_header, apply_to) = parse_header_footer_common_cli(section, kind, apply_to);
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let result = list_hwp_header_footer_json_for_cli(&data, section, is_header, apply_to)
        .unwrap_or_else(|e| exit_cli_error(&e));
    println!("{}", result);
}

fn header_footer_simple_edit_cli(args: &[String], action: &str) {
    if args.is_empty() {
        exit_cli_error("사용법: rhwp create|delete-header-footer <파일.hwp> --section N --kind header|footer --apply-to both|even|odd -o <출력.hwp>");
    }
    let input = args[0].clone();
    let mut section: Option<String> = None;
    let mut kind: Option<String> = None;
    let mut apply_to: Option<String> = None;
    let mut output_path: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--section" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--section 뒤에 값이 필요합니다.");
                }
                section = Some(args[i].clone());
            }
            "--kind" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--kind 뒤에 header 또는 footer가 필요합니다.");
                }
                kind = Some(args[i].clone());
            }
            "--apply-to" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--apply-to 뒤에 both/even/odd 값이 필요합니다.");
                }
                apply_to = Some(args[i].clone());
            }
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("-o/--output 뒤에 경로가 필요합니다.");
                }
                output_path = Some(args[i].clone());
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }

    let (section, is_header, apply_to) = parse_header_footer_common_cli(section, kind, apply_to);
    let output = output_path.unwrap_or_else(|| input.clone());
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let result = match action {
        "create" => create_hwp_header_footer_bytes_for_cli(&data, section, is_header, apply_to),
        "delete" => delete_hwp_header_footer_bytes_for_cli(&data, section, is_header, apply_to),
        _ => Err(format!("지원하지 않는 머리말/꼬리말 작업: {}", action)),
    }
    .unwrap_or_else(|e| exit_cli_error(&e));

    write_hwp_cli_output(&output, &result.bytes).unwrap_or_else(|e| exit_cli_error(&e));
    print_hwp_edit_cli_result(output, result);
}

fn header_footer_text_edit_cli(args: &[String], action: &str) {
    if args.is_empty() {
        exit_cli_error("사용법: rhwp insert|delete-header-footer-text <파일.hwp> --section N --kind header|footer --apply-to both|even|odd --hf-para N --offset N (--text <텍스트>|--count N) -o <출력.hwp>");
    }
    let input = args[0].clone();
    let mut section: Option<String> = None;
    let mut kind: Option<String> = None;
    let mut apply_to: Option<String> = None;
    let mut hf_para: Option<String> = None;
    let mut offset: Option<String> = None;
    let mut count: Option<String> = None;
    let mut inline_text: Option<String> = None;
    let mut text_file: Option<String> = None;
    let mut output_path: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--section" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--section 뒤에 값이 필요합니다.");
                }
                section = Some(args[i].clone());
            }
            "--kind" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--kind 뒤에 header 또는 footer가 필요합니다.");
                }
                kind = Some(args[i].clone());
            }
            "--apply-to" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--apply-to 뒤에 both/even/odd 값이 필요합니다.");
                }
                apply_to = Some(args[i].clone());
            }
            "--hf-para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--hf-para 뒤에 값이 필요합니다.");
                }
                hf_para = Some(args[i].clone());
            }
            "--offset" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--offset 뒤에 값이 필요합니다.");
                }
                offset = Some(args[i].clone());
            }
            "--count" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--count 뒤에 값이 필요합니다.");
                }
                count = Some(args[i].clone());
            }
            "--text" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--text 뒤에 텍스트가 필요합니다.");
                }
                inline_text = Some(args[i].clone());
            }
            "--text-file" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--text-file 뒤에 경로가 필요합니다.");
                }
                text_file = Some(args[i].clone());
            }
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("-o/--output 뒤에 경로가 필요합니다.");
                }
                output_path = Some(args[i].clone());
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }

    let (section, is_header, apply_to) = parse_header_footer_common_cli(section, kind, apply_to);
    let hf_para = parse_usize_cli(hf_para, "--hf-para");
    let offset = parse_usize_cli(offset, "--offset");
    let output = output_path.unwrap_or_else(|| input.clone());
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let result = match action {
        "insert" => {
            let text =
                read_text_argument(inline_text, text_file).unwrap_or_else(|e| exit_cli_error(&e));
            insert_hwp_header_footer_text_bytes_for_cli(
                &data, section, is_header, apply_to, hf_para, offset, &text,
            )
        }
        "delete" => {
            let count = parse_usize_cli(count, "--count");
            delete_hwp_header_footer_text_bytes_for_cli(
                &data, section, is_header, apply_to, hf_para, offset, count,
            )
        }
        _ => Err(format!(
            "지원하지 않는 머리말/꼬리말 텍스트 작업: {}",
            action
        )),
    }
    .unwrap_or_else(|e| exit_cli_error(&e));

    write_hwp_cli_output(&output, &result.bytes).unwrap_or_else(|e| exit_cli_error(&e));
    print_hwp_edit_cli_result(output, result);
}

fn header_footer_paragraph_edit_cli(args: &[String], action: &str) {
    if args.is_empty() {
        exit_cli_error("사용법: rhwp split|merge-header-footer-paragraph <파일.hwp> --section N --kind header|footer --apply-to both|even|odd --hf-para N [--offset N] -o <출력.hwp>");
    }
    let input = args[0].clone();
    let mut section: Option<String> = None;
    let mut kind: Option<String> = None;
    let mut apply_to: Option<String> = None;
    let mut hf_para: Option<String> = None;
    let mut offset: Option<String> = None;
    let mut output_path: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--section" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--section 뒤에 값이 필요합니다.");
                }
                section = Some(args[i].clone());
            }
            "--kind" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--kind 뒤에 header 또는 footer가 필요합니다.");
                }
                kind = Some(args[i].clone());
            }
            "--apply-to" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--apply-to 뒤에 both/even/odd 값이 필요합니다.");
                }
                apply_to = Some(args[i].clone());
            }
            "--hf-para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--hf-para 뒤에 값이 필요합니다.");
                }
                hf_para = Some(args[i].clone());
            }
            "--offset" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--offset 뒤에 값이 필요합니다.");
                }
                offset = Some(args[i].clone());
            }
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("-o/--output 뒤에 경로가 필요합니다.");
                }
                output_path = Some(args[i].clone());
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }

    let (section, is_header, apply_to) = parse_header_footer_common_cli(section, kind, apply_to);
    let hf_para = parse_usize_cli(hf_para, "--hf-para");
    let output = output_path.unwrap_or_else(|| input.clone());
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let result = match action {
        "split" => {
            let offset = parse_usize_cli(offset, "--offset");
            split_hwp_header_footer_paragraph_bytes_for_cli(
                &data, section, is_header, apply_to, hf_para, offset,
            )
        }
        "merge" => merge_hwp_header_footer_paragraph_bytes_for_cli(
            &data, section, is_header, apply_to, hf_para,
        ),
        _ => Err(format!("지원하지 않는 머리말/꼬리말 문단 작업: {}", action)),
    }
    .unwrap_or_else(|e| exit_cli_error(&e));

    write_hwp_cli_output(&output, &result.bytes).unwrap_or_else(|e| exit_cli_error(&e));
    print_hwp_edit_cli_result(output, result);
}

fn get_header_footer_para_info_cli(args: &[String]) {
    if args.is_empty() {
        exit_cli_error("사용법: rhwp get-header-footer-para-info <파일.hwp> --section N --kind header|footer --apply-to both|even|odd --hf-para N");
    }
    let input = args[0].clone();
    let mut section: Option<String> = None;
    let mut kind: Option<String> = None;
    let mut apply_to: Option<String> = None;
    let mut hf_para: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--section" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--section 뒤에 값이 필요합니다.");
                }
                section = Some(args[i].clone());
            }
            "--kind" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--kind 뒤에 header 또는 footer가 필요합니다.");
                }
                kind = Some(args[i].clone());
            }
            "--apply-to" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--apply-to 뒤에 both/even/odd 값이 필요합니다.");
                }
                apply_to = Some(args[i].clone());
            }
            "--hf-para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--hf-para 뒤에 값이 필요합니다.");
                }
                hf_para = Some(args[i].clone());
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }

    let (section, is_header, apply_to) = parse_header_footer_common_cli(section, kind, apply_to);
    let hf_para = parse_usize_cli(hf_para, "--hf-para");
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let result =
        get_hwp_header_footer_para_info_json_for_cli(&data, section, is_header, apply_to, hf_para)
            .unwrap_or_else(|e| exit_cli_error(&e));
    println!("{}", result);
}

fn get_header_footer_para_properties_cli(args: &[String]) {
    if args.is_empty() {
        exit_cli_error("사용법: rhwp get-header-footer-para-properties <파일.hwp> --section N --kind header|footer --apply-to both|even|odd --hf-para N");
    }
    let input = args[0].clone();
    let mut section: Option<String> = None;
    let mut kind: Option<String> = None;
    let mut apply_to: Option<String> = None;
    let mut hf_para: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--section" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--section 뒤에 값이 필요합니다.");
                }
                section = Some(args[i].clone());
            }
            "--kind" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--kind 뒤에 header 또는 footer가 필요합니다.");
                }
                kind = Some(args[i].clone());
            }
            "--apply-to" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--apply-to 뒤에 both/even/odd 값이 필요합니다.");
                }
                apply_to = Some(args[i].clone());
            }
            "--hf-para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--hf-para 뒤에 값이 필요합니다.");
                }
                hf_para = Some(args[i].clone());
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }

    let (section, is_header, apply_to) = parse_header_footer_common_cli(section, kind, apply_to);
    let hf_para = parse_usize_cli(hf_para, "--hf-para");
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let result = get_hwp_header_footer_para_properties_json_for_cli(
        &data, section, is_header, apply_to, hf_para,
    )
    .unwrap_or_else(|e| exit_cli_error(&e));
    println!("{}", result);
}

fn set_header_footer_para_format_cli(args: &[String]) {
    if args.is_empty() {
        exit_cli_error("사용법: rhwp set-header-footer-para-format <파일.hwp> --section N --kind header|footer --apply-to both|even|odd --hf-para N --json <서식JSON> -o <출력.hwp>");
    }
    let input = args[0].clone();
    let mut section: Option<String> = None;
    let mut kind: Option<String> = None;
    let mut apply_to: Option<String> = None;
    let mut hf_para: Option<String> = None;
    let mut inline_json: Option<String> = None;
    let mut json_file: Option<String> = None;
    let mut output_path: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--section" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--section 뒤에 값이 필요합니다.");
                }
                section = Some(args[i].clone());
            }
            "--kind" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--kind 뒤에 header 또는 footer가 필요합니다.");
                }
                kind = Some(args[i].clone());
            }
            "--apply-to" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--apply-to 뒤에 both/even/odd 값이 필요합니다.");
                }
                apply_to = Some(args[i].clone());
            }
            "--hf-para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--hf-para 뒤에 값이 필요합니다.");
                }
                hf_para = Some(args[i].clone());
            }
            "--json" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--json 뒤에 JSON 문자열이 필요합니다.");
                }
                inline_json = Some(args[i].clone());
            }
            "--json-file" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--json-file 뒤에 경로가 필요합니다.");
                }
                json_file = Some(args[i].clone());
            }
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("-o/--output 뒤에 경로가 필요합니다.");
                }
                output_path = Some(args[i].clone());
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }

    let (section, is_header, apply_to) = parse_header_footer_common_cli(section, kind, apply_to);
    let hf_para = parse_usize_cli(hf_para, "--hf-para");
    let props_json =
        read_json_argument(inline_json, json_file).unwrap_or_else(|e| exit_cli_error(&e));
    let output = output_path.unwrap_or_else(|| input.clone());
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let result = set_hwp_header_footer_para_format_bytes_for_cli(
        &data,
        section,
        is_header,
        apply_to,
        hf_para,
        &props_json,
    )
    .unwrap_or_else(|e| exit_cli_error(&e));

    write_hwp_cli_output(&output, &result.bytes).unwrap_or_else(|e| exit_cli_error(&e));
    print_hwp_edit_cli_result(output, result);
}

fn insert_header_footer_field_cli(args: &[String]) {
    if args.is_empty() {
        exit_cli_error("사용법: rhwp insert-header-footer-field <파일.hwp> --section N --kind header|footer --apply-to both|even|odd --hf-para N --offset N --field page-number|total-pages|filename -o <출력.hwp>");
    }
    let input = args[0].clone();
    let mut section: Option<String> = None;
    let mut kind: Option<String> = None;
    let mut apply_to: Option<String> = None;
    let mut hf_para: Option<String> = None;
    let mut offset: Option<String> = None;
    let mut field: Option<String> = None;
    let mut output_path: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--section" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--section 뒤에 값이 필요합니다.");
                }
                section = Some(args[i].clone());
            }
            "--kind" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--kind 뒤에 header 또는 footer가 필요합니다.");
                }
                kind = Some(args[i].clone());
            }
            "--apply-to" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--apply-to 뒤에 both/even/odd 값이 필요합니다.");
                }
                apply_to = Some(args[i].clone());
            }
            "--hf-para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--hf-para 뒤에 값이 필요합니다.");
                }
                hf_para = Some(args[i].clone());
            }
            "--offset" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--offset 뒤에 값이 필요합니다.");
                }
                offset = Some(args[i].clone());
            }
            "--field" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error(
                        "--field 뒤에 page-number|total-pages|filename 값이 필요합니다.",
                    );
                }
                field = Some(args[i].clone());
            }
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("-o/--output 뒤에 경로가 필요합니다.");
                }
                output_path = Some(args[i].clone());
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }

    let (section, is_header, apply_to) = parse_header_footer_common_cli(section, kind, apply_to);
    let hf_para = parse_usize_cli(hf_para, "--hf-para");
    let offset = parse_usize_cli(offset, "--offset");
    let field = field.unwrap_or_else(|| exit_cli_error("--field 값이 필요합니다."));
    let field_type =
        parse_header_footer_field_type_for_cli(&field).unwrap_or_else(|e| exit_cli_error(&e));
    let output = output_path.unwrap_or_else(|| input.clone());
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let result = insert_hwp_header_footer_field_bytes_for_cli(
        &data, section, is_header, apply_to, hf_para, offset, field_type,
    )
    .unwrap_or_else(|e| exit_cli_error(&e));

    write_hwp_cli_output(&output, &result.bytes).unwrap_or_else(|e| exit_cli_error(&e));
    print_hwp_edit_cli_result(output, result);
}

fn apply_header_footer_template_cli(args: &[String]) {
    if args.is_empty() {
        exit_cli_error("사용법: rhwp apply-header-footer-template <파일.hwp> --section N --kind header|footer --apply-to both|even|odd --template N -o <출력.hwp>");
    }
    let input = args[0].clone();
    let mut section: Option<String> = None;
    let mut kind: Option<String> = None;
    let mut apply_to: Option<String> = None;
    let mut template: Option<String> = None;
    let mut output_path: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--section" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--section 뒤에 값이 필요합니다.");
                }
                section = Some(args[i].clone());
            }
            "--kind" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--kind 뒤에 header 또는 footer가 필요합니다.");
                }
                kind = Some(args[i].clone());
            }
            "--apply-to" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--apply-to 뒤에 both/even/odd 값이 필요합니다.");
                }
                apply_to = Some(args[i].clone());
            }
            "--template" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--template 뒤에 0~10 값이 필요합니다.");
                }
                template = Some(args[i].clone());
            }
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("-o/--output 뒤에 경로가 필요합니다.");
                }
                output_path = Some(args[i].clone());
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }

    let (section, is_header, apply_to) = parse_header_footer_common_cli(section, kind, apply_to);
    let template_id = parse_u32_cli(template, "--template");
    if template_id > u8::MAX as u32 {
        exit_cli_error("--template 값이 너무 큽니다.");
    }
    let output = output_path.unwrap_or_else(|| input.clone());
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let result = apply_hwp_header_footer_template_bytes_for_cli(
        &data,
        section,
        is_header,
        apply_to,
        template_id as u8,
    )
    .unwrap_or_else(|e| exit_cli_error(&e));

    write_hwp_cli_output(&output, &result.bytes).unwrap_or_else(|e| exit_cli_error(&e));
    print_hwp_edit_cli_result(output, result);
}

fn list_master_pages_cli(args: &[String]) {
    if args.is_empty() {
        exit_cli_error("사용법: rhwp list-master-pages <파일.hwp> --section N");
    }
    let input = args[0].clone();
    let mut section: Option<String> = Some("0".to_string());

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--section" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--section 뒤에 값이 필요합니다.");
                }
                section = Some(args[i].clone());
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }

    let section = parse_usize_cli(section, "--section");
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let result =
        list_hwp_master_pages_json_for_cli(&data, section).unwrap_or_else(|e| exit_cli_error(&e));
    println!("{}", result);
}

fn create_master_page_cli(args: &[String]) {
    if args.is_empty() {
        exit_cli_error("사용법: rhwp create-master-page <파일.hwp> --section N --apply-to both|even|odd [--text <텍스트>] [--extension] [--overlap] -o <출력.hwp>");
    }
    let input = args[0].clone();
    let mut section: Option<String> = None;
    let mut apply_to: Option<String> = Some("both".to_string());
    let mut inline_text: Option<String> = Some(String::new());
    let mut text_file: Option<String> = None;
    let mut is_extension = false;
    let mut overlap = false;
    let mut output_path: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--section" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--section 뒤에 값이 필요합니다.");
                }
                section = Some(args[i].clone());
            }
            "--apply-to" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--apply-to 뒤에 both/even/odd 값이 필요합니다.");
                }
                apply_to = Some(args[i].clone());
            }
            "--text" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--text 뒤에 텍스트가 필요합니다.");
                }
                inline_text = Some(args[i].clone());
                text_file = None;
            }
            "--text-file" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--text-file 뒤에 경로가 필요합니다.");
                }
                inline_text = None;
                text_file = Some(args[i].clone());
            }
            "--extension" => is_extension = true,
            "--overlap" => overlap = true,
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("-o/--output 뒤에 경로가 필요합니다.");
                }
                output_path = Some(args[i].clone());
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }

    let section = parse_usize_cli(section, "--section");
    let apply_to = apply_to.unwrap_or_else(|| exit_cli_error("--apply-to 값이 필요합니다."));
    let apply_to =
        parse_header_footer_apply_to_for_cli(&apply_to).unwrap_or_else(|e| exit_cli_error(&e));
    let text = read_text_argument(inline_text, text_file).unwrap_or_else(|e| exit_cli_error(&e));
    let output = output_path.unwrap_or_else(|| input.clone());
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let result = create_hwp_master_page_bytes_for_cli(
        &data,
        section,
        apply_to,
        is_extension,
        overlap,
        &text,
    )
    .unwrap_or_else(|e| exit_cli_error(&e));

    write_hwp_cli_output(&output, &result.bytes).unwrap_or_else(|e| exit_cli_error(&e));
    print_hwp_edit_cli_result(output, result);
}

fn set_master_page_text_cli(args: &[String]) {
    if args.is_empty() {
        exit_cli_error("사용법: rhwp set-master-page-text <파일.hwp> --section N --master N --para N --text <텍스트> -o <출력.hwp>");
    }
    let input = args[0].clone();
    let mut section: Option<String> = None;
    let mut master: Option<String> = None;
    let mut para: Option<String> = Some("0".to_string());
    let mut inline_text: Option<String> = None;
    let mut text_file: Option<String> = None;
    let mut output_path: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--section" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--section 뒤에 값이 필요합니다.");
                }
                section = Some(args[i].clone());
            }
            "--master" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--master 뒤에 값이 필요합니다.");
                }
                master = Some(args[i].clone());
            }
            "--para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--para 뒤에 값이 필요합니다.");
                }
                para = Some(args[i].clone());
            }
            "--text" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--text 뒤에 텍스트가 필요합니다.");
                }
                inline_text = Some(args[i].clone());
            }
            "--text-file" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--text-file 뒤에 경로가 필요합니다.");
                }
                text_file = Some(args[i].clone());
            }
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("-o/--output 뒤에 경로가 필요합니다.");
                }
                output_path = Some(args[i].clone());
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }

    let section = parse_usize_cli(section, "--section");
    let master = parse_usize_cli(master, "--master");
    let para = parse_usize_cli(para, "--para");
    let text = read_text_argument(inline_text, text_file).unwrap_or_else(|e| exit_cli_error(&e));
    let output = output_path.unwrap_or_else(|| input.clone());
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let result = set_hwp_master_page_text_bytes_for_cli(&data, section, master, para, &text)
        .unwrap_or_else(|e| exit_cli_error(&e));

    write_hwp_cli_output(&output, &result.bytes).unwrap_or_else(|e| exit_cli_error(&e));
    print_hwp_edit_cli_result(output, result);
}

fn delete_master_page_cli(args: &[String]) {
    if args.is_empty() {
        exit_cli_error(
            "사용법: rhwp delete-master-page <파일.hwp> --section N --master N -o <출력.hwp>",
        );
    }
    let input = args[0].clone();
    let mut section: Option<String> = None;
    let mut master: Option<String> = None;
    let mut output_path: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--section" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--section 뒤에 값이 필요합니다.");
                }
                section = Some(args[i].clone());
            }
            "--master" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--master 뒤에 값이 필요합니다.");
                }
                master = Some(args[i].clone());
            }
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("-o/--output 뒤에 경로가 필요합니다.");
                }
                output_path = Some(args[i].clone());
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }

    let section = parse_usize_cli(section, "--section");
    let master = parse_usize_cli(master, "--master");
    let output = output_path.unwrap_or_else(|| input.clone());
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let result = delete_hwp_master_page_bytes_for_cli(&data, section, master)
        .unwrap_or_else(|e| exit_cli_error(&e));

    write_hwp_cli_output(&output, &result.bytes).unwrap_or_else(|e| exit_cli_error(&e));
    print_hwp_edit_cli_result(output, result);
}

fn export_svg(args: &[String]) {
    if args.is_empty() {
        eprintln!("오류: HWP 파일 경로를 지정해주세요.");
        eprintln!("사용법: rhwp export-svg <파일.hwp> [옵션] (rhwp --help 참조)");
        return;
    }

    let file_path = &args[0];
    let mut output_dir = "output".to_string();
    let mut target_page: Option<u32> = None;
    let mut show_para_marks = false;
    let mut show_control_codes = false;
    let mut debug_overlay = false;
    let mut grid_mm: Option<f64> = None;
    let mut grid_origin = GridOriginOption::Fixed((0.0_f64, 0.0_f64));
    let mut respect_vpos_reset = false;
    let mut font_embed_mode = rhwp::renderer::svg::FontEmbedMode::None;
    let mut font_paths: Vec<std::path::PathBuf> = Vec::new();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--output" | "-o" => {
                if i + 1 < args.len() {
                    output_dir = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("오류: --output 뒤에 폴더 경로가 필요합니다.");
                    return;
                }
            }
            "--page" | "-p" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<u32>() {
                        Ok(n) => target_page = Some(n),
                        Err(_) => {
                            eprintln!("오류: 페이지 번호가 올바르지 않습니다.");
                            return;
                        }
                    }
                    i += 2;
                } else {
                    eprintln!("오류: --page 뒤에 페이지 번호가 필요합니다.");
                    return;
                }
            }
            "--show-para-marks" => {
                show_para_marks = true;
                i += 1;
            }
            "--show-control-codes" => {
                show_control_codes = true;
                i += 1;
            }
            "--debug-overlay" => {
                debug_overlay = true;
                i += 1;
            }
            "--respect-vpos-reset" => {
                respect_vpos_reset = true;
                i += 1;
            }
            arg if arg == "--show-grid" || arg.starts_with("--show-grid=") => {
                grid_mm = if let Some(value) = arg.strip_prefix("--show-grid=") {
                    match parse_grid_mm(value) {
                        Some(v) => Some(v),
                        None => {
                            eprintln!(
                                "오류: --show-grid 값이 올바르지 않습니다. 예: --show-grid=3mm"
                            );
                            return;
                        }
                    }
                } else {
                    Some(1.0)
                };
                i += 1;
            }
            arg if arg == "--grid-origin" || arg == "--grid-paper-origin" => {
                if i + 1 < args.len() {
                    match parse_grid_origin_option(&args[i + 1]) {
                        Some(v) => grid_origin = v,
                        None => {
                            eprintln!(
                                "오류: --grid-origin 값이 올바르지 않습니다. 예: --grid-origin=15mm,20mm 또는 --grid-origin=auto"
                            );
                            return;
                        }
                    }
                    i += 2;
                } else {
                    eprintln!("오류: --grid-origin 뒤에 가로,세로 값이 필요합니다.");
                    return;
                }
            }
            arg if arg.starts_with("--grid-origin=") || arg.starts_with("--grid-paper-origin=") => {
                let value = arg
                    .strip_prefix("--grid-origin=")
                    .or_else(|| arg.strip_prefix("--grid-paper-origin="))
                    .unwrap_or_default();
                match parse_grid_origin_option(value) {
                    Some(v) => grid_origin = v,
                    None => {
                        eprintln!(
                            "오류: --grid-origin 값이 올바르지 않습니다. 예: --grid-origin=15mm,20mm 또는 --grid-origin=auto"
                        );
                        return;
                    }
                }
                i += 1;
            }
            "--font-style" => {
                font_embed_mode = rhwp::renderer::svg::FontEmbedMode::Style;
                i += 1;
            }
            "--embed-fonts" => {
                font_embed_mode = rhwp::renderer::svg::FontEmbedMode::Subset;
                i += 1;
            }
            "--embed-fonts=full" => {
                font_embed_mode = rhwp::renderer::svg::FontEmbedMode::Full;
                i += 1;
            }
            "--font-path" => {
                if i + 1 < args.len() {
                    font_paths.push(std::path::PathBuf::from(&args[i + 1]));
                    i += 2;
                } else {
                    eprintln!("오류: --font-path 뒤에 경로가 필요합니다.");
                    return;
                }
            }
            _ => {
                eprintln!("알 수 없는 옵션: {}", args[i]);
                i += 1;
            }
        }
    }

    // 파일 읽기
    let data = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return;
        }
    };

    // 문서 로드
    let mut doc = match rhwp::wasm_api::HwpDocument::from_bytes(&data) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: HWP 파싱 실패 - {}", e);
            return;
        }
    };

    // [Task #741 후속] 외부 file path 그림 영역 영역 HWP file 영역 영역 같은 dir 영역
    // 영역 image 영역 영역 자동 load (basename 매칭).
    if let Some(parent) = std::path::Path::new(file_path).parent() {
        let _loaded = doc.populate_external_images_from_dir(parent);
    }

    if show_para_marks {
        doc.set_show_paragraph_marks(true);
    }
    if show_control_codes {
        doc.set_show_control_codes(true);
    }
    if debug_overlay {
        doc.set_debug_overlay(true);
    }
    if respect_vpos_reset {
        doc.set_respect_vpos_reset(true);
    }

    let page_count = doc.page_count();
    println!("문서 로드 완료: {} ({}페이지)", file_path, page_count);

    // 출력 폴더 생성
    let output_path = Path::new(&output_dir);
    if !output_path.exists() {
        if let Err(e) = fs::create_dir_all(output_path) {
            eprintln!(
                "오류: 출력 폴더를 생성할 수 없습니다 - {}: {}",
                output_dir, e
            );
            return;
        }
    }

    // 페이지 범위 결정
    let pages: Vec<u32> = match target_page {
        Some(p) => {
            if p >= page_count {
                eprintln!(
                    "오류: 페이지 번호가 범위를 벗어났습니다 (0~{})",
                    page_count - 1
                );
                return;
            }
            vec![p]
        }
        None => (0..page_count).collect(),
    };

    // SVG 내보내기
    let file_stem = Path::new(file_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("page");

    for page_num in &pages {
        let svg_result = if font_embed_mode != rhwp::renderer::svg::FontEmbedMode::None {
            doc.render_page_svg_with_fonts(*page_num, font_embed_mode, &font_paths)
        } else {
            doc.render_page_svg_native(*page_num)
        };
        match svg_result {
            Ok(mut svg) => {
                // 격자 오버레이 삽입
                if let Some(mm) = grid_mm {
                    let origin_mm = match grid_origin {
                        GridOriginOption::Fixed(origin) => origin,
                        GridOriginOption::AutoPaper => {
                            match grid_paper_origin_mm(&doc, *page_num) {
                                Some(origin) => origin,
                                None => {
                                    eprintln!(
                                        "오류: 페이지 {}의 격자 기준 위치를 계산할 수 없습니다.",
                                        page_num
                                    );
                                    continue;
                                }
                            }
                        }
                    };
                    svg = insert_grid_overlay(&svg, mm, origin_mm);
                }
                let svg_filename = if page_count == 1 {
                    format!("{}.svg", file_stem)
                } else {
                    format!("{}_{:03}.svg", file_stem, page_num + 1)
                };
                let svg_path = output_path.join(&svg_filename);

                match fs::write(&svg_path, &svg) {
                    Ok(_) => println!("  → {}", svg_path.display()),
                    Err(e) => eprintln!("오류: SVG 저장 실패 - {}: {}", svg_path.display(), e),
                }
            }
            Err(e) => {
                eprintln!("오류: 페이지 {} 렌더링 실패 - {:?}", page_num, e);
            }
        }
    }

    println!(
        "내보내기 완료: {}개 SVG 파일 → {}/",
        pages.len(),
        output_dir
    );
}

fn export_render_tree(args: &[String]) {
    if args.is_empty() {
        eprintln!("오류: HWP 파일 경로를 지정해주세요.");
        eprintln!("사용법: rhwp export-render-tree <파일.hwp> [옵션] (rhwp --help 참조)");
        return;
    }

    let file_path = &args[0];
    let mut output_dir = "output".to_string();
    let mut target_page: Option<u32> = None;
    let mut show_para_marks = false;
    let mut show_control_codes = false;
    let mut respect_vpos_reset = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--output" | "-o" => {
                if i + 1 < args.len() {
                    output_dir = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("오류: --output 뒤에 폴더 경로가 필요합니다.");
                    return;
                }
            }
            "--page" | "-p" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<u32>() {
                        Ok(n) => target_page = Some(n),
                        Err(_) => {
                            eprintln!("오류: 페이지 번호가 올바르지 않습니다.");
                            return;
                        }
                    }
                    i += 2;
                } else {
                    eprintln!("오류: --page 뒤에 페이지 번호가 필요합니다.");
                    return;
                }
            }
            "--show-para-marks" => {
                show_para_marks = true;
                i += 1;
            }
            "--show-control-codes" => {
                show_control_codes = true;
                i += 1;
            }
            "--respect-vpos-reset" => {
                respect_vpos_reset = true;
                i += 1;
            }
            _ => {
                eprintln!("알 수 없는 옵션: {}", args[i]);
                i += 1;
            }
        }
    }

    let data = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return;
        }
    };

    let mut doc = match rhwp::wasm_api::HwpDocument::from_bytes(&data) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: HWP 파싱 실패 - {}", e);
            return;
        }
    };

    if let Some(parent) = std::path::Path::new(file_path).parent() {
        let _loaded = doc.populate_external_images_from_dir(parent);
    }

    if show_para_marks {
        doc.set_show_paragraph_marks(true);
    }
    if show_control_codes {
        doc.set_show_control_codes(true);
    }
    if respect_vpos_reset {
        doc.set_respect_vpos_reset(true);
    }

    let page_count = doc.page_count();
    println!("문서 로드 완료: {} ({}페이지)", file_path, page_count);

    let output_path = Path::new(&output_dir);
    if !output_path.exists() {
        if let Err(e) = fs::create_dir_all(output_path) {
            eprintln!(
                "오류: 출력 폴더를 생성할 수 없습니다 - {}: {}",
                output_dir, e
            );
            return;
        }
    }

    let pages: Vec<u32> = match target_page {
        Some(p) => {
            if p >= page_count {
                eprintln!(
                    "오류: 페이지 번호가 범위를 벗어났습니다 (0~{})",
                    page_count - 1
                );
                return;
            }
            vec![p]
        }
        None => (0..page_count).collect(),
    };

    for page_num in &pages {
        match doc.build_page_render_tree(*page_num) {
            Ok(tree) => {
                let json_path = output_path.join(format!("render_tree_{:03}.json", page_num + 1));
                let json = tree.root.to_json();
                match fs::write(&json_path, json) {
                    Ok(_) => println!("  → {}", json_path.display()),
                    Err(e) => {
                        eprintln!(
                            "오류: render tree 저장 실패 - {}: {}",
                            json_path.display(),
                            e
                        )
                    }
                }
            }
            Err(e) => {
                eprintln!("오류: 페이지 {} render tree 생성 실패 - {:?}", page_num, e);
            }
        }
    }

    println!(
        "내보내기 완료: {}개 render tree JSON 파일 → {}/",
        pages.len(),
        output_dir
    );
}

fn parse_grid_mm(value: &str) -> Option<f64> {
    let trimmed = value.trim();
    let number = trimmed
        .strip_suffix("mm")
        .or_else(|| trimmed.strip_suffix("MM"))
        .unwrap_or(trimmed)
        .trim();
    let mm = number.parse::<f64>().ok()?;
    if mm.is_finite() && mm > 0.0 {
        Some(mm)
    } else {
        None
    }
}

#[derive(Clone, Copy)]
enum GridOriginOption {
    Fixed((f64, f64)),
    AutoPaper,
}

fn parse_grid_origin_option(value: &str) -> Option<GridOriginOption> {
    if value.eq_ignore_ascii_case("auto") {
        return Some(GridOriginOption::AutoPaper);
    }
    parse_grid_origin_mm(value).map(GridOriginOption::Fixed)
}

fn parse_grid_origin_mm(value: &str) -> Option<(f64, f64)> {
    let (x, y) = value.split_once(',')?;
    Some((parse_grid_mm(x)?, parse_grid_mm(y)?))
}

fn grid_paper_origin_mm(doc: &rhwp::wasm_api::HwpDocument, page_num: u32) -> Option<(f64, f64)> {
    let page_info = doc.get_page_info_native(page_num).ok()?;
    let page_info: serde_json::Value = serde_json::from_str(&page_info).ok()?;
    let section_idx = page_info.get("sectionIndex")?.as_u64()? as usize;
    let page_def = &doc
        .document()
        .sections
        .get(section_idx)?
        .section_def
        .page_def;
    Some((
        hu_to_mm(page_def.margin_left),
        hu_to_mm(page_def.margin_top + page_def.margin_header),
    ))
}

/// SVG에 mm 단위 점 격자 오버레이를 삽입한다.
/// export-svg 디버그용 격자는 한컴오피스의 "종이 기준 위치"를 옵션으로 맞출 수 있다.
fn insert_grid_overlay(svg: &str, grid_mm: f64, origin_mm: (f64, f64)) -> String {
    // SVG viewBox에서 크기 추출
    let (width, height) = extract_svg_dimensions(svg);
    // 96dpi: 1inch = 25.4mm, 1px = 25.4/96 = 0.2646mm.
    let grid_size = 96.0 / 25.4 * grid_mm;
    let origin_x = 96.0 / 25.4 * origin_mm.0;
    let origin_y = 96.0 / 25.4 * origin_mm.1;

    let g = format!("{:.4}", grid_size);
    let ox = format!("{:.4}", origin_x);
    let oy = format!("{:.4}", origin_y);
    let w = format!("{:.2}", width);
    let h = format!("{:.2}", height);
    let defs_part = format!(
        "<defs><pattern id=\"rhwp-grid\" x=\"{ox}\" y=\"{oy}\" width=\"{g}\" height=\"{g}\" patternUnits=\"userSpaceOnUse\"><rect x=\"0\" y=\"0\" width=\"1\" height=\"1\" fill=\"#002096\" fill-opacity=\"0.9\"/></pattern></defs>"
    );
    let grid_rect = format!("\n<rect width=\"{w}\" height=\"{h}\" fill=\"url(#rhwp-grid)\"/>");
    let grid_defs =
        format!("{defs_part}\n<rect width=\"{w}\" height=\"{h}\" fill=\"url(#rhwp-grid)\"/>\n");

    // 페이지 배경(fill="#ffffff") rect 직후에 격자를 삽입
    // 이렇게 해야 흰색 배경 위에, 본문 컨텐츠 아래에 격자가 표시됨
    let bg_pattern = "fill=\"#ffffff\"/>";
    if let Some(pos) = svg.find(bg_pattern) {
        let insert_pos = pos + bg_pattern.len();
        // defs는 SVG 시작 부분에, 격자 rect는 배경 뒤에
        // defs를 <svg> 태그 직후에 삽입
        let mut result = svg.to_string();
        // 배경 rect 뒤에 격자 rect 삽입
        result.insert_str(insert_pos, &grid_rect);
        // <svg ...>\n 직후에 defs 삽입
        if let Some(svg_end) = result.find(">\n") {
            result.insert_str(svg_end + 2, &format!("{}\n", defs_part));
        }
        result
    } else {
        // 배경 rect가 없으면 기존 방식
        if let Some(pos) = svg.find(">\n") {
            let insert_pos = pos + 2;
            format!("{}{}{}", &svg[..insert_pos], grid_defs, &svg[insert_pos..])
        } else {
            svg.to_string()
        }
    }
}

/// SVG의 width/height 속성 또는 viewBox에서 크기를 추출한다.
fn extract_svg_dimensions(svg: &str) -> (f64, f64) {
    // viewBox="0 0 W H" 패턴에서 추출
    if let Some(vb_start) = svg.find("viewBox=\"") {
        let vb = &svg[vb_start + 9..];
        if let Some(vb_end) = vb.find('"') {
            let parts: Vec<&str> = vb[..vb_end].split_whitespace().collect();
            if parts.len() == 4 {
                let w: f64 = parts[2].parse().unwrap_or(800.0);
                let h: f64 = parts[3].parse().unwrap_or(1100.0);
                return (w, h);
            }
        }
    }
    // width/height 속성에서 추출
    let w = extract_attr_f64(svg, "width").unwrap_or(800.0);
    let h = extract_attr_f64(svg, "height").unwrap_or(1100.0);
    (w, h)
}

fn extract_attr_f64(svg: &str, attr: &str) -> Option<f64> {
    let pattern = format!("{}=\"", attr);
    if let Some(start) = svg.find(&pattern) {
        let val = &svg[start + pattern.len()..];
        if let Some(end) = val.find('"') {
            return val[..end].trim_end_matches("px").parse().ok();
        }
    }
    None
}

#[cfg(not(feature = "native-skia"))]
fn export_png(_args: &[String]) {
    eprintln!("오류: export-png 명령은 native-skia feature 가 활성화되어야 합니다.");
    eprintln!("       cargo build --release --features native-skia");
}

#[cfg(feature = "native-skia")]
fn export_png(args: &[String]) {
    use rhwp::document_core::queries::rendering::{PngExportOptions, VlmTarget};

    if args.is_empty() {
        eprintln!("오류: HWP 파일 경로를 지정해주세요.");
        eprintln!("사용법: rhwp export-png <파일.hwp> [옵션] (rhwp --help 참조)");
        return;
    }

    let file_path = &args[0];
    let mut output_dir = "output".to_string();
    let mut target_page: Option<u32> = None;
    let mut font_paths: Vec<std::path::PathBuf> = Vec::new();
    let mut scale: Option<f64> = None;
    let mut max_dimension: Option<i32> = None;
    let mut vlm_target: Option<VlmTarget> = None;
    let mut dpi: Option<f64> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--output" | "-o" => {
                if i + 1 < args.len() {
                    output_dir = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("오류: --output 뒤에 폴더 경로가 필요합니다.");
                    return;
                }
            }
            "--page" | "-p" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<u32>() {
                        Ok(n) => target_page = Some(n),
                        Err(_) => {
                            eprintln!("오류: 페이지 번호가 올바르지 않습니다.");
                            return;
                        }
                    }
                    i += 2;
                } else {
                    eprintln!("오류: --page 뒤에 페이지 번호가 필요합니다.");
                    return;
                }
            }
            "--font-path" => {
                if i + 1 < args.len() {
                    font_paths.push(std::path::PathBuf::from(&args[i + 1]));
                    i += 2;
                } else {
                    eprintln!("오류: --font-path 뒤에 경로가 필요합니다.");
                    return;
                }
            }
            "--scale" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<f64>() {
                        Ok(s) if s.is_finite() && s > 0.0 => scale = Some(s),
                        _ => {
                            eprintln!("오류: --scale 값이 올바르지 않습니다 (양수 실수 필요).");
                            return;
                        }
                    }
                    i += 2;
                } else {
                    eprintln!("오류: --scale 뒤에 배율 값이 필요합니다.");
                    return;
                }
            }
            "--max-dimension" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<i32>() {
                        Ok(n) if n > 0 => max_dimension = Some(n),
                        _ => {
                            eprintln!(
                                "오류: --max-dimension 값이 올바르지 않습니다 (양수 정수 필요)."
                            );
                            return;
                        }
                    }
                    i += 2;
                } else {
                    eprintln!("오류: --max-dimension 뒤에 픽셀 값이 필요합니다.");
                    return;
                }
            }
            "--dpi" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<f64>() {
                        Ok(d) if d.is_finite() && d > 0.0 => dpi = Some(d),
                        _ => {
                            eprintln!("오류: --dpi 값이 올바르지 않습니다 (양수 실수 필요).");
                            return;
                        }
                    }
                    i += 2;
                } else {
                    eprintln!("오류: --dpi 뒤에 DPI 값이 필요합니다.");
                    return;
                }
            }
            "--vlm-target" => {
                if i + 1 < args.len() {
                    match VlmTarget::from_str(&args[i + 1]) {
                        Some(t) => vlm_target = Some(t),
                        None => {
                            eprintln!(
                                "오류: --vlm-target 값이 올바르지 않습니다 (지원: {}).",
                                VlmTarget::all_names()
                            );
                            return;
                        }
                    }
                    i += 2;
                } else {
                    eprintln!("오류: --vlm-target 뒤에 프리셋 이름이 필요합니다.");
                    return;
                }
            }
            _ => {
                eprintln!("알 수 없는 옵션: {}", args[i]);
                i += 1;
            }
        }
    }

    let png_options = PngExportOptions {
        scale,
        max_dimension,
        vlm_target,
        dpi,
        font_paths: font_paths.clone(),
    };

    let data = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return;
        }
    };

    let core = match rhwp::document_core::DocumentCore::from_bytes(&data) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("오류: HWP 파싱 실패 - {:?}", e);
            return;
        }
    };

    let page_count = core.page_count();
    println!("문서 로드 완료: {} ({}페이지)", file_path, page_count);

    let output_path = Path::new(&output_dir);
    if !output_path.exists() {
        if let Err(e) = fs::create_dir_all(output_path) {
            eprintln!(
                "오류: 출력 폴더를 생성할 수 없습니다 - {}: {}",
                output_dir, e
            );
            return;
        }
    }

    let pages: Vec<u32> = match target_page {
        Some(p) => {
            if p >= page_count as u32 {
                eprintln!(
                    "오류: 페이지 번호가 범위를 벗어났습니다 (0~{})",
                    page_count - 1
                );
                return;
            }
            vec![p]
        }
        None => (0..page_count as u32).collect(),
    };

    let file_stem = Path::new(file_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("page");

    let total_pages = pages.len();
    let mut success = 0;
    let mut total_bytes = 0usize;

    for page_num in &pages {
        let has_options = png_options.scale.is_some()
            || png_options.max_dimension.is_some()
            || png_options.vlm_target.is_some()
            || png_options.dpi.is_some();
        let result = if has_options {
            core.render_page_png_native_with_export_options(*page_num, &png_options)
        } else if !font_paths.is_empty() {
            core.render_page_png_native_with_fonts(*page_num, &font_paths)
        } else {
            core.render_page_png_native(*page_num)
        };
        match result {
            Ok(png_bytes) => {
                let png_filename = if total_pages == 1 {
                    format!("{}.png", file_stem)
                } else {
                    format!("{}_{:03}.png", file_stem, page_num + 1)
                };
                let png_path = output_path.join(&png_filename);
                if let Err(e) = fs::write(&png_path, &png_bytes) {
                    eprintln!("오류: 페이지 {} PNG 저장 실패 - {}", page_num + 1, e);
                    continue;
                }
                println!("  → {} ({} bytes)", png_path.display(), png_bytes.len());
                total_bytes += png_bytes.len();
                success += 1;
            }
            Err(e) => {
                eprintln!("오류: 페이지 {} 렌더링 실패 - {:?}", page_num + 1, e);
            }
        }
    }

    println!(
        "내보내기 완료: {}개 PNG 파일 → {}/ ({:.1} MB)",
        success,
        output_dir,
        total_bytes as f64 / 1024.0 / 1024.0
    );
}

fn export_pdf(args: &[String]) {
    if args.is_empty() {
        eprintln!("오류: HWP 파일 경로를 지정해주세요.");
        eprintln!("사용법: rhwp export-pdf <파일.hwp> [-o 출력.pdf] [-p 페이지]");
        return;
    }

    #[cfg(target_arch = "wasm32")]
    {
        eprintln!("오류: PDF 내보내기는 native 빌드에서만 지원됩니다.");
        return;
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let file_path = &args[0];
        let mut output_file = String::new();
        let mut target_page: Option<u32> = None;

        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--output" | "-o" => {
                    if i + 1 < args.len() {
                        output_file = args[i + 1].clone();
                        i += 2;
                    } else {
                        eprintln!("오류: --output 뒤에 파일 경로가 필요합니다.");
                        return;
                    }
                }
                "--page" | "-p" => {
                    if i + 1 < args.len() {
                        match args[i + 1].parse::<u32>() {
                            Ok(n) => target_page = Some(n),
                            Err(_) => {
                                eprintln!("오류: 페이지 번호가 올바르지 않습니다.");
                                return;
                            }
                        }
                        i += 2;
                    } else {
                        eprintln!("오류: --page 뒤에 페이지 번호가 필요합니다.");
                        return;
                    }
                }
                _ => {
                    i += 1;
                }
            }
        }

        // 기본 출력 파일명
        if output_file.is_empty() {
            let stem = Path::new(file_path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("output");
            output_file = format!("output/{}.pdf", stem);
        }

        let data = match fs::read(file_path) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
                return;
            }
        };

        let doc = match rhwp::wasm_api::HwpDocument::from_bytes(&data) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("오류: HWP 파싱 실패 - {}", e);
                return;
            }
        };

        let page_count = doc.page_count();
        println!("문서 로드 완료: {} ({}페이지)", file_path, page_count);

        // 출력 디렉토리 생성
        if let Some(parent) = Path::new(&output_file).parent() {
            if !parent.exists() {
                if let Err(e) = fs::create_dir_all(parent) {
                    eprintln!("오류: 출력 디렉토리를 만들 수 없습니다 - {}", e);
                    return;
                }
            }
        }

        // 페이지 범위 결정
        let pages: Vec<u32> = match target_page {
            Some(p) => {
                if p >= page_count {
                    eprintln!(
                        "오류: 페이지 번호가 범위를 벗어났습니다 (0~{})",
                        page_count - 1
                    );
                    return;
                }
                vec![p]
            }
            None => (0..page_count).collect(),
        };

        let pdf_bytes = match doc.render_pages_pdf_native(&pages) {
            Ok(bytes) => bytes,
            Err(e) => {
                eprintln!("오류: PDF 변환 실패 - {}", e);
                return;
            }
        };
        if let Err(e) = fs::write(&output_file, &pdf_bytes) {
            eprintln!("오류: PDF 저장 실패 - {}", e);
            return;
        }
        println!(
            "  → {} ({}KB, {}페이지)",
            output_file,
            pdf_bytes.len() / 1024,
            pages.len()
        );
        println!("PDF 내보내기 완료");
    }
}

fn export_text(args: &[String]) {
    if args.is_empty() {
        eprintln!("오류: HWP 파일 경로를 지정해주세요.");
        eprintln!("사용법: rhwp export-text <파일.hwp> [옵션] (rhwp --help 참조)");
        return;
    }

    let file_path = &args[0];
    let mut output_dir = "output".to_string();
    let mut target_page: Option<u32> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--output" | "-o" => {
                if i + 1 < args.len() {
                    output_dir = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("오류: --output 뒤에 폴더 경로가 필요합니다.");
                    return;
                }
            }
            "--page" | "-p" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<u32>() {
                        Ok(n) => target_page = Some(n),
                        Err(_) => {
                            eprintln!("오류: 페이지 번호가 올바르지 않습니다.");
                            return;
                        }
                    }
                    i += 2;
                } else {
                    eprintln!("오류: --page 뒤에 페이지 번호가 필요합니다.");
                    return;
                }
            }
            _ => {
                eprintln!("알 수 없는 옵션: {}", args[i]);
                i += 1;
            }
        }
    }

    let data = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return;
        }
    };

    let doc = match rhwp::wasm_api::HwpDocument::from_bytes(&data) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: HWP 파싱 실패 - {}", e);
            return;
        }
    };

    let page_count = doc.page_count();
    println!("문서 로드 완료: {} ({}페이지)", file_path, page_count);
    if page_count == 0 {
        eprintln!("오류: 문서에 페이지가 없습니다.");
        return;
    }

    let output_path = Path::new(&output_dir);
    if !output_path.exists() {
        if let Err(e) = fs::create_dir_all(output_path) {
            eprintln!(
                "오류: 출력 폴더를 생성할 수 없습니다 - {}: {}",
                output_dir, e
            );
            return;
        }
    }

    let pages: Vec<u32> = match target_page {
        Some(p) => {
            if p >= page_count {
                eprintln!(
                    "오류: 페이지 번호가 범위를 벗어났습니다 (0~{})",
                    page_count - 1
                );
                return;
            }
            vec![p]
        }
        None => (0..page_count).collect(),
    };

    let file_stem = Path::new(file_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("page");

    for page_num in &pages {
        match doc.extract_page_text_native(*page_num) {
            Ok(mut text) => {
                if !text.ends_with('\n') {
                    text.push('\n');
                }

                let txt_filename = if page_count == 1 {
                    format!("{}.txt", file_stem)
                } else {
                    format!("{}_{:03}.txt", file_stem, page_num + 1)
                };
                let txt_path = output_path.join(&txt_filename);

                match fs::write(&txt_path, text.as_bytes()) {
                    Ok(_) => println!("  → {}", txt_path.display()),
                    Err(e) => eprintln!("오류: TXT 저장 실패 - {}: {}", txt_path.display(), e),
                }
            }
            Err(e) => {
                eprintln!("오류: 페이지 {} 텍스트 추출 실패 - {:?}", page_num, e);
            }
        }
    }

    println!(
        "텍스트 내보내기 완료: {}개 TXT 파일 → {}/",
        pages.len(),
        output_dir
    );
}

fn export_markdown(args: &[String]) {
    if args.is_empty() {
        eprintln!("오류: HWP 파일 경로를 지정해주세요.");
        eprintln!("사용법: rhwp export-markdown <파일.hwp> [옵션] (rhwp --help 참조)");
        return;
    }

    let file_path = &args[0];
    let mut output_dir = "output".to_string();
    let mut target_page: Option<u32> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--output" | "-o" => {
                if i + 1 < args.len() {
                    output_dir = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("오류: --output 뒤에 폴더 경로가 필요합니다.");
                    return;
                }
            }
            "--page" | "-p" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<u32>() {
                        Ok(n) => target_page = Some(n),
                        Err(_) => {
                            eprintln!("오류: 페이지 번호가 올바르지 않습니다.");
                            return;
                        }
                    }
                    i += 2;
                } else {
                    eprintln!("오류: --page 뒤에 페이지 번호가 필요합니다.");
                    return;
                }
            }
            _ => {
                eprintln!("알 수 없는 옵션: {}", args[i]);
                i += 1;
            }
        }
    }

    let data = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return;
        }
    };

    let doc = match rhwp::wasm_api::HwpDocument::from_bytes(&data) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: HWP 파싱 실패 - {}", e);
            return;
        }
    };

    let page_count = doc.page_count();
    println!("문서 로드 완료: {} ({}페이지)", file_path, page_count);
    if page_count == 0 {
        eprintln!("오류: 문서에 페이지가 없습니다.");
        return;
    }

    let output_path = Path::new(&output_dir);
    if !output_path.exists() {
        if let Err(e) = fs::create_dir_all(output_path) {
            eprintln!(
                "오류: 출력 폴더를 생성할 수 없습니다 - {}: {}",
                output_dir, e
            );
            return;
        }
    }

    let pages: Vec<u32> = match target_page {
        Some(p) => {
            if p >= page_count {
                eprintln!(
                    "오류: 페이지 번호가 범위를 벗어났습니다 (0~{})",
                    page_count - 1
                );
                return;
            }
            vec![p]
        }
        None => (0..page_count).collect(),
    };

    let file_stem = Path::new(file_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("page");

    let assets_dir_name = format!("{}_assets", file_stem);
    let assets_dir_path = output_path.join(&assets_dir_name);
    let mut written_image_count: usize = 0;

    let mime_to_ext = |mime: &str| -> &'static str {
        match mime {
            "image/png" => "png",
            "image/jpeg" => "jpg",
            "image/gif" => "gif",
            "image/bmp" => "bmp",
            "image/webp" => "webp",
            _ => "bin",
        }
    };

    for page_num in &pages {
        match doc.extract_page_markdown_with_images_native(*page_num) {
            Ok((mut markdown, image_refs)) => {
                for (img_idx, (sec_idx, para_idx, control_idx, bin_data_id)) in
                    image_refs.iter().enumerate()
                {
                    let token = format!("[[RHWP_IMAGE:{}]]", img_idx + 1);

                    let try_control = match (sec_idx, para_idx, control_idx) {
                        (Some(si), Some(pi), Some(ci)) => Some((*si, *pi, *ci)),
                        _ => None,
                    };

                    let (mime, image_data) = if let Some((si, pi, ci)) = try_control {
                        match (
                            doc.get_control_image_mime_native(si, pi, &[], ci),
                            doc.get_control_image_data_native(si, pi, &[], ci),
                        ) {
                            (Ok(m), Ok(d)) => (m, d),
                            _ => {
                                if *bin_data_id == 0 {
                                    eprintln!(
                                        "경고: 페이지 {} 이미지 추출 실패 (s{} p{} c{}), fallback bin_data_id 없음",
                                        page_num, si, pi, ci
                                    );
                                    markdown = markdown.replace(&token, "");
                                    continue;
                                }
                                let fb_mime = match doc.get_bin_data_image_mime_native(*bin_data_id)
                                {
                                    Ok(m) => m,
                                    Err(e) => {
                                        eprintln!(
                                            "경고: 페이지 {} 이미지 MIME fallback 실패 (bin={}): {:?}",
                                            page_num, bin_data_id, e
                                        );
                                        markdown = markdown.replace(&token, "");
                                        continue;
                                    }
                                };
                                let fb_data = match doc.get_bin_data_image_data_native(*bin_data_id)
                                {
                                    Ok(d) => d,
                                    Err(e) => {
                                        eprintln!(
                                            "경고: 페이지 {} 이미지 데이터 fallback 실패 (bin={}): {:?}",
                                            page_num, bin_data_id, e
                                        );
                                        markdown = markdown.replace(&token, "");
                                        continue;
                                    }
                                };
                                (fb_mime, fb_data)
                            }
                        }
                    } else {
                        if *bin_data_id == 0 {
                            eprintln!(
                                "경고: 페이지 {} 이미지 추출 실패 (문서 좌표 없음, bin_data_id=0)",
                                page_num
                            );
                            markdown = markdown.replace(&token, "");
                            continue;
                        }
                        let fb_mime = match doc.get_bin_data_image_mime_native(*bin_data_id) {
                            Ok(m) => m,
                            Err(e) => {
                                eprintln!(
                                    "경고: 페이지 {} 이미지 MIME fallback 실패 (bin={}): {:?}",
                                    page_num, bin_data_id, e
                                );
                                markdown = markdown.replace(&token, "");
                                continue;
                            }
                        };
                        let fb_data = match doc.get_bin_data_image_data_native(*bin_data_id) {
                            Ok(d) => d,
                            Err(e) => {
                                eprintln!(
                                    "경고: 페이지 {} 이미지 데이터 fallback 실패 (bin={}): {:?}",
                                    page_num, bin_data_id, e
                                );
                                markdown = markdown.replace(&token, "");
                                continue;
                            }
                        };
                        (fb_mime, fb_data)
                    };

                    if !assets_dir_path.exists() {
                        if let Err(e) = fs::create_dir_all(&assets_dir_path) {
                            eprintln!(
                                "오류: 이미지 출력 폴더 생성 실패 - {}: {}",
                                assets_dir_path.display(),
                                e
                            );
                            markdown = markdown.replace(&token, "");
                            continue;
                        }
                    }

                    let ext = mime_to_ext(&mime);
                    let image_filename = format!(
                        "{}_p{:03}_img{:03}.{}",
                        file_stem,
                        page_num + 1,
                        img_idx + 1,
                        ext
                    );
                    let image_path = assets_dir_path.join(&image_filename);

                    if let Err(e) = fs::write(&image_path, &image_data) {
                        eprintln!("경고: 이미지 저장 실패 - {}: {}", image_path.display(), e);
                        markdown = markdown.replace(&token, "");
                        continue;
                    }

                    let image_link = format!(
                        "![image {}]({}/{})",
                        img_idx + 1,
                        assets_dir_name,
                        image_filename
                    );
                    markdown = markdown.replace(&token, &image_link);
                    written_image_count += 1;
                }

                if !markdown.ends_with('\n') {
                    markdown.push('\n');
                }

                let md_filename = if page_count == 1 {
                    format!("{}.md", file_stem)
                } else {
                    format!("{}_{:03}.md", file_stem, page_num + 1)
                };
                let md_path = output_path.join(&md_filename);

                match fs::write(&md_path, markdown.as_bytes()) {
                    Ok(_) => println!("  → {}", md_path.display()),
                    Err(e) => eprintln!("오류: Markdown 저장 실패 - {}: {}", md_path.display(), e),
                }
            }
            Err(e) => {
                eprintln!("오류: 페이지 {} Markdown 생성 실패 - {:?}", page_num, e);
            }
        }
    }

    if written_image_count > 0 {
        println!(
            "Markdown 내보내기 완료: {}개 MD 파일, {}개 이미지 → {}/",
            pages.len(),
            written_image_count,
            output_dir
        );
    } else {
        println!(
            "Markdown 내보내기 완료: {}개 MD 파일 → {}/",
            pages.len(),
            output_dir
        );
    }
}

fn show_info(args: &[String]) {
    if args.is_empty() {
        eprintln!("오류: HWP 파일 경로를 지정해주세요.");
        return;
    }

    let file_path = &args[0];

    // 파일 읽기
    let data = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return;
        }
    };

    let file_size = data.len();

    // HWP 파싱
    let doc = match rhwp::wasm_api::HwpDocument::from_bytes(&data) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: HWP 파싱 실패 - {}", e);
            return;
        }
    };

    let document = doc.document();

    println!("파일: {}", file_path);
    println!("크기: {} bytes", file_size);
    println!(
        "버전: {}.{}.{}.{}",
        document.header.version.major,
        document.header.version.minor,
        document.header.version.build,
        document.header.version.revision,
    );
    println!(
        "압축: {}",
        if document.header.compressed {
            "예"
        } else {
            "아니오"
        }
    );
    println!(
        "암호화: {}",
        if document.header.encrypted {
            "예"
        } else {
            "아니오"
        }
    );
    println!(
        "배포용: {}",
        if document.header.distribution {
            "예"
        } else {
            "아니오"
        }
    );
    println!("구역 수: {}", document.sections.len());
    println!("페이지 수: {}", doc.page_count());

    // 용지 정보
    for (sec_idx, section) in document.sections.iter().enumerate() {
        let page_def = &section.section_def.page_def;
        let orientation = if page_def.landscape {
            "가로"
        } else {
            "세로"
        };
        println!(
            "구역{} 용지: {}×{} HWPUNIT, 방향={} (여백: 좌{} 우{} 상{} 하{})",
            sec_idx,
            page_def.width,
            page_def.height,
            orientation,
            page_def.margin_left,
            page_def.margin_right,
            page_def.margin_top,
            page_def.margin_bottom,
        );
        println!(
            "  머리말여백={} 꼬리말여백={} 제본여백={}",
            page_def.margin_header, page_def.margin_footer, page_def.margin_gutter
        );
        if section.section_def.hide_empty_line {
            println!("  빈 줄 감추기: 활성");
        }
    }

    // 폰트 목록
    let lang_names = ["한글", "영어", "한자", "일어", "기타", "기호", "사용자"];
    for (i, fonts) in document.doc_info.font_faces.iter().enumerate() {
        if !fonts.is_empty() {
            let name = if i < lang_names.len() {
                lang_names[i]
            } else {
                "기타"
            };
            let font_names: Vec<String> = fonts
                .iter()
                .enumerate()
                .map(|(idx, f)| format!("[{}]{}", idx, f.name))
                .collect();
            println!("폰트({}): {}", name, font_names.join(", "));
        }
    }

    // 스타일 목록
    if !document.doc_info.styles.is_empty() {
        let style_names: Vec<&str> = document
            .doc_info
            .styles
            .iter()
            .map(|s| s.local_name.as_str())
            .collect();
        println!("스타일: {}", style_names.join(", "));
    }

    // 문단 통계
    let total_paras: usize = document.sections.iter().map(|s| s.paragraphs.len()).sum();
    println!("총 문단 수: {}", total_paras);

    // [Task #554] HWP3 → HWP5 변환본 식별 휴리스틱 정보
    // 한컴이 HWP3 → HWP5 변환 시 ParaShape/CharShape 를 거의 재사용하지 않고 매우 적은
    // 수만 생성한다. 직접 작성본은 작성자가 다양한 스타일을 사용하므로 비율이 paragraph
    // 와 비슷하거나 더 높다. 임계값 < 0.05 / < 0.15 로 27 fixture 100% 분류 (Stage 1).
    let ps_count = document.doc_info.para_shapes.len();
    let cs_count = document.doc_info.char_shapes.len();
    if total_paras > 0 {
        let ps_ratio = ps_count as f64 / total_paras as f64;
        let cs_ratio = cs_count as f64 / total_paras as f64;
        let origin = if total_paras > 50 && ps_ratio < 0.05 && cs_ratio < 0.15 {
            "HWP3 변환본 추정 (margin_bottom -1600 HU 보정 적용)"
        } else if total_paras <= 50 {
            "판정 불가 (문단 수 ≤ 50, 비율 왜곡 회피)"
        } else {
            "한컴 한글 직접 작성 추정"
        };
        println!("ParaShape: {} (PS/문단 = {:.3})", ps_count, ps_ratio);
        println!("CharShape: {} (CS/문단 = {:.3})", cs_count, cs_ratio);
        println!("Origin 추정: {}", origin);
    }

    // BinData 정보
    if !document.doc_info.bin_data_list.is_empty() {
        println!("BinData:");
        for (idx, bd) in document.doc_info.bin_data_list.iter().enumerate() {
            let type_str = match bd.data_type {
                rhwp::model::bin_data::BinDataType::Link => "Link",
                rhwp::model::bin_data::BinDataType::Embedding => "Embedding",
                rhwp::model::bin_data::BinDataType::Storage => "Storage",
            };
            let ext = bd.extension.as_deref().unwrap_or("?");
            // 로드된 데이터 크기 확인
            let loaded_size = document
                .bin_data_content
                .iter()
                .find(|c| c.id == bd.storage_id)
                .map(|c| c.data.len())
                .unwrap_or(0);
            println!(
                "  [{}] {} (ID: {}, ext: {}, loaded: {} bytes)",
                idx, type_str, bd.storage_id, ext, loaded_size
            );
        }
    }

    // 테이블 및 그림 정보
    use rhwp::model::control::Control;
    let mut table_idx = 0;
    let mut picture_idx = 0;

    fn count_pictures(ctrl: &Control, picture_idx: &mut usize, location: &str) {
        match ctrl {
            Control::Picture(pic) => {
                *picture_idx += 1;
                println!(
                    "그림{} [{}]: bin_data_id={}, size={}×{}",
                    *picture_idx,
                    location,
                    pic.image_attr.bin_data_id,
                    pic.common.width,
                    pic.common.height,
                );
            }
            Control::Table(table) => {
                // 표 내부 셀의 문단에서도 그림 검색
                for (cell_idx, cell) in table.cells.iter().enumerate() {
                    for (cp_idx, cp) in cell.paragraphs.iter().enumerate() {
                        for cc in &cp.controls {
                            let loc = format!("{}→셀{}:문단{}", location, cell_idx, cp_idx);
                            count_pictures(cc, picture_idx, &loc);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    for (sec_idx, section) in document.sections.iter().enumerate() {
        for (para_idx, para) in section.paragraphs.iter().enumerate() {
            for ctrl in &para.controls {
                let location = format!("구역{}:문단{}", sec_idx, para_idx);
                match ctrl {
                    Control::Table(table) => {
                        table_idx += 1;
                        let page_break_str = match table.page_break {
                            rhwp::model::table::TablePageBreak::None => "나누지 않음",
                            rhwp::model::table::TablePageBreak::CellBreak => "셀 단위 나눔",
                            rhwp::model::table::TablePageBreak::RowBreak => "나눔(행 단위)",
                        };
                        println!(
                            "표{} [{}]: {}행×{}열, 셀 {}개, 쪽나눔={} (attr=0x{:08x}), 제목반복={}",
                            table_idx,
                            location,
                            table.row_count,
                            table.col_count,
                            table.cells.len(),
                            page_break_str,
                            table.raw_table_record_attr,
                            table.repeat_header,
                        );
                        count_pictures(ctrl, &mut picture_idx, &location);
                    }
                    Control::Picture(_) => {
                        count_pictures(ctrl, &mut picture_idx, &location);
                    }
                    Control::Shape(shape) => {
                        use rhwp::model::shape::ShapeObject;
                        let s = shape.as_ref();
                        let shape_type = s.shape_name();
                        let common = s.common();
                        let border_info = match shape.as_ref() {
                            ShapeObject::Rectangle(r) => format!(
                                ", border(color={:#010x}, width={}, attr={:#010x})",
                                r.drawing.border_line.color,
                                r.drawing.border_line.width,
                                r.drawing.border_line.attr,
                            ),
                            ShapeObject::Line(l) => format!(
                                ", border(color={:#010x}, width={}, attr={:#010x})",
                                l.drawing.border_line.color,
                                l.drawing.border_line.width,
                                l.drawing.border_line.attr,
                            ),
                            _ => String::new(),
                        };
                        println!(
                            "도형 [{}]: {}, size={}×{}, treat_as_char={}{}",
                            location,
                            shape_type,
                            common.width,
                            common.height,
                            common.treat_as_char,
                            border_info,
                        );
                        // 그룹 자식 상세 정보
                        if let ShapeObject::Group(g) = shape.as_ref() {
                            for (i, child) in g.children.iter().enumerate() {
                                let ctype = child.shape_name();
                                let cattr = child.shape_attr();
                                let eff_w = (cattr.current_width as f64 * cattr.render_sx) as i32;
                                let eff_h = (cattr.current_height as f64 * cattr.render_sy) as i32;
                                println!("  자식[{}]: {}, orig={}×{}, scale=({:.3},{:.3}), eff={}×{} at ({:.0},{:.0})",
                                    i, ctype,
                                    cattr.current_width, cattr.current_height,
                                    cattr.render_sx, cattr.render_sy,
                                    eff_w, eff_h,
                                    cattr.render_tx, cattr.render_ty);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

/// HWPUNIT(u32)을 mm로 변환
fn hu_to_mm(hu: u32) -> f64 {
    hu as f64 * 25.4 / 7200.0
}

/// HWPUNIT(i32)을 mm로 변환
fn hu_to_mm_i(hu: i32) -> f64 {
    hu as f64 * 25.4 / 7200.0
}

fn dump_note_shape(args: &[String]) {
    if args.is_empty() {
        eprintln!("사용법: rhwp dump-note-shape <파일.hwp|파일.hwpx>");
        return;
    }

    let file_path = &args[0];
    let data = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return;
        }
    };

    let doc = match rhwp::wasm_api::HwpDocument::from_bytes(&data) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: HWP 파싱 실패 - {}", e);
            return;
        }
    };

    let sections: Vec<serde_json::Value> = doc
        .document()
        .sections
        .iter()
        .enumerate()
        .map(|(idx, section)| {
            serde_json::json!({
                "section": idx,
                "footnoteShape": note_shape_json(&section.section_def.footnote_shape),
                "endnoteShape": note_shape_json(&section.section_def.endnote_shape),
            })
        })
        .collect();

    let value = serde_json::json!({
        "file": file_path,
        "sections": sections,
    });
    match serde_json::to_string_pretty(&value) {
        Ok(text) => println!("{}", text),
        Err(e) => eprintln!("오류: JSON 생성 실패 - {}", e),
    }
}

fn note_shape_json(shape: &rhwp::model::footnote::FootnoteShape) -> serde_json::Value {
    serde_json::json!({
        "raw": {
            "attr": shape.attr,
            "numberFormat": format!("{:?}", shape.number_format),
            "userChar": shape.user_char.to_string(),
            "prefixChar": shape.prefix_char.to_string(),
            "suffixChar": shape.suffix_char.to_string(),
            "startNumber": shape.start_number,
            "separatorLength": hu_json(shape.separator_length as i32),
            "separatorMarginTop": hu_json(shape.separator_margin_top as i32),
            "separatorMarginBottom": hu_json(shape.separator_margin_bottom as i32),
            "noteSpacing": hu_json(shape.note_spacing as i32),
            "separatorLineType": shape.separator_line_type,
            "separatorLineWidth": shape.separator_line_width,
            "separatorColor": format!("0x{:08x}", shape.separator_color),
            "numbering": format!("{:?}", shape.numbering),
            "placement": format!("{:?}", shape.placement),
            "numberCodeSuperscript": shape.number_code_superscript,
            "printInlineAfterText": shape.print_inline_after_text,
            "rawUnknown": hu_json(shape.raw_unknown as i32),
        },
        "ui": {
            "separatorAbove": hu_json(shape.separator_above_margin_hu() as i32),
            "separatorBelow": hu_json(shape.separator_below_margin_hu() as i32),
            "betweenNotes": hu_json(shape.between_notes_margin_hu() as i32),
        },
    })
}

fn hu_json(hu: i32) -> serde_json::Value {
    serde_json::json!({
        "hu": hu,
        "mm": rounded_mm(hu),
    })
}

fn rounded_mm(hu: i32) -> f64 {
    (hu_to_mm_i(hu) * 1000.0).round() / 1000.0
}

fn dump_pages(args: &[String]) {
    if args.is_empty() {
        eprintln!("사용법: rhwp dump-pages <파일.hwp> [-p <페이지번호>]");
        return;
    }

    let file_path = &args[0];
    let mut target_page: Option<u32> = None;
    let mut respect_vpos_reset = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--page" | "-p" => {
                if i + 1 < args.len() {
                    target_page = args[i + 1].parse().ok();
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "--respect-vpos-reset" => {
                respect_vpos_reset = true;
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }

    let data = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return;
        }
    };

    let mut doc = match rhwp::wasm_api::HwpDocument::from_bytes(&data) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: HWP 파싱 실패 - {}", e);
            return;
        }
    };

    if respect_vpos_reset {
        doc.set_respect_vpos_reset(true);
    }

    println!("문서 로드: {} ({}페이지)", file_path, doc.page_count());
    print!("{}", doc.dump_page_items(target_page));
}

fn dump_endnote_lines(args: &[String]) {
    if args.len() < 4 {
        eprintln!(
            "사용법: rhwp dump-endnote-lines <파일.hwp> <section> <para> <control> [note-para]"
        );
        return;
    }

    let file_path = &args[0];
    let section_idx = match args[1].parse::<usize>() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("오류: section 인덱스 파싱 실패 - {}", e);
            return;
        }
    };
    let para_idx = match args[2].parse::<usize>() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("오류: para 인덱스 파싱 실패 - {}", e);
            return;
        }
    };
    let control_idx = match args[3].parse::<usize>() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("오류: control 인덱스 파싱 실패 - {}", e);
            return;
        }
    };
    let target_note_para = if args.len() >= 5 {
        match args[4].parse::<usize>() {
            Ok(v) => Some(v),
            Err(e) => {
                eprintln!("오류: note-para 인덱스 파싱 실패 - {}", e);
                return;
            }
        }
    } else {
        None
    };

    let data = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return;
        }
    };

    let doc = match rhwp::wasm_api::HwpDocument::from_bytes(&data) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: HWP 파싱 실패 - {}", e);
            return;
        }
    };

    let document = doc.document();
    let Some(section) = document.sections.get(section_idx) else {
        eprintln!("오류: section {} 범위 초과", section_idx);
        return;
    };
    let Some(source_para) = section.paragraphs.get(para_idx) else {
        eprintln!("오류: para {} 범위 초과", para_idx);
        return;
    };
    let Some(ctrl) = source_para.controls.get(control_idx) else {
        eprintln!("오류: control {} 범위 초과", control_idx);
        return;
    };

    let rhwp::model::control::Control::Endnote(endnote) = ctrl else {
        eprintln!(
            "오류: s{}:p{}:ci{} 는 미주가 아닙니다 ({})",
            section_idx,
            para_idx,
            control_idx,
            control_kind(ctrl)
        );
        return;
    };

    println!(
        "문서: {} source=s{}:p{}:ci{} endnote_no={} note_paras={}",
        file_path,
        section_idx,
        para_idx,
        control_idx,
        endnote.number,
        endnote.paragraphs.len()
    );
    println!("source_text={}", brief_text(&source_para.text, 120));
    println!(
        "source_control_positions={}",
        format_control_positions(source_para)
    );

    for (note_para_idx, para) in endnote.paragraphs.iter().enumerate() {
        if target_note_para.is_some_and(|target| target != note_para_idx) {
            continue;
        }
        println!(
            "\n-- note_para={} source=s{}:p{}:ci{}:note{} --",
            note_para_idx, section_idx, para_idx, control_idx, note_para_idx
        );
        dump_paragraph_line_trace(para);
    }
}

fn dump_paragraph_line_trace(para: &rhwp::model::paragraph::Paragraph) {
    use rhwp::model::control::Control;

    let composed = rhwp::renderer::composer::compose_paragraph(para);
    let control_positions = para.control_text_positions();

    println!(
        "para text_len={} char_count={} controls={} line_segs={} char_offsets={} text={}",
        para.text.chars().count(),
        para.char_count,
        para.controls.len(),
        para.line_segs.len(),
        format_u32_list(&para.char_offsets),
        brief_text(&para.text, 160)
    );
    for (i, seg) in para.line_segs.iter().enumerate() {
        println!(
            "  line_seg[{i}] ts={} char={} vpos={} lh={} th={} bl={} gap={} cs={} sw={} tag=0x{:08x}",
            seg.text_start,
            para.utf16_pos_to_char_idx(seg.text_start),
            seg.vertical_pos,
            seg.line_height,
            seg.text_height,
            seg.baseline_distance,
            seg.line_spacing,
            seg.column_start,
            seg.segment_width,
            seg.tag
        );
    }

    if para.controls.is_empty() {
        println!("  controls=[]");
    } else {
        for (ci, ctrl) in para.controls.iter().enumerate() {
            let pos = control_positions.get(ci).copied().unwrap_or(usize::MAX);
            match ctrl {
                Control::Equation(eq) => println!(
                    "  control[{ci}] kind=Equation pos={} tac=true size={}x{} font={} baseline={} script={}",
                    pos,
                    eq.common.width,
                    eq.common.height,
                    eq.font_size,
                    eq.baseline,
                    brief_text(&eq.script, 100)
                ),
                Control::Picture(pic) => println!(
                    "  control[{ci}] kind=Picture pos={} tac={} size={}x{}",
                    pos, pic.common.treat_as_char, pic.common.width, pic.common.height
                ),
                Control::Shape(shape) => {
                    let common = shape.common();
                    println!(
                        "  control[{ci}] kind=Shape pos={} tac={} size={}x{}",
                        pos, common.treat_as_char, common.width, common.height
                    );
                }
                Control::Table(table) => println!(
                    "  control[{ci}] kind=Table pos={} tac={} rows={} cols={}",
                    pos,
                    table.common.treat_as_char,
                    table.row_count,
                    table.col_count
                ),
                other => println!(
                    "  control[{ci}] kind={} pos={} tac=false",
                    control_kind(other),
                    pos
                ),
            }
        }
    }

    println!("  composed_lines={}", composed.lines.len());
    for (li, line) in composed.lines.iter().enumerate() {
        let next_start = composed
            .lines
            .get(li + 1)
            .map(|next| next.char_start)
            .unwrap_or_else(|| {
                line.char_start
                    + line
                        .runs
                        .iter()
                        .map(|run| run.text.chars().count())
                        .sum::<usize>()
                    + usize::from(line.has_line_break)
            });
        println!(
            "    line[{li}] char={}..{} runs={} break={} lh={} bl={} gap={} cs={} sw={} layout_tacs={}",
            line.char_start,
            next_start,
            format_runs(&line.runs),
            line.has_line_break,
            line.line_height,
            line.baseline_distance,
            line.line_spacing,
            line.column_start,
            line.segment_width,
            format_layout_tac_hits(&composed, li)
        );
    }

    if composed.tac_controls.is_empty() {
        println!("  tac_controls=[]");
    } else {
        println!("  tac_controls:");
        for (pos, width_hu, ci) in &composed.tac_controls {
            let line_hits = composed
                .lines
                .iter()
                .enumerate()
                .filter_map(|(li, line)| {
                    let start = line.char_start;
                    let end = composed
                        .lines
                        .get(li + 1)
                        .map(|next| next.char_start)
                        .unwrap_or_else(|| {
                            line.char_start
                                + line
                                    .runs
                                    .iter()
                                    .map(|run| run.text.chars().count())
                                    .sum::<usize>()
                                + usize::from(line.has_line_break)
                        });
                    if if end > start {
                        *pos >= start && *pos < end
                    } else {
                        *pos == start
                    } {
                        Some(li.to_string())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .join(",");
            println!(
                "    tac ci={} pos={} width={} strict_line_candidates=[{}]",
                ci, pos, width_hu, line_hits
            );
        }
    }
}

fn format_layout_tac_hits(
    composed: &rhwp::renderer::composer::ComposedParagraph,
    line_idx: usize,
) -> String {
    let Some(line) = composed.lines.get(line_idx) else {
        return "[]".to_string();
    };
    if composed.tac_controls.is_empty() {
        return "[]".to_string();
    }

    let mut hits = Vec::new();
    if line.runs.is_empty() {
        let start = line.char_start;
        let end = composed
            .lines
            .get(line_idx + 1)
            .map(|next| next.char_start)
            .unwrap_or(usize::MAX);
        for (pos, _, ci) in &composed.tac_controls {
            if *pos >= start && *pos < end {
                hits.push(format!("ci{}@{}:empty", ci, pos));
            }
        }
    } else {
        let mut run_start = line.char_start;
        for (run_idx, run) in line.runs.iter().enumerate() {
            let run_len = run.text.chars().count();
            let run_end = run_start + run_len;
            let next_line_starts_at_run_end = composed
                .lines
                .get(line_idx + 1)
                .is_some_and(|next| next.char_start == run_end);
            let allow_end = run_idx == line.runs.len() - 1 && !next_line_starts_at_run_end;
            for (pos, _, ci) in &composed.tac_controls {
                if *pos >= run_start && (*pos < run_end || (allow_end && *pos == run_end)) {
                    hits.push(format!(
                        "ci{}@{}:run{}+{}",
                        ci,
                        pos,
                        run_idx,
                        pos.saturating_sub(run_start)
                    ));
                }
            }
            run_start = run_end;
        }
    }

    if hits.is_empty() {
        "[]".to_string()
    } else {
        format!("[{}]", hits.join(","))
    }
}

fn control_kind(ctrl: &rhwp::model::control::Control) -> &'static str {
    use rhwp::model::control::Control;
    match ctrl {
        Control::SectionDef(_) => "SectionDef",
        Control::ColumnDef(_) => "ColumnDef",
        Control::Table(_) => "Table",
        Control::Shape(_) => "Shape",
        Control::Picture(_) => "Picture",
        Control::Header(_) => "Header",
        Control::Footer(_) => "Footer",
        Control::Footnote(_) => "Footnote",
        Control::Endnote(_) => "Endnote",
        Control::AutoNumber(_) => "AutoNumber",
        Control::NewNumber(_) => "NewNumber",
        Control::PageNumberPos(_) => "PageNumberPos",
        Control::Bookmark(_) => "Bookmark",
        Control::Hyperlink(_) => "Hyperlink",
        Control::Ruby(_) => "Ruby",
        Control::CharOverlap(_) => "CharOverlap",
        Control::PageHide(_) => "PageHide",
        Control::HiddenComment(_) => "HiddenComment",
        Control::Equation(_) => "Equation",
        Control::Field(_) => "Field",
        Control::Form(_) => "Form",
        Control::Unknown(_) => "Unknown",
    }
}

fn format_control_positions(para: &rhwp::model::paragraph::Paragraph) -> String {
    let positions = para.control_text_positions();
    if positions.is_empty() {
        return "[]".to_string();
    }
    positions
        .iter()
        .enumerate()
        .map(|(ci, pos)| {
            let kind = para.controls.get(ci).map(control_kind).unwrap_or("?");
            format!("{ci}:{kind}@{pos}")
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn format_runs(runs: &[rhwp::renderer::composer::ComposedTextRun]) -> String {
    if runs.is_empty() {
        return "[]".to_string();
    }
    let parts = runs
        .iter()
        .map(|run| {
            format!(
                "cs{}:l{}:'{}'",
                run.char_style_id,
                run.lang_index,
                brief_text(&run.text, 40)
            )
        })
        .collect::<Vec<_>>();
    format!("[{}]", parts.join("|"))
}

fn format_u32_list(values: &[u32]) -> String {
    if values.is_empty() {
        return "[]".to_string();
    }
    if values.len() <= 16 {
        return format!("{:?}", values);
    }
    let head = values
        .iter()
        .take(8)
        .map(|v| v.to_string())
        .collect::<Vec<_>>()
        .join(",");
    let tail = values
        .iter()
        .rev()
        .take(4)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .map(|v| v.to_string())
        .collect::<Vec<_>>()
        .join(",");
    format!("[{}...{};len={}]", head, tail, values.len())
}

fn brief_text(text: &str, max_chars: usize) -> String {
    let mut out = String::new();
    for (count, ch) in text.chars().enumerate() {
        if count >= max_chars {
            out.push('…');
            break;
        }
        match ch {
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{FFFC}' => out.push('□'),
            c if c.is_control() => out.push_str(&format!("\\u{{{:04X}}}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

fn dump_controls(args: &[String]) {
    if args.is_empty() {
        eprintln!("오류: HWP 파일 경로를 지정해주세요.");
        eprintln!("사용법: rhwp dump <파일.hwp> [--section <번호>] [--para <번호>]");
        return;
    }

    let file_path = &args[0];
    let mut filter_section: Option<usize> = None;
    let mut filter_para: Option<usize> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--section" | "-s" => {
                if i + 1 < args.len() {
                    filter_section = args[i + 1].parse().ok();
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "--para" | "-p" => {
                if i + 1 < args.len() {
                    filter_para = args[i + 1].parse().ok();
                    i += 2;
                } else {
                    i += 1;
                }
            }
            _ => {
                i += 1;
            }
        }
    }

    let data = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return;
        }
    };

    let doc = match rhwp::wasm_api::HwpDocument::from_bytes(&data) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: HWP 파싱 실패 - {}", e);
            return;
        }
    };

    let document = doc.document();

    // border_fill 상세 덤프 (필터 없을 때 전체, 필터 있을 때 관련 bf만)
    if filter_section.is_none() && filter_para.is_none() {
        for (i, bf) in document.doc_info.border_fills.iter().enumerate() {
            let fill = &bf.fill;
            let solid_info = fill
                .solid
                .as_ref()
                .map(|s| {
                    format!(
                        "bg=#{:06X} pat_type={} pat_color=#{:06X}",
                        s.background_color, s.pattern_type, s.pattern_color
                    )
                })
                .unwrap_or_default();
            let grad_info = if fill.gradient.is_some() {
                " gradient"
            } else {
                ""
            };
            let img_info = fill
                .image
                .as_ref()
                .map(|img| {
                    format!(
                        " image(bin_id={}, mode={:?}, brightness={}, contrast={}, effect={})",
                        img.bin_data_id, img.fill_mode, img.brightness, img.contrast, img.effect
                    )
                })
                .unwrap_or_default();
            println!(
                "  border_fill[{}] fill_type={:?} {}{}{}",
                i, fill.fill_type, solid_info, grad_info, img_info
            );
        }
    }

    use rhwp::model::control::Control;
    use rhwp::model::paragraph::ColumnBreakType;
    use rhwp::model::shape::{HorzRelTo, ShapeObject, TextWrap, VertRelTo};

    let vert_str = |v: &VertRelTo| -> &str {
        match v {
            VertRelTo::Paper => "용지",
            VertRelTo::Page => "쪽",
            VertRelTo::Para => "문단",
        }
    };
    let horz_str = |h: &HorzRelTo| -> &str {
        match h {
            HorzRelTo::Paper => "용지",
            HorzRelTo::Page => "쪽",
            HorzRelTo::Column => "단",
            HorzRelTo::Para => "문단",
        }
    };
    let wrap_str = |w: &TextWrap| -> &str {
        match w {
            TextWrap::Square => "어울림",
            TextWrap::Tight => "빈 공간 채움",
            TextWrap::Through => "통과",
            TextWrap::TopAndBottom => "자리차지",
            TextWrap::BehindText => "글뒤로",
            TextWrap::InFrontOfText => "글앞으로",
        }
    };
    let break_str = |b: &ColumnBreakType| -> &str {
        match b {
            ColumnBreakType::None => "",
            ColumnBreakType::Section => "[구역나누기]",
            ColumnBreakType::MultiColumn => "[다단나누기]",
            ColumnBreakType::Page => "[쪽나누기]",
            ColumnBreakType::Column => "[단나누기]",
        }
    };

    // 도형 공통 속성 출력 헬퍼
    let dump_common = |c: &rhwp::model::shape::CommonObjAttr, indent: &str| {
        println!(
            "{}  크기: {:.1}mm × {:.1}mm ({}×{} HU)",
            indent,
            hu_to_mm(c.width),
            hu_to_mm(c.height),
            c.width,
            c.height
        );
        println!(
            "{}  위치: 가로={} 오프셋={:.1}mm({}) 정렬={:?}, 세로={} 오프셋={:.1}mm({}) 정렬={:?}",
            indent,
            horz_str(&c.horz_rel_to),
            hu_to_mm(c.horizontal_offset),
            c.horizontal_offset,
            c.horz_align,
            vert_str(&c.vert_rel_to),
            hu_to_mm(c.vertical_offset),
            c.vertical_offset,
            c.vert_align
        );
        println!(
            "{}  배치: {}, 글자처럼={}, z={}",
            indent,
            wrap_str(&c.text_wrap),
            c.treat_as_char,
            c.z_order
        );
        println!(
            "{}  바깥 여백: left={:.2}mm({}) right={:.2}mm({}) top={:.2}mm({}) bottom={:.2}mm({})",
            indent,
            hu_to_mm_i(c.margin.left as i32),
            c.margin.left,
            hu_to_mm_i(c.margin.right as i32),
            c.margin.right,
            hu_to_mm_i(c.margin.top as i32),
            c.margin.top,
            hu_to_mm_i(c.margin.bottom as i32),
            c.margin.bottom
        );
    };

    // 도형 요소 속성 출력 헬퍼
    let dump_shape_attr = |sa: &rhwp::model::shape::ShapeComponentAttr, indent: &str| {
        let eff_w = (sa.current_width as f64 * sa.render_sx) as u32;
        let eff_h = (sa.current_height as f64 * sa.render_sy) as u32;
        println!("{}  요소: orig={}×{}, curr={}×{}, M=[{:.3},{:.3},{:.0}; {:.3},{:.3},{:.0}], offset=({},{}), eff={:.1}mm×{:.1}mm",
            indent, sa.original_width, sa.original_height,
            sa.current_width, sa.current_height,
            sa.render_sx, sa.render_b, sa.render_tx,
            sa.render_c, sa.render_sy, sa.render_ty,
            sa.offset_x, sa.offset_y,
            hu_to_mm(eff_w), hu_to_mm(eff_h));
        if sa.horz_flip || sa.vert_flip || sa.rotation_angle != 0 {
            println!(
                "{}  변환: 뒤집기=({},{}), 회전={}",
                indent, sa.horz_flip, sa.vert_flip, sa.rotation_angle
            );
        }
    };

    // 재귀적 도형 덤프
    fn dump_shape(
        shape: &ShapeObject,
        indent: &str,
        dump_common_fn: &dyn Fn(&rhwp::model::shape::CommonObjAttr, &str),
        dump_sa_fn: &dyn Fn(&rhwp::model::shape::ShapeComponentAttr, &str),
    ) {
        match shape {
            ShapeObject::Line(s) => {
                println!(
                    "{}[직선] start=({},{}) end=({},{})",
                    indent, s.start.x, s.start.y, s.end.x, s.end.y
                );
                println!(
                    "{}  선: color={:#010x}, width={}, style={:#06x}",
                    indent,
                    s.drawing.border_line.color,
                    s.drawing.border_line.width,
                    s.drawing.border_line.attr
                );
                dump_common_fn(&s.common, indent);
                dump_sa_fn(&s.drawing.shape_attr, indent);
            }
            ShapeObject::Rectangle(s) => {
                println!("{}[사각형] round={}%", indent, s.round_rate);
                println!(
                    "{}  선: color={:#010x}, width={}, style={:#06x}",
                    indent,
                    s.drawing.border_line.color,
                    s.drawing.border_line.width,
                    s.drawing.border_line.attr
                );
                println!(
                    "{}  채우기: {:?}{}",
                    indent,
                    s.drawing.fill.fill_type,
                    if let Some(ref img) = s.drawing.fill.image {
                        format!(
                            ", image=bin_data_id={}, mode={:?}",
                            img.bin_data_id, img.fill_mode
                        )
                    } else {
                        String::new()
                    }
                );
                dump_common_fn(&s.common, indent);
                dump_sa_fn(&s.drawing.shape_attr, indent);
                if let Some(tb) = &s.drawing.text_box {
                    println!("{}  글상자: list_attr={:#010x}, margins=({},{},{},{}), max_width={}, paras={}",
                        indent, tb.list_attr, tb.margin_left, tb.margin_right, tb.margin_top, tb.margin_bottom,
                        tb.max_width, tb.paragraphs.len());
                    for (tpi, tp) in tb.paragraphs.iter().enumerate() {
                        let text_preview = if tp.text.is_empty() {
                            "(빈)".to_string()
                        } else if tp.text.chars().count() > 60 {
                            let end = tp
                                .text
                                .char_indices()
                                .nth(60)
                                .map(|(i, _)| i)
                                .unwrap_or(tp.text.len());
                            format!("\"{}...\"", &tp.text[..end])
                        } else {
                            format!("\"{}\"", tp.text)
                        };
                        println!(
                            "{}    p[{}]: ps_id={}, cc={}, text={}, ls_count={}, ctrls={}",
                            indent,
                            tpi,
                            tp.para_shape_id,
                            tp.char_count,
                            text_preview,
                            tp.line_segs.len(),
                            tp.controls.len()
                        );
                        for (li, ls) in tp.line_segs.iter().enumerate() {
                            println!(
                                "{}      ls[{}]: vpos={}, lh={}, th={}, bl={}, cs={}, sw={}",
                                indent,
                                li,
                                ls.vertical_pos,
                                ls.line_height,
                                ls.text_height,
                                ls.baseline_distance,
                                ls.column_start,
                                ls.segment_width
                            );
                        }
                    }
                }
            }
            ShapeObject::Ellipse(s) => {
                println!("{}[타원]", indent);
                dump_common_fn(&s.common, indent);
                dump_sa_fn(&s.drawing.shape_attr, indent);
            }
            ShapeObject::Arc(s) => {
                println!("{}[호]", indent);
                dump_common_fn(&s.common, indent);
                dump_sa_fn(&s.drawing.shape_attr, indent);
            }
            ShapeObject::Polygon(s) => {
                println!("{}[다각형] points={}", indent, s.points.len());
                dump_common_fn(&s.common, indent);
                dump_sa_fn(&s.drawing.shape_attr, indent);
                // 좌표 범위 출력
                if !s.points.is_empty() {
                    let min_x = s.points.iter().map(|p| p.x).min().unwrap();
                    let max_x = s.points.iter().map(|p| p.x).max().unwrap();
                    let min_y = s.points.iter().map(|p| p.y).min().unwrap();
                    let max_y = s.points.iter().map(|p| p.y).max().unwrap();
                    println!(
                        "{}  좌표범위: x=[{},{}], y=[{},{}]",
                        indent, min_x, max_x, min_y, max_y
                    );
                }
            }
            ShapeObject::Curve(s) => {
                println!("{}[곡선] points={}", indent, s.points.len());
                dump_common_fn(&s.common, indent);
                dump_sa_fn(&s.drawing.shape_attr, indent);
            }
            ShapeObject::Group(g) => {
                println!("{}[묶음] children={}", indent, g.children.len());
                dump_common_fn(&g.common, indent);
                dump_sa_fn(&g.shape_attr, indent);
                let child_indent = format!("{}  ", indent);
                for (ci, child) in g.children.iter().enumerate() {
                    print!("{}child[{}] ", child_indent, ci);
                    dump_shape(child, &child_indent, dump_common_fn, dump_sa_fn);
                }
            }
            ShapeObject::Picture(p) => {
                println!("{}[그림] bin_data_id={}", indent, p.image_attr.bin_data_id);
                dump_common_fn(&p.common, indent);
                dump_sa_fn(&p.shape_attr, indent);
            }
            ShapeObject::Chart(c) => {
                println!(
                    "{}[차트] type={:?} series={} raw_chart_data={}B",
                    indent,
                    c.chart_type,
                    c.series.len(),
                    c.raw_chart_data.len()
                );
                dump_common_fn(&c.common, indent);
                dump_sa_fn(&c.drawing.shape_attr, indent);
            }
            ShapeObject::Ole(o) => {
                println!(
                    "{}[OLE] bin_data_id={} extent={}x{} flags=0x{:02X} raw={}B",
                    indent,
                    o.bin_data_id,
                    o.extent_x,
                    o.extent_y,
                    o.flags,
                    o.raw_tag_data.len()
                );
                dump_common_fn(&o.common, indent);
                dump_sa_fn(&o.drawing.shape_attr, indent);
            }
        }
    }

    for (sec_idx, section) in document.sections.iter().enumerate() {
        if let Some(fs) = filter_section {
            if sec_idx != fs {
                continue;
            }
        }

        let pd = &section.section_def.page_def;
        println!("=== 구역 {} ===", sec_idx);
        println!(
            "  용지: {:.1}mm × {:.1}mm ({}×{} HU), {}",
            hu_to_mm(pd.width),
            hu_to_mm(pd.height),
            pd.width,
            pd.height,
            if pd.landscape { "가로" } else { "세로" }
        );
        println!(
            "  여백: 좌={:.1} 우={:.1} 상={:.1} 하={:.1} 머리말={:.1} 꼬리말={:.1} mm",
            hu_to_mm(pd.margin_left),
            hu_to_mm(pd.margin_right),
            hu_to_mm(pd.margin_top),
            hu_to_mm(pd.margin_bottom),
            hu_to_mm(pd.margin_header),
            hu_to_mm(pd.margin_footer)
        );

        // 바탕쪽 정보
        if !section.section_def.master_pages.is_empty() {
            println!("  바탕쪽: {}개", section.section_def.master_pages.len());
            for (mi, mp) in section.section_def.master_pages.iter().enumerate() {
                println!("    [{}] {:?}, 문단 {}개, 영역 {}×{} HU, is_ext={}, overlap={}, ext_flags=0x{:04X}, text_ref={}, num_ref={}",
                    mi, mp.apply_to, mp.paragraphs.len(), mp.text_width, mp.text_height,
                    mp.is_extension, mp.overlap, mp.ext_flags, mp.text_ref, mp.num_ref);
                for (pi, para) in mp.paragraphs.iter().enumerate() {
                    println!(
                        "      p[{}]: cc={}, text=\"{}\"",
                        pi,
                        para.controls.len(),
                        if para.text.is_empty() {
                            "(빈 문단)".to_string()
                        } else {
                            para.text.chars().take(30).collect::<String>()
                        }
                    );
                    for (ci, ctrl) in para.controls.iter().enumerate() {
                        let ctrl_name = match ctrl {
                            Control::Table(t) => {
                                let cell_texts: Vec<String> = t
                                    .cells
                                    .iter()
                                    .take(3)
                                    .map(|c| {
                                        c.paragraphs
                                            .iter()
                                            .map(|p| p.text.chars().take(20).collect::<String>())
                                            .collect::<Vec<_>>()
                                            .join("|")
                                    })
                                    .collect();
                                format!("표({}x{}, tac={}, wrap={:?}, vert={:?}/{}, horz={:?}/{}, size={}x{}, cells=[{}])",
                                    t.row_count, t.col_count, t.common.treat_as_char,
                                    t.common.text_wrap, t.common.vert_rel_to, t.common.vertical_offset,
                                    t.common.horz_rel_to, t.common.horizontal_offset,
                                    t.common.width, t.common.height,
                                    cell_texts.join("; "))
                            }
                            Control::Shape(s) => {
                                let mut desc = format!("도형(ctrl_id=0x{:08X}, w={}, h={}, attr=0x{:08X}, wc={:?}, hc={:?})",
                                    s.common().ctrl_id, s.common().width, s.common().height,
                                    s.common().attr, s.common().width_criterion, s.common().height_criterion);
                                // TextBox 내용 출력
                                if let Some(tb) = s.drawing().and_then(|d| d.text_box.as_ref()) {
                                    desc += &format!(" 글상자({}문단)", tb.paragraphs.len());
                                    for (tpi, tp) in tb.paragraphs.iter().enumerate() {
                                        let tp_text: String = tp.text.chars().take(20).collect();
                                        desc += &format!(
                                            "\n          tb_p[{}]: cc={} text=\"{}\"",
                                            tpi,
                                            tp.controls.len(),
                                            tp_text
                                        );
                                        for (tci, tc) in tp.controls.iter().enumerate() {
                                            let tc_name = match tc {
                                                Control::AutoNumber(an) => {
                                                    format!("자동번호({:?})", an.number_type)
                                                }
                                                _ => format!("{:?}", std::mem::discriminant(tc)),
                                            };
                                            desc += &format!(
                                                "\n            tb_ctrl[{}]: {}",
                                                tci, tc_name
                                            );
                                        }
                                    }
                                }
                                desc
                            }
                            Control::Picture(p) => {
                                let wm = p
                                    .image_attr
                                    .watermark_preset()
                                    .map(|s| format!(", watermark={}", s))
                                    .unwrap_or_default();
                                format!(
                                    "그림(bin_id={}, w={}, h={}, tac={}{})",
                                    p.image_attr.bin_data_id,
                                    p.common.width,
                                    p.common.height,
                                    p.common.treat_as_char,
                                    wm
                                )
                            }
                            Control::Header(_) => "머리말".to_string(),
                            Control::Footer(_) => "꼬리말".to_string(),
                            _ => format!("{:?}", std::mem::discriminant(ctrl)),
                        };
                        println!("        ctrl[{}]: {}", ci, ctrl_name);
                    }
                }
            }
        }
        if section.section_def.hide_master_page {
            println!("  바탕쪽 감추기: true");
        }

        for (para_idx, para) in section.paragraphs.iter().enumerate() {
            if let Some(fp) = filter_para {
                if para_idx != fp {
                    continue;
                }
            }

            let text_preview = if para.text.is_empty() {
                "(빈 문단)".to_string()
            } else {
                let preview = if para.text.chars().count() > 50 {
                    let end = para
                        .text
                        .char_indices()
                        .nth(50)
                        .map(|(i, _)| i)
                        .unwrap_or(para.text.len());
                    format!("\"{}...\"", &para.text[..end])
                } else {
                    format!("\"{}\"", para.text)
                };
                preview
            };

            let break_info = break_str(&para.column_type);
            println!(
                "\n--- 문단 {}.{} --- cc={}, text_len={}, controls={} {}",
                sec_idx,
                para_idx,
                para.char_count,
                para.text.chars().count(),
                para.controls.len(),
                break_info
            );
            println!("  텍스트: {}", text_preview);
            // char_shapes 출력
            if !para.char_shapes.is_empty() {
                let text_chars: Vec<char> = para.text.chars().collect();
                for (ci, cs) in para.char_shapes.iter().enumerate() {
                    let next_pos = para
                        .char_shapes
                        .get(ci + 1)
                        .map(|n| n.start_pos)
                        .unwrap_or(u32::MAX);
                    let char_at = text_chars
                        .iter()
                        .enumerate()
                        .find(|(i, _)| {
                            if *i < para.char_offsets.len() {
                                para.char_offsets[*i] >= cs.start_pos
                                    && para.char_offsets[*i] < next_pos
                            } else {
                                false
                            }
                        })
                        .map(|(_, c)| *c);
                    if let Some(chs) = document.doc_info.char_shapes.get(cs.char_shape_id as usize)
                    {
                        let bold = (chs.attr & 0x02) != 0;
                        let spacing = chs.spacings[0]; // 한국어 자간
                        let ratio = chs.ratios[0]; // 한국어 장평
                        println!(
                            "  [CS] pos={} id={} bold={} spacing={}% ratio={}% base={} attr=0x{:08X} text=#{:06X} shade=#{:06X} shadow=#{:06X} border_fill_id={} shadow_type={} shadow_off=({}, {}) char={:?}",
                            cs.start_pos,
                            cs.char_shape_id,
                            bold,
                            spacing,
                            ratio,
                            chs.base_size,
                            chs.attr,
                            chs.text_color,
                            chs.shade_color,
                            chs.shadow_color,
                            chs.border_fill_id,
                            chs.shadow_type,
                            chs.shadow_offset_x,
                            chs.shadow_offset_y,
                            char_at.map(|c| c.to_string()).unwrap_or_default()
                        );
                    }
                }
            }
            if let Some(ps) = document
                .doc_info
                .para_shapes
                .get(para.para_shape_id as usize)
            {
                // 문단 모양 기본 정보 (항상 출력)
                println!(
                    "  [PS] ps_id={} align={:?} spacing: before={} after={} line={}/{:?}",
                    para.para_shape_id,
                    ps.alignment,
                    ps.spacing_before,
                    ps.spacing_after,
                    ps.line_spacing,
                    ps.line_spacing_type
                );
                println!(
                    "       margins: left={} right={} indent={} border_fill_id={}",
                    ps.margin_left, ps.margin_right, ps.indent, ps.border_fill_id
                );
                if ps.border_fill_id > 0 {
                    println!(
                        "       border_spacing: left={} right={} top={} bottom={}",
                        ps.border_spacing[0],
                        ps.border_spacing[1],
                        ps.border_spacing[2],
                        ps.border_spacing[3]
                    );
                }
                if ps.head_type != rhwp::model::style::HeadType::None {
                    println!("       head={:?} level={} num_id={} attr1=0x{:08X} attr2=0x{:08X} raw_extra={:?}",
                        ps.head_type, ps.para_level, ps.numbering_id, ps.attr1, ps.attr2,
                        &para.raw_header_extra);
                }
                {
                    let td_id = ps.tab_def_id;
                    if let Some(td) = document.doc_info.tab_defs.get(td_id as usize) {
                        let tabs_str: Vec<String> = td
                            .tabs
                            .iter()
                            .enumerate()
                            .map(|(i, t)| {
                                format!(
                                    "tab[{}] pos={} ({:.1}mm) type={} fill={}",
                                    i,
                                    t.position,
                                    hu_to_mm(t.position),
                                    t.tab_type,
                                    t.fill_type
                                )
                            })
                            .collect();
                        println!(
                            "       tab_def_id={} auto_left={} auto_right={} tabs=[{}]",
                            td_id,
                            td.auto_tab_left,
                            td.auto_tab_right,
                            if tabs_str.is_empty() {
                                "(없음)".to_string()
                            } else {
                                tabs_str.join(", ")
                            }
                        );
                    } else {
                        println!("       tab_def_id={} (정의 없음)", td_id);
                    }
                }
            }
            // line_segs 출력
            if !para.line_segs.is_empty() {
                for (li, ls) in para.line_segs.iter().enumerate() {
                    println!("  ls[{}]: ts={}, vpos={}, lh={}, th={}, bl={}, ls={}, cs={}, sw={}, tag=0x{:08X}",
                        li, ls.text_start, ls.vertical_pos, ls.line_height, ls.text_height,
                        ls.baseline_distance, ls.line_spacing, ls.column_start, ls.segment_width, ls.tag);
                }
            }

            for (ctrl_idx, ctrl) in para.controls.iter().enumerate() {
                let prefix = format!("  [{}] ", ctrl_idx);
                match ctrl {
                    Control::ColumnDef(cd) => {
                        let ct = match cd.column_type {
                            rhwp::model::page::ColumnType::Normal => "일반",
                            rhwp::model::page::ColumnType::Distribute => "배분",
                            rhwp::model::page::ColumnType::Parallel => "병행",
                        };
                        println!(
                            "{}단정의: {}단, 유형={}, 간격={:.1}mm({}), 같은너비={}",
                            prefix,
                            cd.column_count,
                            ct,
                            hu_to_mm_i(cd.spacing as i32),
                            cd.spacing,
                            cd.same_width
                        );
                        if !cd.widths.is_empty() {
                            // 비례값일 경우 body_width 기준으로 실제 mm 변환
                            let body_width_hu = {
                                let spd = &section.section_def.page_def;
                                let (pw, _) = if spd.landscape {
                                    (spd.height, spd.width)
                                } else {
                                    (spd.width, spd.height)
                                };
                                (pw - spd.margin_left - spd.margin_right - spd.margin_gutter) as f64
                            };
                            let total: f64 = if cd.proportional_widths {
                                cd.widths
                                    .iter()
                                    .chain(cd.gaps.iter())
                                    .map(|&v| (v as u16) as f64)
                                    .sum()
                            } else {
                                1.0
                            };
                            let cols_info: Vec<String> = cd
                                .widths
                                .iter()
                                .enumerate()
                                .map(|(i, w)| {
                                    let gap = cd.gaps.get(i).copied().unwrap_or(0);
                                    if cd.proportional_widths && total > 0.0 {
                                        let w_hu = (*w as u16) as f64 / total * body_width_hu;
                                        let g_hu = (gap as u16) as f64 / total * body_width_hu;
                                        format!(
                                            "너비={:.1}mm 간격={:.1}mm",
                                            w_hu * 25.4 / 7200.0,
                                            g_hu * 25.4 / 7200.0
                                        )
                                    } else {
                                        format!(
                                            "너비={:.1}mm 간격={:.1}mm",
                                            hu_to_mm_i(*w as i32),
                                            hu_to_mm_i(gap as i32)
                                        )
                                    }
                                })
                                .collect();
                            println!("{}  단별: [{}]", prefix, cols_info.join(", "));
                        }
                        if cd.separator_type > 0 {
                            println!(
                                "{}  구분선: type={}, width={}, color={:#010x}",
                                prefix, cd.separator_type, cd.separator_width, cd.separator_color
                            );
                        }
                    }
                    Control::SectionDef(sd) => {
                        let spd = &sd.page_def;
                        println!(
                            "{}구역정의: 용지 {:.1}×{:.1}mm, {}, flags=0x{:08X}",
                            prefix,
                            hu_to_mm(spd.width),
                            hu_to_mm(spd.height),
                            if spd.landscape { "가로" } else { "세로" },
                            sd.flags
                        );
                        if sd.hide_header || sd.hide_footer || sd.hide_master_page {
                            println!(
                                "{}  감추기: 머리말={} 꼬리말={} 바탕쪽={}",
                                prefix, sd.hide_header, sd.hide_footer, sd.hide_master_page
                            );
                        }
                    }
                    Control::Table(table) => {
                        println!("{}표: {}행×{}열, 셀={}, 쪽나눔={:?} (attr=0x{:08x}), padding=({},{},{},{}), cs={}",
                            prefix, table.row_count, table.col_count,
                            table.cells.len(), table.page_break, table.raw_table_record_attr,
                            table.padding.left, table.padding.right, table.padding.top, table.padding.bottom,
                            table.cell_spacing);
                        if !table.zones.is_empty() {
                            for (zi, z) in table.zones.iter().enumerate() {
                                println!(
                                    "{}  zone[{}] row={}..{} col={}..{} bf={}",
                                    prefix,
                                    zi,
                                    z.start_row,
                                    z.end_row,
                                    z.start_col,
                                    z.end_col,
                                    z.border_fill_id
                                );
                            }
                        }
                        {
                            let c = &table.common;
                            println!("{}  [common] treat_as_char={}, wrap={}, vert={}({}={:.1}mm), horz={}({}={:.1}mm)",
                                prefix, c.treat_as_char, wrap_str(&c.text_wrap),
                                vert_str(&c.vert_rel_to), c.vertical_offset, hu_to_mm(c.vertical_offset),
                                horz_str(&c.horz_rel_to), c.horizontal_offset, hu_to_mm(c.horizontal_offset));
                            println!(
                                "{}  [common] size={}×{}({:.1}×{:.1}mm), valign={:?}, halign={:?}",
                                prefix,
                                c.width,
                                c.height,
                                hu_to_mm(c.width),
                                hu_to_mm(c.height),
                                c.vert_align,
                                c.horz_align
                            );
                            println!("{}  [outer_margin] left={:.1}mm({}) right={:.1}mm({}) top={:.1}mm({}) bottom={:.1}mm({})",
                                prefix,
                                hu_to_mm_i(table.outer_margin_left as i32), table.outer_margin_left,
                                hu_to_mm_i(table.outer_margin_right as i32), table.outer_margin_right,
                                hu_to_mm_i(table.outer_margin_top as i32), table.outer_margin_top,
                                hu_to_mm_i(table.outer_margin_bottom as i32), table.outer_margin_bottom);
                            if table.raw_ctrl_data.len() >= 20 {
                                println!(
                                    "{}  [raw] {:02X?}",
                                    prefix,
                                    &table.raw_ctrl_data[..20.min(table.raw_ctrl_data.len())]
                                );
                            }
                        }
                        // 셀 상세 출력
                        fn dump_table_deep(
                            table: &rhwp::model::table::Table,
                            indent: &str,
                            depth: usize,
                        ) {
                            for (ci, cell) in table.cells.iter().enumerate() {
                                let text_preview: String = cell
                                    .paragraphs
                                    .iter()
                                    .map(|p| p.text.chars().take(30).collect::<String>())
                                    .collect::<Vec<_>>()
                                    .join("|");
                                println!("{}셀[{}] r={},c={} rs={},cs={} h={} w={} pad=({},{},{},{}) valign={:?} aim={} bf={} paras={} text=\"{}\"",
                                    indent, ci, cell.row, cell.col, cell.row_span, cell.col_span,
                                    cell.height, cell.width,
                                    cell.padding.left, cell.padding.right, cell.padding.top, cell.padding.bottom,
                                    cell.vertical_align,
                                    cell.apply_inner_margin,
                                    cell.border_fill_id, cell.paragraphs.len(), text_preview);
                                if let Some(ref fname) = cell.field_name {
                                    println!("{}  field=\"{}\"", indent, fname);
                                }
                                // 셀 내 LINE_SEG 상세
                                for (pi, cp) in cell.paragraphs.iter().enumerate() {
                                    if !cp.line_segs.is_empty() || !cp.controls.is_empty() {
                                        let ls_info: Vec<String> = cp
                                            .line_segs
                                            .iter()
                                            .enumerate()
                                            .map(|(li, ls)| {
                                                format!(
                                                    "ls[{}] vpos={} lh={} ls={}",
                                                    li,
                                                    ls.vertical_pos,
                                                    ls.line_height,
                                                    ls.line_spacing
                                                )
                                            })
                                            .collect();
                                        println!(
                                            "{}  p[{}] ps_id={} ctrls={} text_len={} {}",
                                            indent,
                                            pi,
                                            cp.para_shape_id,
                                            cp.controls.len(),
                                            cp.text.len(),
                                            ls_info.join(", ")
                                        );
                                    }
                                    // 셀 내부 컨트롤 상세
                                    for (ci, ctrl) in cp.controls.iter().enumerate() {
                                        match ctrl {
                                            Control::Picture(p) => {
                                                println!("{}    ctrl[{}] 그림: bin_id={}, w={} h={} ({:.1}×{:.1}mm), tac={}, wrap={:?}, vert={:?}(off={}), horz={:?}(off={}), orig={}×{}, cur={}×{}, crop=({},{},{},{})",
                                                    indent, ci, p.image_attr.bin_data_id,
                                                    p.common.width, p.common.height,
                                                    p.common.width as f64 / 7200.0 * 25.4,
                                                    p.common.height as f64 / 7200.0 * 25.4,
                                                    p.common.treat_as_char,
                                                    p.common.text_wrap, p.common.vert_rel_to, p.common.vertical_offset,
                                                    p.common.horz_rel_to, p.common.horizontal_offset,
                                                    p.shape_attr.original_width, p.shape_attr.original_height,
                                                    p.shape_attr.current_width, p.shape_attr.current_height,
                                                    p.crop.left, p.crop.top, p.crop.right, p.crop.bottom);
                                                println!("{}      [image_attr] effect={:?} brightness={} contrast={} watermark={}",
                                                    indent, p.image_attr.effect, p.image_attr.brightness, p.image_attr.contrast,
                                                    p.image_attr.watermark_preset().unwrap_or("none"));
                                            }
                                            Control::Shape(s) => {
                                                println!(
                                                    "{}    ctrl[{}] {}: tac={}, wrap={:?}",
                                                    indent,
                                                    ci,
                                                    s.shape_name(),
                                                    s.common().treat_as_char,
                                                    s.common().text_wrap
                                                );
                                            }
                                            Control::PageHide(ph) => {
                                                println!("{}    ctrl[{}] PageHide: header={} footer={} master={} border={} fill={} page_num={}",
                                                    indent, ci,
                                                    ph.hide_header, ph.hide_footer, ph.hide_master_page,
                                                    ph.hide_border, ph.hide_fill, ph.hide_page_num);
                                            }
                                            _ => {}
                                        }
                                    }
                                    // 내부 표 재귀
                                    if depth < 3 {
                                        for ctrl in &cp.controls {
                                            if let Control::Table(inner) = ctrl {
                                                println!("{}  p[{}] 내부표: {}행×{}열, 셀={}, cs={}, pad=({},{},{},{})",
                                                    indent, pi, inner.row_count, inner.col_count,
                                                    inner.cells.len(), inner.cell_spacing,
                                                    inner.padding.left, inner.padding.right, inner.padding.top, inner.padding.bottom);
                                                let next_indent = format!("{}    ", indent);
                                                dump_table_deep(inner, &next_indent, depth + 1);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        dump_table_deep(table, &format!("{}  ", prefix), 0);
                    }
                    Control::Shape(shape) => {
                        print!("{}", prefix);
                        dump_shape(shape, "  ", &dump_common, &dump_shape_attr);
                    }
                    Control::Picture(pic) => {
                        let sa = &pic.shape_attr;
                        println!("{}그림: bin_id={}, common={}×{} ({:.1}×{:.1}mm), orig={}×{} ({:.1}×{:.1}mm), cur={}×{} ({:.1}×{:.1}mm), tac={}",
                            prefix, pic.image_attr.bin_data_id, pic.common.width, pic.common.height,
                            pic.common.width as f64 / 7200.0 * 25.4, pic.common.height as f64 / 7200.0 * 25.4,
                            sa.original_width, sa.original_height,
                            sa.original_width as f64 / 7200.0 * 25.4, sa.original_height as f64 / 7200.0 * 25.4,
                            sa.current_width, sa.current_height,
                            sa.current_width as f64 / 7200.0 * 25.4, sa.current_height as f64 / 7200.0 * 25.4,
                            pic.common.treat_as_char);
                        println!(
                            "{}  [image_attr] effect={:?} brightness={} contrast={} watermark={}{}",
                            prefix,
                            pic.image_attr.effect,
                            pic.image_attr.brightness,
                            pic.image_attr.contrast,
                            pic.image_attr.watermark_preset().unwrap_or("none"),
                            pic.image_attr
                                .external_path
                                .as_ref()
                                .map(|p| format!(" external_path=\"{}\"", p))
                                .unwrap_or_default()
                        );
                        println!("{}  border_x={:?} border_y={:?} border_color=#{:06X} border_width={} ({:.2}mm) border_attr={:?}",
                            prefix, pic.border_x, pic.border_y,
                            pic.border_color, pic.border_width, pic.border_width as f64 / 7200.0 * 25.4,
                            pic.border_attr);
                        println!(
                            "{}  crop=({},{},{},{}) crop_mm=({:.2},{:.2},{:.2},{:.2})",
                            prefix,
                            pic.crop.left,
                            pic.crop.top,
                            pic.crop.right,
                            pic.crop.bottom,
                            pic.crop.left as f64 / 7200.0 * 25.4,
                            pic.crop.top as f64 / 7200.0 * 25.4,
                            pic.crop.right as f64 / 7200.0 * 25.4,
                            pic.crop.bottom as f64 / 7200.0 * 25.4
                        );
                        if let Some(ref cap) = pic.caption {
                            let cap_text: String = cap
                                .paragraphs
                                .iter()
                                .map(|p| p.text.clone())
                                .collect::<Vec<_>>()
                                .join("|");
                            println!(
                                "{}  caption: dir={:?} width={} paras={} text={:?}",
                                prefix,
                                cap.direction,
                                cap.width,
                                cap.paragraphs.len(),
                                cap_text
                            );
                        }
                        let shape_indent = format!("{}  ", prefix);
                        dump_shape_attr(sa, &shape_indent);
                        dump_common(&pic.common, "  ");
                    }
                    Control::Header(h) => {
                        let text: String = h
                            .paragraphs
                            .iter()
                            .filter(|p| !p.text.is_empty())
                            .map(|p| p.text.clone())
                            .collect::<Vec<_>>()
                            .join(" ");
                        println!(
                            "{}머리말({:?}): paras={} \"{}\"",
                            prefix,
                            h.apply_to,
                            h.paragraphs.len(),
                            text
                        );
                        for (hpi, hp) in h.paragraphs.iter().enumerate() {
                            if !hp.controls.is_empty() {
                                for (hci, hc) in hp.controls.iter().enumerate() {
                                    let cn = match hc {
                                        Control::AutoNumber(an) => {
                                            format!("자동번호({:?})", an.number_type)
                                        }
                                        Control::Shape(s) => {
                                            let c = s.common();
                                            let mut desc = format!(
                                                "Shape horz={:?}/{} halign={:?} w={} h={}",
                                                c.horz_rel_to,
                                                c.horizontal_offset,
                                                c.horz_align,
                                                c.width,
                                                c.height
                                            );
                                            if let Some(tb) =
                                                s.drawing().and_then(|d| d.text_box.as_ref())
                                            {
                                                let text: String = tb
                                                    .paragraphs
                                                    .iter()
                                                    .flat_map(|p| p.text.chars().take(20))
                                                    .collect();
                                                desc += &format!(" text={:?}", text);
                                            }
                                            desc
                                        }
                                        Control::Table(t) => {
                                            let mut desc = format!(
                                                "표 {}행×{}열 셀={}",
                                                t.row_count,
                                                t.col_count,
                                                t.cells.len()
                                            );
                                            for (si, cell) in t.cells.iter().enumerate() {
                                                let cell_text: String = cell
                                                    .paragraphs
                                                    .iter()
                                                    .flat_map(|p| p.text.chars().take(20))
                                                    .collect();
                                                desc += &format!(
                                                    "\n{}    셀[{}] text={:?}",
                                                    prefix, si, cell_text
                                                );
                                                for (cpi, cp) in cell.paragraphs.iter().enumerate()
                                                {
                                                    for (cci, cc) in cp.controls.iter().enumerate()
                                                    {
                                                        let ccn = match cc {
                                                            Control::AutoNumber(an) => format!(
                                                                "자동번호({:?})",
                                                                an.number_type
                                                            ),
                                                            Control::Shape(s) => {
                                                                let c = s.common();
                                                                let mut d = format!("Shape vert={:?}/{} valign={:?} horz={:?}/{} halign={:?} w={} h={}",
                                                c.vert_rel_to, c.vertical_offset, c.vert_align,
                                                c.horz_rel_to, c.horizontal_offset, c.horz_align, c.width, c.height);
                                                                if let Some(tb) =
                                                                    s.drawing().and_then(|dd| {
                                                                        dd.text_box.as_ref()
                                                                    })
                                                                {
                                                                    for (tpi, tp) in tb
                                                                        .paragraphs
                                                                        .iter()
                                                                        .enumerate()
                                                                    {
                                                                        let t: String = tp
                                                                            .text
                                                                            .chars()
                                                                            .take(30)
                                                                            .collect();
                                                                        d += &format!(" tb_p[{}] ps_id={} text={:?}", tpi, tp.para_shape_id, t);
                                                                    }
                                                                }
                                                                d
                                                            }
                                                            _ => format!(
                                                                "{:?}",
                                                                std::mem::discriminant(cc)
                                                            ),
                                                        };
                                                        desc += &format!(
                                                            "\n{}      p[{}]c[{}]: {}",
                                                            prefix, cpi, cci, ccn
                                                        );
                                                    }
                                                }
                                            }
                                            desc
                                        }
                                        Control::Picture(pic) => {
                                            let sa = &pic.shape_attr;
                                            format!("그림: bin_id={}, common={}×{} ({:.1}×{:.1}mm), orig={}×{} ({:.1}×{:.1}mm), cur={}×{} ({:.1}×{:.1}mm), tac={}, crop=({},{},{},{}) crop_mm=({:.2},{:.2},{:.2},{:.2})",
                                            pic.image_attr.bin_data_id, pic.common.width, pic.common.height,
                                            pic.common.width as f64 / 7200.0 * 25.4, pic.common.height as f64 / 7200.0 * 25.4,
                                            sa.original_width, sa.original_height,
                                            sa.original_width as f64 / 7200.0 * 25.4, sa.original_height as f64 / 7200.0 * 25.4,
                                            sa.current_width, sa.current_height,
                                            sa.current_width as f64 / 7200.0 * 25.4, sa.current_height as f64 / 7200.0 * 25.4,
                                            pic.common.treat_as_char,
                                            pic.crop.left, pic.crop.top, pic.crop.right, pic.crop.bottom,
                                            pic.crop.left as f64 / 7200.0 * 25.4, pic.crop.top as f64 / 7200.0 * 25.4,
                                            pic.crop.right as f64 / 7200.0 * 25.4, pic.crop.bottom as f64 / 7200.0 * 25.4)
                                        }
                                        _ => format!("{:?}", std::mem::discriminant(hc)),
                                    };
                                    let display = if cn.chars().count() > 30 {
                                        format!(
                                            "{}...(truncated)",
                                            cn.chars().take(30).collect::<String>()
                                        )
                                    } else {
                                        cn
                                    };
                                    println!("{}  hp[{}] ctrl[{}]: {}", prefix, hpi, hci, display);
                                }
                            }
                        }
                    }
                    Control::Footer(f) => {
                        let text: String = f
                            .paragraphs
                            .iter()
                            .filter(|p| !p.text.is_empty())
                            .map(|p| p.text.clone())
                            .collect::<Vec<_>>()
                            .join(" ");
                        println!(
                            "{}꼬리말({:?}): paras={} \"{}\"",
                            prefix,
                            f.apply_to,
                            f.paragraphs.len(),
                            text
                        );
                        for (fpi, fp) in f.paragraphs.iter().enumerate() {
                            if !fp.controls.is_empty() {
                                for (fci, fc) in fp.controls.iter().enumerate() {
                                    let cn = match fc {
                                        Control::Picture(pic) => {
                                            let sa = &pic.shape_attr;
                                            format!("그림: bin_id={}, common={}×{} ({:.1}×{:.1}mm), orig={}×{} ({:.1}×{:.1}mm), cur={}×{} ({:.1}×{:.1}mm), tac={}, crop=({},{},{},{}) crop_mm=({:.2},{:.2},{:.2},{:.2})",
                                            pic.image_attr.bin_data_id, pic.common.width, pic.common.height,
                                            pic.common.width as f64 / 7200.0 * 25.4, pic.common.height as f64 / 7200.0 * 25.4,
                                            sa.original_width, sa.original_height,
                                            sa.original_width as f64 / 7200.0 * 25.4, sa.original_height as f64 / 7200.0 * 25.4,
                                            sa.current_width, sa.current_height,
                                            sa.current_width as f64 / 7200.0 * 25.4, sa.current_height as f64 / 7200.0 * 25.4,
                                            pic.common.treat_as_char,
                                            pic.crop.left, pic.crop.top, pic.crop.right, pic.crop.bottom,
                                            pic.crop.left as f64 / 7200.0 * 25.4, pic.crop.top as f64 / 7200.0 * 25.4,
                                            pic.crop.right as f64 / 7200.0 * 25.4, pic.crop.bottom as f64 / 7200.0 * 25.4)
                                        }
                                        _ => format!("{:?}", std::mem::discriminant(fc)),
                                    };
                                    println!("{}  fp[{}] ctrl[{}]: {}", prefix, fpi, fci, cn);
                                }
                            }
                        }
                    }
                    Control::Footnote(fn_) => {
                        println!("{}각주: paragraphs={}", prefix, fn_.paragraphs.len());
                    }
                    Control::Endnote(en) => {
                        println!("{}미주: paragraphs={}", prefix, en.paragraphs.len());
                    }
                    Control::AutoNumber(an) => {
                        println!(
                            "{}자동번호: type={:?}, number={}",
                            prefix, an.number_type, an.number
                        );
                    }
                    Control::NewNumber(nn) => {
                        println!(
                            "{}새번호: type={:?}, number={}",
                            prefix, nn.number_type, nn.number
                        );
                    }
                    Control::PageNumberPos(pn) => {
                        println!(
                            "{}쪽번호위치: format={}, pos={}",
                            prefix, pn.format, pn.position
                        );
                    }
                    Control::Bookmark(bm) => {
                        println!("{}책갈피: \"{}\"", prefix, bm.name);
                    }
                    Control::Hyperlink(hl) => {
                        println!("{}하이퍼링크: \"{}\"", prefix, hl.url);
                    }
                    Control::Ruby(r) => {
                        println!("{}덧말: \"{}\"", prefix, r.ruby_text);
                    }
                    Control::PageHide(ph) => {
                        println!("{}감추기: header={}, footer={}, master={}, border={}, fill={}, page_num={}",
                            prefix, ph.hide_header, ph.hide_footer, ph.hide_master_page, ph.hide_border, ph.hide_fill, ph.hide_page_num);
                    }
                    Control::HiddenComment(_) => {
                        println!("{}숨은설명", prefix);
                    }
                    Control::Field(f) => {
                        let name = f.field_name().unwrap_or("(이름없음)");
                        println!(
                            "{}필드: {:?} name=\"{}\" cmd=\"{}\"",
                            prefix, f.field_type, name, f.command
                        );
                    }
                    Control::CharOverlap(co) => {
                        println!("{}글자겹침: {:?}", prefix, co.chars);
                    }
                    Control::Equation(eq) => {
                        println!(
                            "{}수식: script=\"{}\" font_size={} font=\"{}\" size={}x{} tac={}",
                            prefix,
                            eq.script,
                            eq.font_size,
                            eq.font_name,
                            eq.common.width,
                            eq.common.height,
                            eq.common.treat_as_char
                        );
                    }
                    Control::Form(f) => {
                        println!(
                            "{}양식개체: {:?} name=\"{}\" caption=\"{}\" {}x{}",
                            prefix, f.form_type, f.name, f.caption, f.width, f.height
                        );
                    }
                    Control::Unknown(u) => {
                        println!("{}알수없음: ctrl_id={:#010x}", prefix, u.ctrl_id);
                    }
                }
            }
        }
    }

    println!(
        "\n=== 완료: {} 구역, {} 문단 ===",
        document.sections.len(),
        document
            .sections
            .iter()
            .map(|s| s.paragraphs.len())
            .sum::<usize>()
    );
}

fn diag_document(args: &[String]) {
    if args.is_empty() {
        eprintln!("오류: HWP 파일 경로를 지정해주세요.");
        eprintln!("사용법: rhwp diag <파일.hwp>");
        return;
    }

    let file_path = &args[0];
    let data = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return;
        }
    };

    let doc = match rhwp::wasm_api::HwpDocument::from_bytes(&data) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: HWP 파싱 실패 - {}", e);
            return;
        }
    };

    let document = doc.document();
    use rhwp::model::style::HeadType;

    // === DocInfo 요약 ===
    println!("=== DocInfo 요약 ===");
    println!("  Numbering: {}개", document.doc_info.numberings.len());
    for (i, num) in document.doc_info.numberings.iter().enumerate() {
        let formats: Vec<String> = num
            .level_formats
            .iter()
            .enumerate()
            .filter(|(_, f)| !f.is_empty())
            .map(|(lv, f)| format!("L{}=\"{}\"", lv + 1, f))
            .collect();
        println!(
            "    [{}] start={}, formats: {}",
            i,
            num.start_number,
            formats.join(", ")
        );
    }

    println!("  Bullet: {}개", document.doc_info.bullets.len());
    for (i, bullet) in document.doc_info.bullets.iter().enumerate() {
        println!(
            "    [{}] char='{}' (U+{:04X})",
            i, bullet.bullet_char, bullet.bullet_char as u32
        );
    }

    // === ParaShape head_type 분포 ===
    println!("\n=== ParaShape head_type 분포 ===");
    let mut count_none = 0u32;
    let mut count_outline = 0u32;
    let mut count_number = 0u32;
    let mut count_bullet = 0u32;
    for ps in &document.doc_info.para_shapes {
        match ps.head_type {
            HeadType::None => count_none += 1,
            HeadType::Outline => count_outline += 1,
            HeadType::Number => count_number += 1,
            HeadType::Bullet => count_bullet += 1,
        }
    }
    println!(
        "  None: {}개, Outline: {}개, Number: {}개, Bullet: {}개",
        count_none, count_outline, count_number, count_bullet
    );

    // === SectionDef 개요번호 ===
    println!("\n=== SectionDef 개요번호 ===");
    for (sec_idx, section) in document.sections.iter().enumerate() {
        // SectionDef의 raw_ctrl_extra에서 바이트 14-15 추출 (outline_numbering_id)
        // 현재 outline_numbering_id 필드가 없으므로 파싱 전 상태에서는 raw_ctrl_extra 참조
        // 6단계에서 필드 추가 후 직접 참조로 변경 예정
        let sd = &section.section_def;
        let num_ref = if sd.outline_numbering_id > 0 {
            format!(" → Numbering[{}]", sd.outline_numbering_id - 1)
        } else {
            " (없음)".to_string()
        };
        println!(
            "  구역{}: outline_numbering_id={}{}, flags={:#010x}",
            sec_idx, sd.outline_numbering_id, num_ref, sd.flags
        );
    }

    // === 비None head_type 문단 ===
    println!("\n=== 비None head_type 문단 ===");
    for (sec_idx, section) in document.sections.iter().enumerate() {
        for (para_idx, para) in section.paragraphs.iter().enumerate() {
            if let Some(ps) = document
                .doc_info
                .para_shapes
                .get(para.para_shape_id as usize)
            {
                if ps.head_type != HeadType::None {
                    let text_preview: String = para.text.chars().take(40).collect();
                    let text_display = if para.text.chars().count() > 40 {
                        format!("\"{}...\"", text_preview)
                    } else {
                        format!("\"{}\"", text_preview)
                    };
                    println!(
                        "  구역{}:문단{} head={:?} level={} num_id={} text={}",
                        sec_idx,
                        para_idx,
                        ps.head_type,
                        ps.para_level,
                        ps.numbering_id,
                        text_display
                    );
                }
            }
        }
    }
}

fn convert_hwp(args: &[String]) {
    if args.len() < 2 {
        eprintln!("오류: 입력 파일과 출력 파일 경로를 지정해주세요.");
        eprintln!("사용법: rhwp convert <입력.hwp|입력.hwpx> <출력.hwp>");
        return;
    }

    let input_path = &args[0];
    let output_path = &args[1];

    // 입력 파일 읽기
    let data = match fs::read(input_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", input_path, e);
            return;
        }
    };

    // 문서 로드
    let mut doc = match rhwp::wasm_api::HwpDocument::from_bytes(&data) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: HWP 파싱 실패 - {}", e);
            return;
        }
    };

    let was_distribution = doc.document().header.distribution;
    if !was_distribution {
        println!("{}: 이미 편집 가능한 문서입니다.", input_path);
    }

    // 변환
    match doc.convert_to_editable_native() {
        Ok(_) => {
            if was_distribution {
                println!("배포용 → 편집 가능 변환 완료");
            }
        }
        Err(e) => {
            eprintln!("오류: 변환 실패 - {}", e);
            return;
        }
    }

    // 직렬화
    match doc.export_hwp_with_adapter() {
        Ok(bytes) => match fs::write(output_path, &bytes) {
            Ok(_) => {
                println!("저장 완료: {} ({}KB)", output_path, bytes.len() / 1024);
            }
            Err(e) => {
                eprintln!("오류: 파일 저장 실패 - {}: {}", output_path, e);
            }
        },
        Err(e) => {
            eprintln!("오류: 직렬화 실패 - {}", e);
        }
    }
}

/// `rhwp build-from-ingest <ingest.json> [--media-dir <dir>] -o <out.hwpx>`
///
/// Claude Code Skill (`rhwp-exam-ingest`)이 생성한 JSON 중간 표현을 HWPX로 변환한다.
/// Task #660 (Neumann 본 작업 1단계).
fn build_from_ingest(args: &[String]) {
    if args.is_empty() {
        eprintln!("사용법: rhwp build-from-ingest <ingest.json> [--media-dir <dir>] -o <out.hwpx>");
        return;
    }

    let mut input_path: Option<&str> = None;
    let mut output_path: Option<&str> = None;
    let mut media_dir: Option<&str> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-o" | "--output" => {
                if i + 1 >= args.len() {
                    eprintln!("오류: -o 옵션에 값이 필요합니다");
                    return;
                }
                output_path = Some(&args[i + 1]);
                i += 2;
            }
            "--media-dir" => {
                if i + 1 >= args.len() {
                    eprintln!("오류: --media-dir 옵션에 값이 필요합니다");
                    return;
                }
                media_dir = Some(&args[i + 1]);
                i += 2;
            }
            other => {
                if input_path.is_none() {
                    input_path = Some(other);
                } else {
                    eprintln!("경고: 알 수 없는 인자 '{}' 무시", other);
                }
                i += 1;
            }
        }
    }

    let input = match input_path {
        Some(p) => p,
        None => {
            eprintln!("오류: 입력 ingest JSON 경로가 누락되었습니다");
            return;
        }
    };
    let output = match output_path {
        Some(p) => p,
        None => {
            eprintln!("오류: -o <출력 경로> 가 누락되었습니다");
            return;
        }
    };

    let bytes = match fs::read(input) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("오류: 입력 파일 읽기 실패 - {}: {}", input, e);
            return;
        }
    };

    let ingest = match rhwp::parser::ingest::parse_ingest_bytes(&bytes) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: ingest JSON 파싱 실패 - {}", e);
            return;
        }
    };

    if let Some(md) = media_dir {
        let p = Path::new(md);
        if !p.exists() {
            eprintln!(
                "경고: 미디어 디렉토리가 존재하지 않습니다 ({}). 본 단계는 이미지 placeholder로 처리됩니다.",
                md
            );
        }
    }

    let doc = rhwp::document_core::builders::exam_paper::build_exam_paper(&ingest);

    let hwpx_bytes = match rhwp::serializer::serialize_hwpx(&doc) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("오류: HWPX 직렬화 실패 - {}", e);
            return;
        }
    };

    match fs::write(output, &hwpx_bytes) {
        Ok(_) => println!(
            "저장 완료: {} ({}바이트, 문제 {}개, 문단 {}개)",
            output,
            hwpx_bytes.len(),
            ingest.questions.len(),
            doc.sections
                .iter()
                .map(|s| s.paragraphs.len())
                .sum::<usize>()
        ),
        Err(e) => eprintln!("오류: 파일 저장 실패 - {}: {}", output, e),
    }
}

fn dump_raw_records(args: &[String]) {
    if args.is_empty() {
        eprintln!("사용법: rhwp dump-records <파일.hwp>");
        return;
    }
    let data = match fs::read(&args[0]) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: {}", e);
            return;
        }
    };
    use rhwp::parser::cfb_reader::CfbReader;
    use rhwp::parser::record::Record;
    let mut cfb = match CfbReader::open(&data) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("오류: {:?}", e);
            return;
        }
    };
    // FileHeader에서 압축 여부 확인
    let header = cfb.read_stream_raw("FileHeader").unwrap_or_default();
    let compressed = header.len() >= 40 && (header[36] & 0x01) != 0;
    let section = match cfb.read_body_text_section(0, compressed, false) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("오류: {:?}", e);
            return;
        }
    };
    let records = match Record::read_all(&section) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("오류: {:?}", e);
            return;
        }
    };
    let tag_name = |id: u16| -> &str {
        match id {
            66 => "PARA_HEADER",
            67 => "PARA_TEXT",
            68 => "PARA_CHAR_SHAPE",
            69 => "PARA_LINE_SEG",
            70 => "PARA_RANGE_TAG",
            71 => "CTRL_HEADER",
            72 => "LIST_HEADER",
            73 => "PAGE_DEF",
            74 => "FOOTNOTE_SHAPE",
            75 => "PAGE_BORDER_FILL",
            76 => "SHAPE_COMPONENT",
            77 => "TABLE",
            78 => "SC_LINE",
            79 => "SC_RECT",
            80 => "SC_ELLIPSE",
            81 => "SC_ARC",
            82 => "SC_POLYGON",
            83 => "SC_CURVE",
            85 => "SC_PICTURE",
            86 => "SC_CONTAINER",
            89 => "CTRL_DATA",
            _ => "?",
        }
    };
    for (i, rec) in records.iter().enumerate() {
        let indent = "  ".repeat(rec.level as usize);
        println!(
            "[{:3}] {}tag={:<3} {:16} lv={} sz={}",
            i,
            indent,
            rec.tag_id,
            tag_name(rec.tag_id),
            rec.level,
            rec.data.len()
        );
        // shape 관련 레코드만 hex 덤프
        if matches!(rec.tag_id, 71 | 72 | 76 | 79 | 85 | 89) {
            // 16바이트씩 나눠서 hex 출력
            for chunk in rec.data.chunks(16) {
                let hex: String = chunk
                    .iter()
                    .map(|b| format!("{:02x}", b))
                    .collect::<Vec<_>>()
                    .join(" ");
                println!("       {}  {}", indent, hex);
            }
        }
    }
}

fn test_shape_roundtrip(args: &[String]) {
    let input = if args.is_empty() {
        "saved/g555-s.hwp"
    } else {
        &args[0]
    };
    let output = if args.len() > 1 {
        &args[1]
    } else {
        "/tmp/test-shape-out.hwp"
    };

    let data = match fs::read(input) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("입력 파일 읽기 오류: {}", e);
            return;
        }
    };

    let mut doc = match rhwp::wasm_api::HwpDocument::from_bytes(&data) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("HWP 파싱 오류: {:?}", e);
            return;
        }
    };

    let _ = doc.convert_to_editable_native();

    // 글상자 생성 (9000 x 6750 HWPUNIT)
    let result = doc.create_shape_control_native(
        0,
        0,
        0,
        9000,
        6750,
        0,
        0,
        false,
        "InFrontOfText",
        "rectangle",
        false,
        false,
        &[],
    );
    match &result {
        Ok(r) => eprintln!("글상자 생성 성공: {}", r),
        Err(e) => {
            eprintln!("글상자 생성 실패: {:?}", e);
            return;
        }
    }

    match doc.export_hwp_native() {
        Ok(bytes) => {
            if let Err(e) = fs::write(output, &bytes) {
                eprintln!("파일 저장 오류: {}", e);
            } else {
                eprintln!("저장 완료: {} ({}KB)", output, bytes.len() / 1024);
            }
        }
        Err(e) => eprintln!("직렬화 오류: {:?}", e),
    }
}

/// 캡션 방향별 테스트: 4개 이미지에 각각 Bottom/Top/Left/Right 캡션을 설정하고 SVG 출력
fn test_caption(args: &[String]) {
    if args.is_empty() {
        eprintln!("사용법: rhwp test-caption <파일.hwp>");
        return;
    }

    let data = match fs::read(&args[0]) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("파일 읽기 오류: {}", e);
            return;
        }
    };

    let mut doc = match rhwp::wasm_api::HwpDocument::from_bytes(&data) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("파싱 오류: {}", e);
            return;
        }
    };

    // 문단 0: 컨트롤 2,3 / 문단 1: 컨트롤 0,1
    let pic_refs: [(usize, usize); 4] = [(0, 2), (0, 3), (1, 0), (1, 1)];

    // 4개 이미지에 각각 다른 캡션 방향 설정
    let directions = [
        ("Bottom", "Top"),
        ("Top", "Top"),
        ("Left", "Center"),
        ("Right", "Center"),
    ];

    for (i, ((para, ci), (dir, va))) in pic_refs.iter().zip(directions.iter()).enumerate() {
        let json = format!(
            r#"{{"hasCaption":true,"captionDirection":"{}","captionVertAlign":"{}","captionWidth":8504,"captionSpacing":850}}"#,
            dir, va
        );
        println!("[{}] para={}, ci={}, dir={}, va={}", i, para, ci, dir, va);
        match doc.set_picture_properties_native(0, *para, *ci, &json) {
            Ok(r) => println!("  결과: {}", r),
            Err(e) => println!("  오류: {:?}", e),
        }
    }

    // 캡션 상태 확인
    for (i, (para, ci)) in pic_refs.iter().enumerate() {
        let section = &doc.document().sections[0];
        let p = &section.paragraphs[*para];
        if let rhwp::model::control::Control::Picture(pic) = &p.controls[*ci] {
            println!(
                "[{}] caption={:?}",
                i,
                pic.caption.as_ref().map(|c| {
                    format!(
                        "dir={:?}, paras={}, text={:?}",
                        c.direction,
                        c.paragraphs.len(),
                        c.paragraphs.first().map(|p| &p.text)
                    )
                })
            );
        }
    }

    // SVG 출력
    let output_dir = "output/caption-test";
    let _ = fs::create_dir_all(output_dir);
    let page_count = doc.page_count();
    println!("페이지 수: {}", page_count);
    for p in 0..page_count {
        let svg = doc.render_page_svg(p).expect("SVG 렌더링 오류");
        let path = format!("{}/caption-test-p{}.svg", output_dir, p);
        fs::write(&path, &svg).unwrap();
        println!("  → {}", path);
    }
    println!("완료");
}

fn gen_table(args: &[String]) {
    let rows: u16 = args.first().and_then(|s| s.parse().ok()).unwrap_or(1000);
    let cols: u16 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(6);
    let output = args
        .get(2)
        .map(|s| s.as_str())
        .unwrap_or("output/gen_table.hwp");

    println!("{}행 × {}열 표 생성 중...", rows, cols);

    let mut core = rhwp::document_core::DocumentCore::new_empty();
    core.create_blank_document_native()
        .expect("빈 문서 생성 실패");

    // 표 생성
    let result = core
        .create_table_native(0, 0, 0, rows, cols)
        .expect("표 생성 실패");
    println!("  표 생성: {}", result);

    // 결과에서 paraIdx 파싱
    let table_para_idx: usize = result
        .split("\"paraIdx\":")
        .nth(1)
        .and_then(|s| s.split(&[',', '}'][..]).next())
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(1);
    println!("  표 문단 인덱스: {}", table_para_idx);

    // 배치 모드로 셀 내용 채우기
    core.begin_batch_native().expect("배치 시작 실패");

    let headers = ["번호", "이름", "부서", "직급", "연락처", "비고"];
    // 헤더 행
    for (ci, header) in headers.iter().enumerate().take(cols as usize) {
        let _ = core.insert_text_in_cell_native(0, table_para_idx, 0, ci, 0, 0, header);
    }

    // 데이터 행
    let departments = ["개발팀", "기획팀", "디자인팀", "영업팀", "인사팀", "재무팀"];
    let positions = ["사원", "대리", "과장", "차장", "부장"];
    for row in 1..rows as usize {
        for col in 0..cols as usize {
            let cell_idx = row * cols as usize + col;
            let text = match col {
                0 => format!("{}", row),
                1 => format!("홍길동{}", row),
                2 => departments[row % departments.len()].to_string(),
                3 => positions[row % positions.len()].to_string(),
                4 => format!(
                    "010-{:04}-{:04}",
                    1000 + row % 9000,
                    1000 + (row * 7) % 9000
                ),
                5 => {
                    if row % 3 == 0 {
                        "특이사항 없음".to_string()
                    } else {
                        String::new()
                    }
                }
                _ => format!("R{}C{}", row, col),
            };
            if !text.is_empty() {
                let _ =
                    core.insert_text_in_cell_native(0, table_para_idx, 0, cell_idx, 0, 0, &text);
            }
        }
        if row % 100 == 0 {
            println!("  {} / {} 행 완료", row, rows);
        }
    }

    core.end_batch_native().expect("배치 종료 실패");
    println!("  셀 내용 입력 완료");

    // 저장
    let bytes = core.export_hwp_native().expect("HWP 내보내기 실패");
    let out_path = Path::new(output);
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent).ok();
    }
    fs::write(out_path, bytes).expect("파일 저장 실패");
    println!("저장 완료: {} ({}행 × {}열)", output, rows, cols);
}

/// PUA (Private Use Area) 문자 셋트를 입력한 HWP 테스트 문서 생성.
///
/// Task #509 (PUA 회귀 정정) 의 한컴 정답지 확보용. 본 라이브러리가 발견한
/// 14 샘플 광범위 PUA 코드포인트 18 종을 한 문서에 입력 → 한컴 편집기로 PDF
/// 출력 + rhwp SVG 출력 시각 비교.
///
/// 사용:
///   rhwp gen-pua [output_path]
///   기본 출력: output/pua-test.hwp
fn gen_pua_test(args: &[String]) {
    let output = args
        .first()
        .map(|s| s.as_str())
        .unwrap_or("output/pua-test.hwp");

    println!("PUA 문자 셋트 입력 HWP 문서 생성 중...");

    let mut core = rhwp::document_core::DocumentCore::new_empty();
    core.create_blank_document_native()
        .expect("빈 문서 생성 실패");

    // PUA 코드포인트 셋트 (Task #509 Stage 1 의 14 샘플 광범위 통계 정합)
    // (codepoint, 영역 분류, 사용 샘플, 본 라이브러리 현재 매핑)
    let pua_set: &[(u32, &str, &str, &str)] = &[
        // ── Basic PUA (0xF020~0xF0FF) — 매핑 표 적용 영역 ──
        (0x0F076, "Basic", "mel-001", "❖ U+2756"),
        (0x0F09F, "Basic", "biz_plan", "• U+2022"),
        (0x0F0A0, "Basic", "synam-001", "▪ U+25AA"),
        (0x0F0A7, "Basic", "kps-ai", "▪ U+25AA"),
        (0x0F0E8, "Basic", "kps-ai", "(미정의)"),
        (0x0F0F2, "Basic", "KTX", "⇩ U+21E9 (의도 정정 후보)"),
        (0x0F0FE, "Basic", "k-water-rfp", "☑ U+2611"),
        // ── Basic PUA — 매핑 표 외 영역 ──
        (0x0F53A, "Basic-out", "hwpspec", "(매핑 표 외)"),
        // ── Supplementary PUA-A (0xF0000~0xFFFFD) — 매핑 표 미지원 영역 ──
        (0xF02B1, "Suppl-A", "mel-001", "(매핑 표 외)"),
        (0xF02B2, "Suppl-A", "mel-001", "(매핑 표 외)"),
        (0xF02B3, "Suppl-A", "mel-001", "(매핑 표 외)"),
        (0xF02B4, "Suppl-A", "mel-001", "(매핑 표 외)"),
        (0xF02B5, "Suppl-A", "mel-001", "(매핑 표 외)"),
        (0xF02B6, "Suppl-A", "mel-001", "(매핑 표 외)"),
        (0xF02B7, "Suppl-A", "mel-001", "(매핑 표 외)"),
        (0xF02B8, "Suppl-A", "mel-001", "(매핑 표 외)"),
        (0xF02B9, "Suppl-A", "mel-001", "(매핑 표 외)"),
        (0xF02EF, "Suppl-A", "KTX (회귀)", "(매핑 표 외) ★"),
    ];

    println!("  PUA 코드포인트 {} 종 입력", pua_set.len());

    core.begin_batch_native().expect("배치 시작 실패");

    // 첫 paragraph (0번) 에 제목 입력
    let title = "[PUA 회귀 검증 — Task #509]";
    core.insert_text_native(0, 0, 0, title)
        .expect("제목 입력 실패");

    // 각 PUA 글자별로 paragraph 추가:
    // "U+0F0F2 (Basic, KTX): {char}    ← 한컴 정답지 / rhwp 비교"
    // 빈 paragraph 추가 + 텍스트 입력 패턴
    for (i, &(cp, area, sample, mapping)) in pua_set.iter().enumerate() {
        let pi = i + 1; // 0번은 제목, 1번부터 PUA paragraphs

        // 새 paragraph 추가 (pi 위치에 새 문단 삽입)
        core.insert_paragraph_native(0, pi)
            .unwrap_or_else(|e| panic!("paragraph 추가 실패 (pi={}): {:?}", pi, e));

        // PUA 글자 char 변환 (i32 unsafe 회피)
        let pua_char =
            char::from_u32(cp).unwrap_or_else(|| panic!("invalid codepoint U+{:05X}", cp));

        // 텍스트: "U+0F0F2 (Basic, KTX, ⇩ U+21E9 매핑): " + PUA + "  ← 한컴 PDF 글리프 정답지"
        let text = format!(
            "U+{:05X} ({}, {}, {}): {}  ← 한컴 PDF 정답지",
            cp, area, sample, mapping, pua_char
        );

        core.insert_text_native(0, pi, 0, &text)
            .unwrap_or_else(|e| panic!("텍스트 입력 실패 (pi={}): {:?}", pi, e));
    }

    core.end_batch_native().expect("배치 종료 실패");

    // 저장
    let bytes = core.export_hwp_native().expect("HWP 내보내기 실패");
    let out_path = Path::new(output);
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent).ok();
    }
    fs::write(out_path, bytes).expect("파일 저장 실패");
    println!("저장 완료: {} ({} 종 PUA)", output, pua_set.len());
    println!();
    println!("다음 단계:");
    println!("  1. 한컴 2022 편집기에서 본 파일 열기 → PDF 출력 (정답지)");
    println!("  2. rhwp export-svg {} → SVG 출력 비교", output);
    println!("  3. 시각 비교로 매핑 정합 확정");
}

fn test_field_roundtrip(args: &[String]) {
    let input = args
        .first()
        .map(|s| s.as_str())
        .unwrap_or("hwp_webctl/bsbc01_10_000.hwp");
    let output = args
        .get(1)
        .map(|s| s.as_str())
        .unwrap_or("output/field_test.hwp");

    let data = std::fs::read(input).expect("파일 읽기 실패");
    let mut core = rhwp::document_core::DocumentCore::from_bytes(&data).expect("문서 파싱 실패");

    // 1. 필드 목록 출력
    let fields = core.collect_all_fields();
    println!("=== 필드 목록 ({}개) ===", fields.len());
    for fi in &fields {
        let name = fi.field.field_name().unwrap_or("(이름없음)");
        println!("  {} = \"{}\"", name, fi.value);
    }

    // 2. 필드에 값 설정
    let test_data = [
        ("mbizNm", "청소년 자립지원사업"),
        ("newCtnuTxt", "계속"),
        ("chargerNm", "홍길동"),
        ("telno", "02-1234-5678"),
        ("sFisYear", "2026"),
        // 셀 필드
        ("bizPurps", "청소년 자립 역량 강화"),
        ("bizPrdTxt", "2026.01 ~ 2026.12"),
        ("insttNm", "시청 복지과"),
    ];

    println!("\n=== 필드 값 설정 ===");
    for (name, value) in &test_data {
        match core.set_field_value_by_name(name, value) {
            Ok(r) => println!("  ✓ {} = \"{}\" → {}", name, value, r),
            Err(e) => println!("  ✗ {} = \"{}\" → {}", name, value, e),
        }
    }

    // 3. 설정 후 확인
    println!("\n=== 설정 후 확인 ===");
    let fields2 = core.collect_all_fields();
    for fi in &fields2 {
        let name = fi.field.field_name().unwrap_or("(이름없음)");
        println!("  {} = \"{}\"", name, fi.value);
    }

    // 3.5 pi=0 문단 텍스트 직접 확인
    let para0 = &core.document().sections[0].paragraphs[0];

    // 4. 직렬화 → 저장
    let saved = core.export_hwp_native().expect("직렬화 실패");
    std::fs::write(output, &saved).expect("저장 실패");
    println!("\n저장: {} ({}바이트)", output, saved.len());

    // 5. 재로딩 → 필드 확인
    let mut core2 = rhwp::document_core::DocumentCore::from_bytes(&saved).expect("재로딩 실패");
    let fields3 = core2.collect_all_fields();
    println!("\n=== 재로딩 후 확인 ===");
    for fi in &fields3 {
        let name = fi.field.field_name().unwrap_or("(이름없음)");
        println!("  {} = \"{}\"", name, fi.value);
    }
}

fn control_tag(c: &rhwp::model::control::Control) -> &'static str {
    use rhwp::model::control::Control;
    match c {
        Control::SectionDef(_) => "secd",
        Control::ColumnDef(_) => "cold",
        Control::Table(_) => "tbl",
        Control::Shape(_) => "shape",
        Control::Picture(_) => "pic",
        Control::Header(_) => "head",
        Control::Footer(_) => "foot",
        Control::Footnote(_) => "fn",
        Control::Endnote(_) => "en",
        Control::AutoNumber(_) => "atno",
        Control::NewNumber(_) => "nwno",
        Control::PageNumberPos(_) => "pgnp",
        Control::Bookmark(_) => "bokm",
        Control::Hyperlink(_) => "hlk",
        Control::Ruby(_) => "ruby",
        Control::CharOverlap(_) => "tcps",
        Control::PageHide(_) => "pghd",
        Control::HiddenComment(_) => "tcmt",
        Control::Equation(_) => "eqed",
        Control::Field(_) => "field",
        Control::Form(_) => "form",
        Control::Unknown(_) => "unknown",
    }
}

fn diff_table(
    diffs: &mut Vec<String>,
    ci: usize,
    a: &rhwp::model::table::Table,
    b: &rhwp::model::table::Table,
) {
    if a.row_count != b.row_count {
        diffs.push(format!(
            "ctrl[{}] tbl rows: A={} vs B={}",
            ci, a.row_count, b.row_count
        ));
    }
    if a.col_count != b.col_count {
        diffs.push(format!(
            "ctrl[{}] tbl cols: A={} vs B={}",
            ci, a.col_count, b.col_count
        ));
    }
    if a.page_break != b.page_break {
        diffs.push(format!(
            "ctrl[{}] tbl page_break: A={:?} vs B={:?}",
            ci, a.page_break, b.page_break
        ));
    }
    if a.repeat_header != b.repeat_header {
        diffs.push(format!(
            "ctrl[{}] tbl repeat_header: A={} vs B={}",
            ci, a.repeat_header, b.repeat_header
        ));
    }
    if a.cell_spacing != b.cell_spacing {
        diffs.push(format!(
            "ctrl[{}] tbl cell_spacing: A={} vs B={}",
            ci, a.cell_spacing, b.cell_spacing
        ));
    }
    if a.border_fill_id != b.border_fill_id {
        diffs.push(format!(
            "ctrl[{}] tbl border_fill_id: A={} vs B={}",
            ci, a.border_fill_id, b.border_fill_id
        ));
    }
    if a.outer_margin_left != b.outer_margin_left
        || a.outer_margin_right != b.outer_margin_right
        || a.outer_margin_top != b.outer_margin_top
        || a.outer_margin_bottom != b.outer_margin_bottom
    {
        diffs.push(format!(
            "ctrl[{}] tbl outer_margin: A=({},{},{},{}) vs B=({},{},{},{})",
            ci,
            a.outer_margin_left,
            a.outer_margin_top,
            a.outer_margin_right,
            a.outer_margin_bottom,
            b.outer_margin_left,
            b.outer_margin_top,
            b.outer_margin_right,
            b.outer_margin_bottom,
        ));
    }
    diff_common_obj(diffs, ci, "tbl", &a.common, &b.common);
}

fn diff_common_obj(
    diffs: &mut Vec<String>,
    ci: usize,
    tag: &str,
    a: &rhwp::model::shape::CommonObjAttr,
    b: &rhwp::model::shape::CommonObjAttr,
) {
    if a.treat_as_char != b.treat_as_char {
        diffs.push(format!(
            "ctrl[{}] {} tac: A={} vs B={}",
            ci, tag, a.treat_as_char, b.treat_as_char
        ));
    }
    if a.text_wrap != b.text_wrap {
        diffs.push(format!(
            "ctrl[{}] {} wrap: A={:?} vs B={:?}",
            ci, tag, a.text_wrap, b.text_wrap
        ));
    }
    if a.width != b.width || a.height != b.height {
        diffs.push(format!(
            "ctrl[{}] {} size: A={}x{} vs B={}x{}",
            ci, tag, a.width, a.height, b.width, b.height
        ));
    }
    if a.vertical_offset != b.vertical_offset {
        diffs.push(format!(
            "ctrl[{}] {} v_offset: A={} vs B={}",
            ci, tag, a.vertical_offset, b.vertical_offset
        ));
    }
    if a.horizontal_offset != b.horizontal_offset {
        diffs.push(format!(
            "ctrl[{}] {} h_offset: A={} vs B={}",
            ci, tag, a.horizontal_offset, b.horizontal_offset
        ));
    }
    if a.vert_rel_to != b.vert_rel_to {
        diffs.push(format!(
            "ctrl[{}] {} vert_rel: A={:?} vs B={:?}",
            ci, tag, a.vert_rel_to, b.vert_rel_to
        ));
    }
    if a.horz_rel_to != b.horz_rel_to {
        diffs.push(format!(
            "ctrl[{}] {} horz_rel: A={:?} vs B={:?}",
            ci, tag, a.horz_rel_to, b.horz_rel_to
        ));
    }
}

/// `tab_extended`(`[u16; 7]`) 두 인라인 탭 레코드가 **의미 있는** 필드에서 다른지 판정.
///
/// HWPX 파서(`parse_tab_extension`)는 인라인 탭을 `ext[0]`=width,
/// `ext[2]`=`type<<8 | leader`(leader 는 low byte), `ext[6]`=0x0009 마커로만 채우고
/// `ext[1]`·`ext[3]`·`ext[4]`·`ext[5]`는 0 으로 둔다. HWPX 직렬화(`render_hp_t_content`)도
/// width/leader/type 를 오직 `ext[0]`·`ext[2]`에서만 읽는다. 반면 HWP5 인라인 탭(8 WCHAR
/// 블록)은 `ext[1]`을 leader/fill 슬롯으로, `ext[3]`·`ext[4]`·`ext[5]`를 WCHAR 4~6 원본
/// 바이트(보통 0x20)로 채운다 — 이들은 HWPX `<hp:tab>`에 대응 속성이 없어 HWPX 쪽이 항상
/// 0 이라, HWPX↔HWP5 parity 비교에서 거의 모든 탭에 거짓 차이(0 vs leader, 0 vs 32)를 만들어
/// 실제 차이(width/type/leader)를 가린다. 따라서 두 포맷이 공통으로 쓰는 필드
/// [0]=width, [2]=type/leader 팩, [6]=마커만 비교하고 [1],[3],[4],[5]는 제외한다.
/// (HWP5 직렬화는 [1],[3..6]을 그대로 보존하므로 self-roundtrip 충실도에는 영향 없음 —
/// 도구 비교에서만 제외.)
fn tab_ext_semantic_differs(a: &[u16; 7], b: &[u16; 7]) -> bool {
    // 두 포맷 공통 필드만: [0]=width, [2]=type<<8|leader, [6]=0x0009 마커.
    // [1](HWP5 leader/fill 슬롯, HWPX=0)·[3]·[4]·[5](HWP5 예약 바이트, HWPX=0)는 제외.
    const SEMANTIC: [usize; 3] = [0, 2, 6];
    SEMANTIC.iter().any(|&k| a[k] != b[k])
}

fn ir_diff(args: &[String]) {
    if args.len() < 2 {
        eprintln!("사용법: rhwp ir-diff <파일A> <파일B> [-s <구역>] [-p <문단>] [--summary] [--max-lines <N>]");
        return;
    }

    let file_a = &args[0];
    let file_b = &args[1];
    let mut section_filter: Option<usize> = None;
    let mut para_filter: Option<usize> = None;
    // [Task #653 보강] 출력 가드 옵션
    let mut summary_mode = false;
    let mut max_lines: Option<usize> = None;

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "-s" | "--section" if i + 1 < args.len() => {
                section_filter = args[i + 1].parse().ok();
                i += 2;
            }
            "-p" | "--para" if i + 1 < args.len() => {
                para_filter = args[i + 1].parse().ok();
                i += 2;
            }
            "--summary" => {
                summary_mode = true;
                i += 1;
            }
            "--max-lines" if i + 1 < args.len() => {
                max_lines = args[i + 1].parse().ok();
                i += 2;
            }
            _ => {
                i += 1;
            }
        }
    }

    let data_a = match fs::read(file_a) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: {} 읽기 실패: {}", file_a, e);
            return;
        }
    };
    let data_b = match fs::read(file_b) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: {} 읽기 실패: {}", file_b, e);
            return;
        }
    };

    let doc_a = match rhwp::parser::parse_document(&data_a) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: {} 파싱 실패: {:?}", file_a, e);
            return;
        }
    };
    let doc_b = match rhwp::parser::parse_document(&data_b) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: {} 파싱 실패: {:?}", file_b, e);
            return;
        }
    };

    let name_a = Path::new(file_a)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy();
    let name_b = Path::new(file_b)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy();
    if !summary_mode {
        println!("=== IR 비교: {} vs {} ===", name_a, name_b);
    }

    // [Task #653 보강] 출력 가드 상태
    let mut printed_lines: usize = 0;
    let mut truncated = false;
    let mut summary_buckets: std::collections::BTreeMap<String, u32> =
        std::collections::BTreeMap::new();

    // emit_header: paragraph/섹션 헤더. summary 모드에서는 출력 안 함, max_lines 초과 시 truncate.
    macro_rules! emit_header {
        ($($arg:tt)*) => {{
            if !summary_mode {
                let line = format!($($arg)*);
                match max_lines {
                    Some(limit) if printed_lines >= limit => {
                        if !truncated {
                            println!("... 이하 생략 (--max-lines {} 도달)", limit);
                            truncated = true;
                        }
                    }
                    _ => {
                        println!("{}", line);
                        printed_lines += 1;
                    }
                }
            }
        }};
    }
    // emit_diff: 차이 라인. summary 모드에서는 카테고리별 카운트, 일반 모드에서는 "  [차이] {}" 형식.
    // 카테고리 추출: ":" 앞쪽 첫 토큰. controls[N].xxx 는 ".xxx" 만 추출.
    macro_rules! emit_diff {
        ($($arg:tt)*) => {{
            let body = format!($($arg)*);
            if summary_mode {
                let prefix = body.split(':').next().unwrap_or(&body);
                let cat = if let Some(pos) = prefix.rfind(']') {
                    prefix[pos + 1..].trim_start_matches('.').trim().to_string()
                } else {
                    prefix.trim().to_string()
                };
                let key = if cat.is_empty() { body.clone() } else { cat };
                *summary_buckets.entry(key).or_insert(0) += 1;
            } else {
                let line = format!("  [차이] {}", body);
                match max_lines {
                    Some(limit) if printed_lines >= limit => {
                        if !truncated {
                            println!("... 이하 생략 (--max-lines {} 도달)", limit);
                            truncated = true;
                        }
                    }
                    _ => {
                        println!("{}", line);
                        printed_lines += 1;
                    }
                }
            }
        }};
    }

    // 구역 수 비교
    if doc_a.sections.len() != doc_b.sections.len() {
        emit_diff!(
            "구역 수: A={} vs B={}",
            doc_a.sections.len(),
            doc_b.sections.len()
        );
    }

    let sec_count = doc_a.sections.len().min(doc_b.sections.len());
    let mut total_diffs = 0u32;

    for sec_idx in 0..sec_count {
        if let Some(sf) = section_filter {
            if sec_idx != sf {
                continue;
            }
        }

        let sec_a = &doc_a.sections[sec_idx];
        let sec_b = &doc_b.sections[sec_idx];

        if sec_a.paragraphs.len() != sec_b.paragraphs.len() {
            emit_diff!(
                "구역 {}: 문단 수 A={} vs B={}",
                sec_idx,
                sec_a.paragraphs.len(),
                sec_b.paragraphs.len()
            );
            total_diffs += 1;
        }

        let para_count = sec_a.paragraphs.len().min(sec_b.paragraphs.len());
        for pi in 0..para_count {
            if let Some(pf) = para_filter {
                if pi != pf {
                    continue;
                }
            }

            let pa = &sec_a.paragraphs[pi];
            let pb = &sec_b.paragraphs[pi];
            let mut diffs: Vec<String> = Vec::new();

            // 텍스트 비교
            if pa.text != pb.text {
                diffs.push(format!(
                    "text: A={:?} vs B={:?}",
                    pa.text.chars().take(30).collect::<String>(),
                    pb.text.chars().take(30).collect::<String>()
                ));
            }

            // char_count 비교
            if pa.char_count != pb.char_count {
                diffs.push(format!("cc: A={} vs B={}", pa.char_count, pb.char_count));
            }

            // char_offsets 비교
            if pa.char_offsets != pb.char_offsets {
                let len_a = pa.char_offsets.len();
                let len_b = pb.char_offsets.len();
                if len_a != len_b {
                    diffs.push(format!("char_offsets len: A={} vs B={}", len_a, len_b));
                } else {
                    let first_diff = pa
                        .char_offsets
                        .iter()
                        .zip(pb.char_offsets.iter())
                        .enumerate()
                        .find(|(_, (a, b))| a != b);
                    if let Some((idx, (a, b))) = first_diff {
                        diffs.push(format!("char_offsets[{}]: A={} vs B={}", idx, a, b));
                    }
                }
            }

            // para_shape_id 비교
            if pa.para_shape_id != pb.para_shape_id {
                diffs.push(format!(
                    "ps_id: A={} vs B={}",
                    pa.para_shape_id, pb.para_shape_id
                ));
            }

            // tab_extended 비교
            if pa.tab_extended.len() != pb.tab_extended.len() {
                diffs.push(format!(
                    "tab_ext count: A={} vs B={}",
                    pa.tab_extended.len(),
                    pb.tab_extended.len()
                ));
            } else {
                for (ti, (ta, tb)) in pa
                    .tab_extended
                    .iter()
                    .zip(pb.tab_extended.iter())
                    .enumerate()
                {
                    if tab_ext_semantic_differs(ta, tb) {
                        diffs.push(format!("tab_ext[{}]: A={:?} vs B={:?}", ti, ta, tb));
                        break;
                    }
                }
            }

            // LINE_SEG 비교
            if pa.line_segs.len() != pb.line_segs.len() {
                diffs.push(format!(
                    "line_segs count: A={} vs B={}",
                    pa.line_segs.len(),
                    pb.line_segs.len()
                ));
            } else {
                for (li, (la, lb)) in pa.line_segs.iter().zip(pb.line_segs.iter()).enumerate() {
                    if la.text_start != lb.text_start {
                        diffs.push(format!(
                            "ls[{}].ts: A={} vs B={}",
                            li, la.text_start, lb.text_start
                        ));
                    }
                    if la.vertical_pos != lb.vertical_pos {
                        diffs.push(format!(
                            "ls[{}].vpos: A={} vs B={}",
                            li, la.vertical_pos, lb.vertical_pos
                        ));
                    }
                    if la.line_height != lb.line_height {
                        diffs.push(format!(
                            "ls[{}].lh: A={} vs B={}",
                            li, la.line_height, lb.line_height
                        ));
                    }
                    if la.text_height != lb.text_height {
                        diffs.push(format!(
                            "ls[{}].th: A={} vs B={}",
                            li, la.text_height, lb.text_height
                        ));
                    }
                    if la.baseline_distance != lb.baseline_distance {
                        diffs.push(format!(
                            "ls[{}].bl: A={} vs B={}",
                            li, la.baseline_distance, lb.baseline_distance
                        ));
                    }
                    if la.line_spacing != lb.line_spacing {
                        diffs.push(format!(
                            "ls[{}].ls: A={} vs B={}",
                            li, la.line_spacing, lb.line_spacing
                        ));
                    }
                    if la.column_start != lb.column_start {
                        diffs.push(format!(
                            "ls[{}].cs: A={} vs B={}",
                            li, la.column_start, lb.column_start
                        ));
                    }
                    if la.segment_width != lb.segment_width {
                        diffs.push(format!(
                            "ls[{}].sw: A={} vs B={}",
                            li, la.segment_width, lb.segment_width
                        ));
                    }
                }
            }

            // 컨트롤 식별 비교
            if pa.controls.len() != pb.controls.len() {
                diffs.push(format!(
                    "controls count: A={} vs B={}",
                    pa.controls.len(),
                    pb.controls.len()
                ));
            }
            {
                use rhwp::model::control::Control;
                let ctrl_count = pa.controls.len().min(pb.controls.len());
                for ci in 0..ctrl_count {
                    let ca = &pa.controls[ci];
                    let cb = &pb.controls[ci];
                    match (ca, cb) {
                        (Control::Table(ta), Control::Table(tb)) => {
                            diff_table(&mut diffs, ci, ta, tb);
                        }
                        (Control::Picture(pic_a), Control::Picture(pic_b)) => {
                            diff_common_obj(&mut diffs, ci, "pic", &pic_a.common, &pic_b.common);
                        }
                        (Control::Shape(sa), Control::Shape(sb)) => {
                            diff_common_obj(&mut diffs, ci, "shape", sa.common(), sb.common());
                        }
                        _ if control_tag(ca) != control_tag(cb) => {
                            diffs.push(format!(
                                "ctrl[{}] type: A={} vs B={}",
                                ci,
                                control_tag(ca),
                                control_tag(cb)
                            ));
                        }
                        _ => {}
                    }
                }
            }

            // char_shapes 비교
            if pa.char_shapes.len() != pb.char_shapes.len() {
                diffs.push(format!(
                    "char_shapes count: A={} vs B={}",
                    pa.char_shapes.len(),
                    pb.char_shapes.len()
                ));
            } else {
                for (ci, (ca, cb)) in pa.char_shapes.iter().zip(pb.char_shapes.iter()).enumerate() {
                    if ca.start_pos != cb.start_pos {
                        diffs.push(format!(
                            "cs[{}].pos: A={} vs B={}",
                            ci, ca.start_pos, cb.start_pos
                        ));
                        break;
                    }
                    if ca.char_shape_id != cb.char_shape_id {
                        diffs.push(format!(
                            "cs[{}].id: A={} vs B={}",
                            ci, ca.char_shape_id, cb.char_shape_id
                        ));
                        break;
                    }
                }
            }

            if !diffs.is_empty() {
                let text_preview: String = pa.text.chars().take(30).collect();
                emit_header!("\n--- 문단 {}.{} --- \"{}\"", sec_idx, pi, text_preview);
                for d in &diffs {
                    emit_diff!("{}", d);
                }
                total_diffs += diffs.len() as u32;
            }
        }
    }

    // doc_info 비교: ParaShape
    {
        let ps_a = &doc_a.doc_info.para_shapes;
        let ps_b = &doc_b.doc_info.para_shapes;
        if ps_a.len() != ps_b.len() {
            emit_diff!("ParaShape 수: A={} vs B={}", ps_a.len(), ps_b.len());
            total_diffs += 1;
        }
        let ps_count = ps_a.len().min(ps_b.len());
        for i in 0..ps_count {
            let a = &ps_a[i];
            let b = &ps_b[i];
            let mut ps_diffs: Vec<String> = Vec::new();
            if a.margin_left != b.margin_left {
                ps_diffs.push(format!("ml: {}vs{}", a.margin_left, b.margin_left));
            }
            if a.margin_right != b.margin_right {
                ps_diffs.push(format!("mr: {}vs{}", a.margin_right, b.margin_right));
            }
            if a.indent != b.indent {
                ps_diffs.push(format!("indent: {}vs{}", a.indent, b.indent));
            }
            if a.tab_def_id != b.tab_def_id {
                ps_diffs.push(format!("tab_def: {}vs{}", a.tab_def_id, b.tab_def_id));
            }
            if a.spacing_before != b.spacing_before {
                ps_diffs.push(format!("sb: {}vs{}", a.spacing_before, b.spacing_before));
            }
            if a.spacing_after != b.spacing_after {
                ps_diffs.push(format!("sa: {}vs{}", a.spacing_after, b.spacing_after));
            }
            if a.line_spacing != b.line_spacing {
                ps_diffs.push(format!("ls: {}vs{}", a.line_spacing, b.line_spacing));
            }
            if !ps_diffs.is_empty() {
                emit_diff!("PS[{}] {}", i, ps_diffs.join(", "));
                total_diffs += ps_diffs.len() as u32;
            }
        }
    }

    // doc_info 비교: TabDef
    {
        let td_a = &doc_a.doc_info.tab_defs;
        let td_b = &doc_b.doc_info.tab_defs;
        if td_a.len() != td_b.len() {
            emit_diff!("TabDef 수: A={} vs B={}", td_a.len(), td_b.len());
            total_diffs += 1;
        }
        let td_count = td_a.len().min(td_b.len());
        for i in 0..td_count {
            let a = &td_a[i];
            let b = &td_b[i];
            if a.tabs.len() != b.tabs.len() {
                emit_diff!("TD[{}] 탭 수: A={} vs B={}", i, a.tabs.len(), b.tabs.len());
                total_diffs += 1;
            } else {
                for (ti, (ta, tb)) in a.tabs.iter().zip(b.tabs.iter()).enumerate() {
                    if ta.position != tb.position
                        || ta.tab_type != tb.tab_type
                        || ta.fill_type != tb.fill_type
                    {
                        emit_diff!(
                            "TD[{}][{}] pos: {}vs{}, type: {}vs{}, fill: {}vs{}",
                            i,
                            ti,
                            ta.position,
                            tb.position,
                            ta.tab_type,
                            tb.tab_type,
                            ta.fill_type,
                            tb.fill_type
                        );
                        total_diffs += 1;
                    }
                }
            }
        }
    }

    // [Task #653 보강] 요약 모드 출력 — 카테고리별 카운트 (내림차순 → 알파벳)
    if summary_mode {
        println!("=== 카테고리별 차이 요약 ===");
        let mut entries: Vec<(String, u32)> = summary_buckets.into_iter().collect();
        entries.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        for (cat, count) in &entries {
            println!("  {:>5}건  {}", count, cat);
        }
    }

    println!("\n=== 비교 완료: 차이 {} 건 ===", total_diffs);
}

fn extract_thumbnail(args: &[String]) {
    if args.is_empty() {
        eprintln!("사용법: rhwp thumbnail <파일.hwp> [옵션]");
        eprintln!("  -o, --output <파일>   출력 파일 경로");
        eprintln!("  --base64              base64 문자열 출력");
        eprintln!("  --data-uri            data:image/... URI 출력");
        std::process::exit(1);
    }

    let input_path = &args[0];
    let mut output_path: Option<String> = None;
    let mut mode = "file"; // "file", "base64", "data-uri"

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-o" | "--output" => {
                i += 1;
                if i < args.len() {
                    output_path = Some(args[i].clone());
                }
            }
            "--base64" => mode = "base64",
            "--data-uri" => mode = "data-uri",
            _ => {}
        }
        i += 1;
    }

    let data = match fs::read(input_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다: {} ({})", input_path, e);
            std::process::exit(1);
        }
    };

    let result = match rhwp::parser::extract_thumbnail_only(&data) {
        Some(r) => r,
        None => {
            eprintln!("오류: PrvImage 썸네일이 없습니다: {}", input_path);
            std::process::exit(1);
        }
    };

    let mime = match result.format.as_str() {
        "png" => "image/png",
        "bmp" => "image/bmp",
        "gif" => "image/gif",
        _ => "application/octet-stream",
    };

    match mode {
        "base64" => {
            use base64::Engine;
            let b64 = base64::engine::general_purpose::STANDARD.encode(&result.data);
            println!("{}", b64);
        }
        "data-uri" => {
            use base64::Engine;
            let b64 = base64::engine::general_purpose::STANDARD.encode(&result.data);
            println!("data:{};base64,{}", mime, b64);
        }
        _ => {
            // 파일 출력
            let out = output_path.unwrap_or_else(|| {
                let stem = Path::new(input_path)
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy();
                let ext = &result.format;
                format!("output/{}_thumb.{}", stem, ext)
            });

            // 출력 디렉토리 생성
            if let Some(parent) = Path::new(&out).parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent).ok();
                }
            }

            match fs::write(&out, &result.data) {
                Ok(_) => {
                    println!(
                        "썸네일 추출 완료: {} ({}x{}, {} bytes, {})",
                        out,
                        result.width,
                        result.height,
                        result.data.len(),
                        result.format
                    );
                }
                Err(e) => {
                    eprintln!("오류: 파일 저장 실패: {} ({})", out, e);
                    std::process::exit(1);
                }
            }
        }
    }
}

#[cfg(test)]
mod doc_mcp_hwp_write_cli_tests {
    use super::*;

    #[test]
    fn create_hwp_bytes_from_text_roundtrips_body_text() {
        let result =
            create_hwp_bytes_from_text_for_cli("사업계획서\n지원 목적", None).expect("create hwp");
        assert!(result.bytes.starts_with(&[0xD0, 0xCF, 0x11, 0xE0]));
        assert_eq!(result.paragraph_count, 2);

        let core =
            rhwp::document_core::DocumentCore::from_bytes(&result.bytes).expect("reload hwp");
        let text = core
            .document()
            .sections
            .iter()
            .flat_map(|s| s.paragraphs.iter())
            .map(|p| p.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(text.contains("사업계획서"));
        assert!(text.contains("지원 목적"));
    }

    #[test]
    fn replace_hwp_bytes_edits_existing_binary_hwp() {
        let created =
            create_hwp_bytes_from_text_for_cli("초안 보고서\n본문", None).expect("create hwp");
        let edited = replace_hwp_text_bytes_for_cli(&created.bytes, "초안", "확정", true, false)
            .expect("replace hwp text");
        assert_eq!(edited.count, 1);

        let core =
            rhwp::document_core::DocumentCore::from_bytes(&edited.bytes).expect("reload hwp");
        let text = core
            .document()
            .sections
            .iter()
            .flat_map(|s| s.paragraphs.iter())
            .map(|p| p.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(text.contains("확정 보고서"));
        assert!(!text.contains("초안 보고서"));
    }

    #[test]
    fn direct_body_text_insert_and_delete_roundtrip() {
        let created = create_hwp_bytes_from_text_for_cli("사업계획서", None).expect("create hwp");
        let inserted = insert_hwp_text_bytes_for_cli(&created.bytes, 0, 0, 2, " 세부")
            .expect("insert body text");
        assert_eq!(inserted.details["operation"], "insert-text");
        assert_eq!(inserted.details["charOffset"], 5);

        let deleted =
            delete_hwp_text_bytes_for_cli(&inserted.bytes, 0, 0, 0, 2).expect("delete body text");
        assert_eq!(deleted.details["operation"], "delete-text");
        assert_eq!(deleted.details["charOffset"], 0);

        let core =
            rhwp::document_core::DocumentCore::from_bytes(&deleted.bytes).expect("reload hwp");
        assert_eq!(
            core.document().sections[0].paragraphs[0].text,
            " 세부계획서"
        );
    }

    #[test]
    fn clickhere_field_creation_roundtrips_and_can_be_filled() {
        let created =
            create_hwp_bytes_from_text_for_cli("사업명:\n지원 목적", None).expect("create hwp");
        let inserted = insert_hwp_clickhere_field_bytes_for_cli(
            &created.bytes,
            0,
            0,
            4,
            "biz_name",
            "사업명",
            "사업명을 입력하세요",
            "미입력",
        )
        .expect("insert clickhere field");
        assert_eq!(inserted.details["operation"], "insert-clickhere-field");
        assert_eq!(inserted.details["name"], "biz_name");

        let fields = list_hwp_fields_json_for_cli(&inserted.bytes).expect("list fields");
        assert_eq!(fields["ok"], true);
        assert_eq!(fields["count"], 1);
        assert_eq!(fields["fields"][0]["name"], "biz_name");
        assert_eq!(fields["fields"][0]["guide"], "사업명");
        assert_eq!(fields["fields"][0]["value"], "미입력");

        let filled = set_hwp_field_bytes_for_cli(&inserted.bytes, "biz_name", "AI 전환 사업")
            .expect("set generated field");
        let fields = list_hwp_fields_json_for_cli(&filled.bytes).expect("list filled fields");
        assert_eq!(fields["fields"][0]["name"], "biz_name");
        assert_eq!(fields["fields"][0]["value"], "AI 전환 사업");

        rhwp::document_core::DocumentCore::from_bytes(&filled.bytes)
            .expect("reload generated clickhere field hwp");
    }

    #[test]
    fn clickhere_field_info_and_remove_roundtrip() {
        let created =
            create_hwp_bytes_from_text_for_cli("사업명:\n지원 목적", None).expect("create hwp");
        let inserted = insert_hwp_clickhere_field_bytes_for_cli(
            &created.bytes,
            0,
            0,
            4,
            "biz_name",
            "사업명",
            "사업명을 입력하세요",
            "미입력",
        )
        .expect("insert clickhere field");

        let info =
            get_hwp_field_info_json_for_cli(&inserted.bytes, 0, 0, 5).expect("get field info");
        assert_eq!(info["inField"], true);
        assert_eq!(info["fieldType"], "clickhere");
        assert_eq!(info["guideName"], "사업명");

        let removed = remove_hwp_field_bytes_for_cli(&inserted.bytes, 0, 0, 5)
            .expect("remove clickhere field");
        assert_eq!(removed.details["operation"], "remove-field");
        let fields = list_hwp_fields_json_for_cli(&removed.bytes).expect("list removed fields");
        assert_eq!(fields["count"], 0);

        let core =
            rhwp::document_core::DocumentCore::from_bytes(&removed.bytes).expect("reload hwp");
        assert_eq!(core.document().sections[0].paragraphs[0].text, "사업명:");
        assert!(!core.document().sections[0].paragraphs[0]
            .controls
            .iter()
            .any(|ctrl| matches!(ctrl, rhwp::model::control::Control::Field(_))));
    }

    #[test]
    fn form_object_create_and_set_roundtrips() {
        let created = create_hwp_bytes_from_text_for_cli("동의", None).expect("create hwp");
        let form = create_hwp_form_object_bytes_for_cli(
            &created.bytes,
            0,
            0,
            2,
            "checkbox",
            "agree",
            "동의",
            "",
            1200,
            900,
            1,
            true,
            "{}",
        )
        .expect("create checkbox form");
        assert_eq!(form.details["operation"], "create-form");
        assert_eq!(form.details["formType"], "CheckBox");
        assert_eq!(form.details["name"], "agree");
        assert_eq!(form.para_idx, 0);

        let info = get_hwp_form_info_json_for_cli(&form.bytes, 0, form.para_idx, form.control_idx)
            .expect("get form info");
        assert_eq!(info["formType"], "CheckBox");
        assert_eq!(info["caption"], "동의");
        assert_eq!(info["value"], 1);

        let updated = set_hwp_form_value_bytes_for_cli(
            &form.bytes,
            0,
            form.para_idx,
            form.control_idx,
            r#"{"value":0,"caption":"미동의"}"#,
        )
        .expect("set form value");
        assert_eq!(updated.details["operation"], "set-form");

        let info =
            get_hwp_form_info_json_for_cli(&updated.bytes, 0, form.para_idx, form.control_idx)
                .expect("get updated form info");
        assert_eq!(info["value"], 0);
        assert_eq!(info["caption"], "미동의");

        let core =
            rhwp::document_core::DocumentCore::from_bytes(&updated.bytes).expect("reload hwp");
        let form_ctrl = &core.document().sections[0].paragraphs[0].controls[form.control_idx];
        match form_ctrl {
            rhwp::model::control::Control::Form(f) => {
                assert_eq!(f.name, "agree");
                assert_eq!(f.value, 0);
                assert_eq!(f.caption, "미동의");
            }
            _ => panic!("expected form control"),
        }
    }

    #[test]
    fn form_object_in_table_cell_create_and_set_roundtrips() {
        let created = create_hwp_bytes_from_text_for_cli("사업 표", None).expect("create hwp");
        let table =
            create_hwp_table_bytes_for_cli(&created.bytes, 0, 0, 3, 2, 2).expect("create table");
        let cell = set_hwp_cell_text_bytes_for_cli(&table.bytes, table.para_idx, 0, 1, 0, "선택")
            .expect("set cell text");
        let form = create_hwp_cell_form_object_bytes_for_cli(
            &cell.bytes,
            0,
            table.para_idx,
            r#"[{"controlIndex":0,"cellIndex":1,"cellParaIndex":0}]"#,
            2,
            "checkbox",
            "agree",
            "동의",
            "",
            1200,
            900,
            1,
            true,
            "{}",
        )
        .expect("create cell checkbox form");
        assert_eq!(form.details["operation"], "create-form");
        assert_eq!(form.details["container"], "cell");
        assert_eq!(form.details["formType"], "CheckBox");
        assert_eq!(form.para_idx, table.para_idx);
        assert_eq!(form.control_idx, 0);

        let info = get_hwp_cell_form_info_json_for_cli(
            &form.bytes,
            0,
            table.para_idx,
            r#"[{"controlIndex":0,"cellIndex":1,"cellParaIndex":0}]"#,
            form.control_idx,
        )
        .expect("get cell form info");
        assert_eq!(info["container"], "cell");
        assert_eq!(info["caption"], "동의");
        assert_eq!(info["value"], 1);

        let updated = set_hwp_cell_form_value_bytes_for_cli(
            &form.bytes,
            0,
            table.para_idx,
            r#"[{"controlIndex":0,"cellIndex":1,"cellParaIndex":0}]"#,
            form.control_idx,
            r#"{"value":0,"caption":"미동의"}"#,
        )
        .expect("set cell form value");
        assert_eq!(updated.details["operation"], "set-form");
        assert_eq!(updated.details["container"], "cell");

        let info = get_hwp_cell_form_info_json_for_cli(
            &updated.bytes,
            0,
            table.para_idx,
            r#"[{"controlIndex":0,"cellIndex":1,"cellParaIndex":0}]"#,
            form.control_idx,
        )
        .expect("get updated cell form info");
        assert_eq!(info["value"], 0);
        assert_eq!(info["caption"], "미동의");

        let core =
            rhwp::document_core::DocumentCore::from_bytes(&updated.bytes).expect("reload hwp");
        let table_para = &core.document().sections[0].paragraphs[table.para_idx];
        let table = match &table_para.controls[table.control_idx] {
            rhwp::model::control::Control::Table(t) => t,
            _ => panic!("expected table control"),
        };
        let form_ctrl = &table.cells[1].paragraphs[0].controls[form.control_idx];
        match form_ctrl {
            rhwp::model::control::Control::Form(f) => {
                assert_eq!(f.name, "agree");
                assert_eq!(f.value, 0);
                assert_eq!(f.caption, "미동의");
            }
            _ => panic!("expected cell form control"),
        }
    }

    #[test]
    fn form_object_in_table_cell_create_and_set_roundtrips_by_row_col() {
        let created = create_hwp_bytes_from_text_for_cli("사업 표", None).expect("create hwp");
        let table =
            create_hwp_table_bytes_for_cli(&created.bytes, 0, 0, 3, 2, 2).expect("create table");
        let cell = set_hwp_cell_text_bytes_for_cli(&table.bytes, table.para_idx, 0, 3, 0, "선택")
            .expect("set cell text");
        let form = create_hwp_cell_form_object_at_bytes_for_cli(
            &cell.bytes,
            0,
            table.para_idx,
            table.control_idx,
            1,
            1,
            0,
            2,
            "checkbox",
            "agree",
            "동의",
            "",
            1200,
            900,
            1,
            true,
            "{}",
        )
        .expect("create cell checkbox form by row col");
        assert_eq!(form.details["operation"], "create-form");
        assert_eq!(form.details["container"], "cell");
        assert_eq!(form.details["row"], 1);
        assert_eq!(form.details["col"], 1);
        assert_eq!(form.details["cellIndex"], 3);
        assert_eq!(form.details["formType"], "CheckBox");

        let info = get_hwp_cell_form_info_at_json_for_cli(
            &form.bytes,
            0,
            table.para_idx,
            table.control_idx,
            1,
            1,
            0,
            form.control_idx,
        )
        .expect("get cell form info by row col");
        assert_eq!(info["container"], "cell");
        assert_eq!(info["row"], 1);
        assert_eq!(info["col"], 1);
        assert_eq!(info["cellIndex"], 3);
        assert_eq!(info["caption"], "동의");
        assert_eq!(info["value"], 1);

        let updated = set_hwp_cell_form_value_at_bytes_for_cli(
            &form.bytes,
            0,
            table.para_idx,
            table.control_idx,
            1,
            1,
            0,
            form.control_idx,
            r#"{"value":0,"caption":"미동의"}"#,
        )
        .expect("set cell form value by row col");
        assert_eq!(updated.details["operation"], "set-form");
        assert_eq!(updated.details["container"], "cell");
        assert_eq!(updated.details["row"], 1);
        assert_eq!(updated.details["col"], 1);
        assert_eq!(updated.details["cellIndex"], 3);

        let info = get_hwp_cell_form_info_at_json_for_cli(
            &updated.bytes,
            0,
            table.para_idx,
            table.control_idx,
            1,
            1,
            0,
            form.control_idx,
        )
        .expect("get updated cell form info by row col");
        assert_eq!(info["value"], 0);
        assert_eq!(info["caption"], "미동의");
    }

    #[test]
    fn form_object_delete_roundtrips_for_body_and_cell() {
        let created = create_hwp_bytes_from_text_for_cli("동의", None).expect("create hwp");
        let form = create_hwp_form_object_bytes_for_cli(
            &created.bytes,
            0,
            0,
            2,
            "checkbox",
            "agree",
            "동의",
            "",
            1200,
            900,
            1,
            true,
            "{}",
        )
        .expect("create body form");
        let deleted =
            delete_hwp_form_object_bytes_for_cli(&form.bytes, 0, form.para_idx, form.control_idx)
                .expect("delete body form");
        assert_eq!(deleted.details["operation"], "delete-form");
        let core =
            rhwp::document_core::DocumentCore::from_bytes(&deleted.bytes).expect("reload body");
        let body_controls = &core.document().sections[0].paragraphs[form.para_idx].controls;
        assert!(
            !body_controls
                .iter()
                .any(|control| matches!(control, rhwp::model::control::Control::Form(_))),
            "body controls after delete: {:?}",
            body_controls
        );

        let created = create_hwp_bytes_from_text_for_cli("사업 표", None).expect("create hwp");
        let table =
            create_hwp_table_bytes_for_cli(&created.bytes, 0, 0, 0, 2, 2).expect("create table");
        let form = create_hwp_cell_form_object_at_bytes_for_cli(
            &table.bytes,
            0,
            table.para_idx,
            table.control_idx,
            1,
            1,
            0,
            0,
            "checkbox",
            "agree",
            "동의",
            "",
            1200,
            900,
            1,
            true,
            "{}",
        )
        .expect("create cell form");
        let deleted = delete_hwp_cell_form_object_at_bytes_for_cli(
            &form.bytes,
            0,
            table.para_idx,
            table.control_idx,
            1,
            1,
            0,
            form.control_idx,
        )
        .expect("delete cell form by row col");
        assert_eq!(deleted.details["operation"], "delete-form");
        assert_eq!(deleted.details["container"], "cell");
        assert_eq!(deleted.details["row"], 1);
        assert_eq!(deleted.details["col"], 1);
        assert_eq!(deleted.details["cellIndex"], 3);

        let core =
            rhwp::document_core::DocumentCore::from_bytes(&deleted.bytes).expect("reload cell");
        let table_para = &core.document().sections[0].paragraphs[table.para_idx];
        let table = match &table_para.controls[table.control_idx] {
            rhwp::model::control::Control::Table(t) => t,
            _ => panic!("expected table control"),
        };
        let cell_controls = &table.cells[3].paragraphs[0].controls;
        assert!(
            !cell_controls
                .iter()
                .any(|control| matches!(control, rhwp::model::control::Control::Form(_))),
            "cell controls after delete: {:?}",
            cell_controls
        );
    }

    #[test]
    fn form_object_list_finds_body_and_cell_forms() {
        let created = create_hwp_bytes_from_text_for_cli("동의", None).expect("create hwp");
        let body_form = create_hwp_form_object_bytes_for_cli(
            &created.bytes,
            0,
            0,
            2,
            "checkbox",
            "agree",
            "동의",
            "",
            1200,
            900,
            1,
            true,
            "{}",
        )
        .expect("create body form");
        let table =
            create_hwp_table_bytes_for_cli(&body_form.bytes, 0, 0, 3, 2, 2).expect("create table");
        let cell_form = create_hwp_cell_form_object_at_bytes_for_cli(
            &table.bytes,
            0,
            table.para_idx,
            table.control_idx,
            1,
            1,
            0,
            0,
            "edit",
            "biz_no",
            "",
            "123",
            1800,
            900,
            0,
            true,
            "{}",
        )
        .expect("create cell edit form");

        let list = list_hwp_forms_json_for_cli(&cell_form.bytes).expect("list forms");
        assert_eq!(list["ok"], true);
        assert_eq!(list["count"], 2);
        let forms = list["forms"].as_array().expect("forms array");
        let body = forms
            .iter()
            .find(|form| form["name"] == "agree")
            .expect("body form");
        assert_eq!(body["container"], "body");
        assert_eq!(body["section"], 0);
        assert_eq!(body["paragraph"], body_form.para_idx);
        assert_eq!(body["control"], body_form.control_idx);
        assert_eq!(body["formType"], "CheckBox");
        assert_eq!(body["caption"], "동의");
        assert_eq!(body["value"], 1);

        let cell = forms
            .iter()
            .find(|form| form["name"] == "biz_no")
            .expect("cell form");
        assert_eq!(cell["container"], "cell");
        assert_eq!(cell["section"], 0);
        assert_eq!(cell["paragraph"], table.para_idx);
        assert_eq!(cell["control"], cell_form.control_idx);
        assert_eq!(cell["tableControl"], table.control_idx);
        assert_eq!(cell["cellIndex"], 3);
        assert_eq!(cell["cellParagraph"], 0);
        assert_eq!(cell["cellPath"][0]["controlIndex"], table.control_idx);
        assert_eq!(cell["cellPath"][0]["cellIndex"], 3);
        assert_eq!(cell["cellPath"][0]["cellParaIndex"], 0);
        assert_eq!(cell["formType"], "Edit");
        assert_eq!(cell["text"], "123");
    }

    #[test]
    fn precise_paragraph_and_table_edit_roundtrip() {
        let created = create_hwp_bytes_from_text_for_cli("원본 문단", None).expect("create hwp");
        let paragraph = set_hwp_paragraph_text_bytes_for_cli(&created.bytes, 0, 0, "사업 개요")
            .expect("set paragraph");
        let table =
            create_hwp_table_bytes_for_cli(&paragraph.bytes, 0, 0, 5, 2, 2).expect("create table");
        let cell = set_hwp_cell_text_bytes_for_cli(&table.bytes, table.para_idx, 0, 0, 0, "항목")
            .expect("set first cell");
        let cell = set_hwp_cell_text_bytes_for_cli(&cell.bytes, table.para_idx, 0, 1, 0, "금액")
            .expect("set second cell");

        let structure = extract_hwp_structure_json_for_cli(&cell.bytes).expect("extract structure");
        assert_eq!(
            structure["sections"][0]["paragraphs"][0]["text"],
            "사업 개요"
        );
        assert_eq!(structure["sections"][0]["tables"][0]["rowCount"], 2);
        assert_eq!(
            structure["sections"][0]["tables"][0]["cells"][0]["text"],
            "항목"
        );
        assert_eq!(
            structure["sections"][0]["tables"][0]["cells"][1]["text"],
            "금액"
        );

        let core =
            rhwp::document_core::DocumentCore::from_bytes(&cell.bytes).expect("reload edited hwp");
        let table_para = &core.document().sections[0].paragraphs[table.para_idx];
        let table = match &table_para.controls[0] {
            rhwp::model::control::Control::Table(t) => t,
            _ => panic!("expected table control"),
        };
        assert_eq!(table.cells[0].paragraphs[0].text, "항목");
        assert_eq!(table.cells[1].paragraphs[0].text, "금액");
    }

    #[test]
    fn table_cell_field_name_roundtrips_and_can_be_filled() {
        let created = create_hwp_bytes_from_text_for_cli("사업 표", None).expect("create hwp");
        let table =
            create_hwp_table_bytes_for_cli(&created.bytes, 0, 0, 4, 2, 2).expect("create table");
        let fielded = set_hwp_cell_field_bytes_for_cli(
            &table.bytes,
            table.para_idx,
            table.control_idx,
            1,
            Some("biz_cost"),
        )
        .expect("set cell field");
        assert_eq!(fielded.details["operation"], "set-cell-field");
        assert_eq!(fielded.details["name"], "biz_cost");

        let fields = list_hwp_fields_json_for_cli(&fielded.bytes).expect("list cell fields");
        assert_eq!(fields["count"], 1);
        assert_eq!(fields["fields"][0]["name"], "biz_cost");
        assert_eq!(fields["fields"][0]["value"], "");

        let filled = set_hwp_field_bytes_for_cli(&fielded.bytes, "biz_cost", "1,200만원")
            .expect("fill cell field");
        let fields = list_hwp_fields_json_for_cli(&filled.bytes).expect("list filled cell field");
        assert_eq!(fields["fields"][0]["name"], "biz_cost");
        assert_eq!(fields["fields"][0]["value"], "1,200만원");

        let cleared = set_hwp_cell_field_bytes_for_cli(
            &filled.bytes,
            table.para_idx,
            table.control_idx,
            1,
            None,
        )
        .expect("clear cell field");
        assert_eq!(cleared.details["operation"], "clear-cell-field");
        let fields = list_hwp_fields_json_for_cli(&cleared.bytes).expect("list cleared fields");
        assert_eq!(fields["count"], 0);

        let core =
            rhwp::document_core::DocumentCore::from_bytes(&cleared.bytes).expect("reload hwp");
        let table_para = &core.document().sections[0].paragraphs[table.para_idx];
        let table = match &table_para.controls[table.control_idx] {
            rhwp::model::control::Control::Table(t) => t,
            _ => panic!("expected table control"),
        };
        assert_eq!(table.cells[1].field_name, None);
        assert_eq!(table.cells[1].paragraphs[0].text, "1,200만원");
    }

    #[test]
    fn textbox_clickhere_field_roundtrips_and_can_be_filled() {
        let created = create_hwp_bytes_from_text_for_cli("사업 양식", None).expect("create hwp");
        let textbox = create_hwp_shape_bytes_for_cli(
            &created.bytes,
            0,
            0,
            0,
            6000,
            1600,
            1000,
            1000,
            false,
            "Square",
            "textbox",
            false,
            false,
            "[]",
        )
        .expect("create textbox");

        let fielded = insert_hwp_nested_clickhere_field_bytes_for_cli(
            &textbox.bytes,
            0,
            textbox.para_idx,
            textbox.control_idx,
            0,
            0,
            0,
            true,
            "biz_summary",
            "사업 요약",
            "요약 입력",
            "미입력",
        )
        .expect("insert textbox field");
        assert_eq!(fielded.details["operation"], "insert-clickhere-field");
        assert_eq!(fielded.details["container"], "textbox");
        assert_eq!(fielded.details["name"], "biz_summary");

        let fields = list_hwp_fields_json_for_cli(&fielded.bytes).expect("list textbox fields");
        assert_eq!(fields["count"], 1);
        assert_eq!(fields["fields"][0]["name"], "biz_summary");
        assert_eq!(fields["fields"][0]["value"], "미입력");
        assert_eq!(
            fields["fields"][0]["location"]["path"][0]["type"],
            "textbox"
        );

        let filled = set_hwp_field_bytes_for_cli(&fielded.bytes, "biz_summary", "AI LCA 자동화")
            .expect("fill textbox field");
        let fields = list_hwp_fields_json_for_cli(&filled.bytes).expect("list filled fields");
        assert_eq!(fields["fields"][0]["value"], "AI LCA 자동화");

        let info = get_hwp_nested_field_info_json_for_cli(
            &filled.bytes,
            0,
            textbox.para_idx,
            textbox.control_idx,
            0,
            0,
            2,
            true,
        )
        .expect("get textbox field info");
        assert_eq!(info["inField"], true);
        assert_eq!(info["fieldType"], "clickhere");

        let removed = remove_hwp_nested_field_bytes_for_cli(
            &filled.bytes,
            0,
            textbox.para_idx,
            textbox.control_idx,
            0,
            0,
            2,
            true,
        )
        .expect("remove textbox field");
        let fields = list_hwp_fields_json_for_cli(&removed.bytes).expect("list removed fields");
        assert_eq!(fields["count"], 0);
    }

    #[test]
    fn table_cell_row_col_addressing_sets_text_and_field() {
        let created = create_hwp_bytes_from_text_for_cli("사업 표", None).expect("create hwp");
        let table =
            create_hwp_table_bytes_for_cli(&created.bytes, 0, 0, 4, 2, 2).expect("create table");

        let texted = set_hwp_cell_text_by_position_bytes_for_cli(
            &table.bytes,
            table.para_idx,
            table.control_idx,
            1,
            0,
            0,
            "담당자",
        )
        .expect("set cell text by row/col");
        assert_eq!(texted.details["row"], 1);
        assert_eq!(texted.details["col"], 0);
        assert_eq!(texted.details["cell"], 2);

        let fielded = set_hwp_cell_field_by_position_bytes_for_cli(
            &texted.bytes,
            table.para_idx,
            table.control_idx,
            1,
            1,
            Some("biz_manager"),
        )
        .expect("set cell field by row/col");
        assert_eq!(fielded.details["row"], 1);
        assert_eq!(fielded.details["col"], 1);
        assert_eq!(fielded.details["cell"], 3);

        let filled = set_hwp_field_bytes_for_cli(&fielded.bytes, "biz_manager", "홍길동")
            .expect("fill row/col cell field");
        let fields = list_hwp_fields_json_for_cli(&filled.bytes).expect("list fields");
        assert_eq!(fields["fields"][0]["name"], "biz_manager");
        assert_eq!(fields["fields"][0]["value"], "홍길동");

        let core = rhwp::document_core::DocumentCore::from_bytes(&filled.bytes).expect("reload");
        let table_para = &core.document().sections[0].paragraphs[table.para_idx];
        let table = match &table_para.controls[table.control_idx] {
            rhwp::model::control::Control::Table(t) => t,
            _ => panic!("expected table"),
        };
        assert_eq!(table.cells[2].paragraphs[0].text, "담당자");
        assert_eq!(table.cells[3].paragraphs[0].text, "홍길동");
    }

    #[test]
    fn table_cell_text_insert_and_delete_roundtrip() {
        let created = create_hwp_bytes_from_text_for_cli("사업 표", None).expect("create hwp");
        let table =
            create_hwp_table_bytes_for_cli(&created.bytes, 0, 0, 4, 2, 2).expect("create table");
        let texted = set_hwp_cell_text_by_position_bytes_for_cli(
            &table.bytes,
            table.para_idx,
            table.control_idx,
            1,
            0,
            0,
            "사업계획서",
        )
        .expect("set cell text");

        let inserted = insert_hwp_cell_text_by_position_bytes_for_cli(
            &texted.bytes,
            table.para_idx,
            table.control_idx,
            1,
            0,
            0,
            2,
            " 세부",
        )
        .expect("insert cell text by row/col");
        assert_eq!(inserted.details["operation"], "insert-cell-text");
        assert_eq!(inserted.details["row"], 1);
        assert_eq!(inserted.details["col"], 0);
        assert_eq!(inserted.details["cell"], 2);
        assert_eq!(inserted.details["charOffset"], 5);

        let deleted = delete_hwp_cell_text_bytes_for_cli(
            &inserted.bytes,
            table.para_idx,
            table.control_idx,
            2,
            0,
            0,
            2,
        )
        .expect("delete cell text by exact cell");
        assert_eq!(deleted.details["operation"], "delete-cell-text");
        assert_eq!(deleted.details["cell"], 2);
        assert_eq!(deleted.details["charOffset"], 0);

        let core = rhwp::document_core::DocumentCore::from_bytes(&deleted.bytes).expect("reload");
        let table_para = &core.document().sections[0].paragraphs[table.para_idx];
        let table = match &table_para.controls[table.control_idx] {
            rhwp::model::control::Control::Table(t) => t,
            _ => panic!("expected table control"),
        };
        assert_eq!(table.cells[2].paragraphs[0].text, " 세부계획서");
    }

    #[test]
    fn table_cell_paragraph_split_and_merge_roundtrip() {
        let created = create_hwp_bytes_from_text_for_cli("사업 표", None).expect("create hwp");
        let table =
            create_hwp_table_bytes_for_cli(&created.bytes, 0, 0, 4, 2, 2).expect("create table");
        let texted = set_hwp_cell_text_by_position_bytes_for_cli(
            &table.bytes,
            table.para_idx,
            table.control_idx,
            1,
            0,
            0,
            "착수보고완료보고",
        )
        .expect("set cell text");

        let split = split_hwp_cell_paragraph_by_position_bytes_for_cli(
            &texted.bytes,
            table.para_idx,
            table.control_idx,
            1,
            0,
            0,
            4,
        )
        .expect("split cell paragraph by row/col");
        assert_eq!(split.details["operation"], "split-cell-paragraph");
        assert_eq!(split.details["row"], 1);
        assert_eq!(split.details["col"], 0);
        assert_eq!(split.details["cell"], 2);
        assert_eq!(split.details["cellParaIndex"], 1);

        let core = rhwp::document_core::DocumentCore::from_bytes(&split.bytes).expect("reload");
        let table_para = &core.document().sections[0].paragraphs[table.para_idx];
        let table_ref = match &table_para.controls[table.control_idx] {
            rhwp::model::control::Control::Table(t) => t,
            _ => panic!("expected table control"),
        };
        assert_eq!(table_ref.cells[2].paragraphs.len(), 2);
        assert_eq!(table_ref.cells[2].paragraphs[0].text, "착수보고");
        assert_eq!(table_ref.cells[2].paragraphs[1].text, "완료보고");

        let merged = merge_hwp_cell_paragraph_bytes_for_cli(
            &split.bytes,
            table.para_idx,
            table.control_idx,
            2,
            1,
        )
        .expect("merge cell paragraph by exact cell");
        assert_eq!(merged.details["operation"], "merge-cell-paragraph");
        assert_eq!(merged.details["cell"], 2);
        assert_eq!(merged.details["cellParaIndex"], 0);
        assert_eq!(merged.details["charOffset"], 4);

        let core = rhwp::document_core::DocumentCore::from_bytes(&merged.bytes).expect("reload");
        let table_para = &core.document().sections[0].paragraphs[table.para_idx];
        let table_ref = match &table_para.controls[table.control_idx] {
            rhwp::model::control::Control::Table(t) => t,
            _ => panic!("expected table control"),
        };
        assert_eq!(table_ref.cells[2].paragraphs.len(), 1);
        assert_eq!(table_ref.cells[2].paragraphs[0].text, "착수보고완료보고");
    }

    #[test]
    fn table_cell_paragraph_insert_and_delete_roundtrip() {
        let created = create_hwp_bytes_from_text_for_cli("사업 표", None).expect("create hwp");
        let table =
            create_hwp_table_bytes_for_cli(&created.bytes, 0, 0, 4, 2, 2).expect("create table");
        let texted = set_hwp_cell_text_by_position_bytes_for_cli(
            &table.bytes,
            table.para_idx,
            table.control_idx,
            1,
            0,
            0,
            "기존",
        )
        .expect("set cell text");

        let inserted = insert_hwp_cell_paragraph_by_position_bytes_for_cli(
            &texted.bytes,
            table.para_idx,
            table.control_idx,
            1,
            0,
            1,
            Some("추가"),
        )
        .expect("insert cell paragraph by row/col");
        assert_eq!(inserted.details["operation"], "insert-cell-paragraph");
        assert_eq!(inserted.details["row"], 1);
        assert_eq!(inserted.details["col"], 0);
        assert_eq!(inserted.details["cell"], 2);
        assert_eq!(inserted.details["cellParaIndex"], 1);

        let core = rhwp::document_core::DocumentCore::from_bytes(&inserted.bytes).expect("reload");
        let table_para = &core.document().sections[0].paragraphs[table.para_idx];
        let table_ref = match &table_para.controls[table.control_idx] {
            rhwp::model::control::Control::Table(t) => t,
            _ => panic!("expected table control"),
        };
        assert_eq!(table_ref.cells[2].paragraphs.len(), 2);
        assert_eq!(table_ref.cells[2].paragraphs[0].text, "기존");
        assert_eq!(table_ref.cells[2].paragraphs[1].text, "추가");

        let deleted = delete_hwp_cell_paragraph_bytes_for_cli(
            &inserted.bytes,
            table.para_idx,
            table.control_idx,
            2,
            0,
        )
        .expect("delete cell paragraph by exact cell");
        assert_eq!(deleted.details["operation"], "delete-cell-paragraph");
        assert_eq!(deleted.details["cell"], 2);
        assert_eq!(deleted.details["cellParaIndex"], 0);
        assert_eq!(deleted.details["newParagraphCount"], 1);

        let core = rhwp::document_core::DocumentCore::from_bytes(&deleted.bytes).expect("reload");
        let table_para = &core.document().sections[0].paragraphs[table.para_idx];
        let table_ref = match &table_para.controls[table.control_idx] {
            rhwp::model::control::Control::Table(t) => t,
            _ => panic!("expected table control"),
        };
        assert_eq!(table_ref.cells[2].paragraphs.len(), 1);
        assert_eq!(table_ref.cells[2].paragraphs[0].text, "추가");
    }

    #[test]
    fn paragraph_structure_edits_roundtrip() {
        let created =
            create_hwp_bytes_from_text_for_cli("첫 문단\n둘째 문단", None).expect("create hwp");
        let inserted = insert_hwp_paragraph_bytes_for_cli(&created.bytes, 0, 1, None)
            .expect("insert paragraph");
        assert_eq!(inserted.details["operation"], "insert-paragraph");
        assert_eq!(inserted.details["paraIdx"], 1);
        let inserted = set_hwp_paragraph_text_bytes_for_cli(&inserted.bytes, 0, 1, "삽입 문단")
            .expect("set inserted paragraph");
        let structure =
            extract_hwp_structure_json_for_cli(&inserted.bytes).expect("extract inserted");
        assert_eq!(structure["sections"][0]["paragraphCount"], 3);
        assert_eq!(
            structure["sections"][0]["paragraphs"][1]["text"],
            "삽입 문단"
        );

        let split =
            split_hwp_paragraph_bytes_for_cli(&inserted.bytes, 0, 2, 2).expect("split paragraph");
        assert_eq!(split.details["operation"], "split-paragraph");
        assert_eq!(split.details["paraIdx"], 3);
        let structure = extract_hwp_structure_json_for_cli(&split.bytes).expect("extract split");
        assert_eq!(structure["sections"][0]["paragraphCount"], 4);
        assert_eq!(structure["sections"][0]["paragraphs"][2]["text"], "둘째");
        assert_eq!(structure["sections"][0]["paragraphs"][3]["text"], " 문단");

        let merged =
            merge_hwp_paragraph_bytes_for_cli(&split.bytes, 0, 3).expect("merge paragraph");
        assert_eq!(merged.details["operation"], "merge-paragraph");
        let structure = extract_hwp_structure_json_for_cli(&merged.bytes).expect("extract merged");
        assert_eq!(structure["sections"][0]["paragraphCount"], 3);
        assert_eq!(
            structure["sections"][0]["paragraphs"][2]["text"],
            "둘째 문단"
        );

        let deleted =
            delete_hwp_paragraph_bytes_for_cli(&merged.bytes, 0, 1).expect("delete paragraph");
        assert_eq!(deleted.details["operation"], "delete-paragraph");
        let structure =
            extract_hwp_structure_json_for_cli(&deleted.bytes).expect("extract deleted");
        assert_eq!(structure["sections"][0]["paragraphCount"], 2);
        assert_eq!(structure["sections"][0]["paragraphs"][0]["text"], "첫 문단");
        assert_eq!(
            structure["sections"][0]["paragraphs"][1]["text"],
            "둘째 문단"
        );

        rhwp::document_core::DocumentCore::from_bytes(&deleted.bytes)
            .expect("reload paragraph-structure-edited hwp");
    }

    #[test]
    fn paragraph_copy_roundtrip_preserves_text() {
        let created =
            create_hwp_bytes_from_text_for_cli("요약\n반복 문단", None).expect("create hwp");

        let copied =
            copy_hwp_paragraph_bytes_for_cli(&created.bytes, 0, 1, true).expect("copy paragraph");
        assert_eq!(copied.details["operation"], "copy-paragraph");
        assert_eq!(copied.details["sourceParaIdx"], 1);
        assert_eq!(copied.details["targetParaIdx"], 2);
        assert_eq!(copied.details["newParagraphCount"], 3);

        let structure = extract_hwp_structure_json_for_cli(&copied.bytes).expect("extract copy");
        assert_eq!(structure["sections"][0]["paragraphCount"], 3);
        assert_eq!(structure["sections"][0]["paragraphs"][0]["text"], "요약");
        assert_eq!(
            structure["sections"][0]["paragraphs"][1]["text"],
            "반복 문단"
        );
        assert_eq!(
            structure["sections"][0]["paragraphs"][2]["text"],
            "반복 문단"
        );

        rhwp::document_core::DocumentCore::from_bytes(&copied.bytes)
            .expect("reload paragraph-copy-edited hwp");
    }

    #[test]
    fn paragraph_range_copy_roundtrip_preserves_order_and_text() {
        let created = create_hwp_bytes_from_text_for_cli("제목\n가 항목\n나 항목\n끝", None)
            .expect("create hwp");

        let copied = copy_hwp_paragraph_range_bytes_for_cli(&created.bytes, 0, 1, 2, true)
            .expect("copy paragraph range");
        assert_eq!(copied.details["operation"], "copy-paragraph-range");
        assert_eq!(copied.details["sourceStartParaIdx"], 1);
        assert_eq!(copied.details["sourceEndParaIdx"], 2);
        assert_eq!(copied.details["targetStartParaIdx"], 3);
        assert_eq!(copied.details["targetEndParaIdx"], 4);
        assert_eq!(copied.details["copiedCount"], 2);
        assert_eq!(copied.details["newParagraphCount"], 6);

        let structure = extract_hwp_structure_json_for_cli(&copied.bytes).expect("extract copy");
        let texts: Vec<&str> = structure["sections"][0]["paragraphs"]
            .as_array()
            .expect("paragraphs")
            .iter()
            .map(|p| p["text"].as_str().expect("text"))
            .collect();
        assert_eq!(
            texts,
            vec!["제목", "가 항목", "나 항목", "가 항목", "나 항목", "끝"]
        );

        rhwp::document_core::DocumentCore::from_bytes(&copied.bytes)
            .expect("reload paragraph-range-copy-edited hwp");
    }

    #[test]
    fn paragraph_range_copy_can_replace_only_the_copied_block() {
        let created =
            create_hwp_bytes_from_text_for_cli("양식\n회사: {{회사}}\n금액: {{금액}}\n끝", None)
                .expect("create hwp");

        let copied = copy_hwp_paragraph_range_with_replacements_bytes_for_cli(
            &created.bytes,
            0,
            1,
            2,
            true,
            &[
                ("{{회사}}".to_string(), "평화오일씰공업".to_string()),
                ("{{금액}}".to_string(), "123,456".to_string()),
            ],
        )
        .expect("copy paragraph range with replacements");
        assert_eq!(copied.details["operation"], "copy-paragraph-range");
        assert_eq!(copied.details["replacementCount"], 2);

        let structure = extract_hwp_structure_json_for_cli(&copied.bytes).expect("extract copy");
        let texts: Vec<&str> = structure["sections"][0]["paragraphs"]
            .as_array()
            .expect("paragraphs")
            .iter()
            .map(|p| p["text"].as_str().expect("text"))
            .collect();
        assert_eq!(
            texts,
            vec![
                "양식",
                "회사: {{회사}}",
                "금액: {{금액}}",
                "회사: 평화오일씰공업",
                "금액: 123,456",
                "끝"
            ]
        );

        rhwp::document_core::DocumentCore::from_bytes(&copied.bytes)
            .expect("reload paragraph-range-copy-replaced hwp");
    }

    #[test]
    fn layout_break_and_column_def_roundtrip() {
        let created = create_hwp_bytes_from_text_for_cli("가나다라", None).expect("create hwp");
        let columned = set_hwp_column_def_bytes_for_cli(&created.bytes, 0, 2, 1, true, 720)
            .expect("set column def");
        assert_eq!(columned.details["operation"], "set-column-def");
        let core =
            rhwp::document_core::DocumentCore::from_bytes(&columned.bytes).expect("reload columns");
        let column_def = core.document().sections[0].paragraphs[0]
            .controls
            .iter()
            .find_map(|ctrl| match ctrl {
                rhwp::model::control::Control::ColumnDef(column_def) => Some(column_def),
                _ => None,
            })
            .expect("column def control");
        assert_eq!(column_def.column_count, 2);
        assert_eq!(
            column_def.column_type,
            rhwp::model::page::ColumnType::Distribute
        );
        assert!(column_def.same_width);
        assert_eq!(column_def.spacing, 720);

        let paged =
            insert_hwp_page_break_bytes_for_cli(&columned.bytes, 0, 0, 2).expect("page break");
        assert_eq!(paged.details["operation"], "insert-page-break");
        assert_eq!(paged.details["paraIdx"], 1);
        let core =
            rhwp::document_core::DocumentCore::from_bytes(&paged.bytes).expect("reload page break");
        assert_eq!(core.document().sections[0].paragraphs.len(), 2);
        assert_eq!(core.document().sections[0].paragraphs[0].text, "가나");
        assert_eq!(core.document().sections[0].paragraphs[1].text, "다라");
        assert_eq!(
            core.document().sections[0].paragraphs[1].column_type,
            rhwp::model::paragraph::ColumnBreakType::Page
        );
        assert_eq!(
            core.document().sections[0].paragraphs[1].raw_break_type,
            0x04
        );

        let column_break =
            insert_hwp_column_break_bytes_for_cli(&paged.bytes, 0, 1, 1).expect("column break");
        assert_eq!(column_break.details["operation"], "insert-column-break");
        assert_eq!(column_break.details["paraIdx"], 2);
        let core = rhwp::document_core::DocumentCore::from_bytes(&column_break.bytes)
            .expect("reload column break");
        assert_eq!(core.document().sections[0].paragraphs.len(), 3);
        assert_eq!(core.document().sections[0].paragraphs[1].text, "다");
        assert_eq!(core.document().sections[0].paragraphs[2].text, "라");
        assert_eq!(
            core.document().sections[0].paragraphs[2].column_type,
            rhwp::model::paragraph::ColumnBreakType::Column
        );
        assert_eq!(
            core.document().sections[0].paragraphs[2].raw_break_type,
            0x08
        );
    }

    #[test]
    fn page_number_and_page_hide_controls_roundtrip() {
        let created = create_hwp_bytes_from_text_for_cli("표지\n본문", None).expect("create hwp");
        let numbered = insert_hwp_new_number_bytes_for_cli(&created.bytes, 0, 1, 0, 3)
            .expect("insert new page number");
        assert_eq!(numbered.details["operation"], "insert-new-number");
        assert_eq!(numbered.details["startNumber"], 3);

        let core =
            rhwp::document_core::DocumentCore::from_bytes(&numbered.bytes).expect("reload number");
        let new_number = core.document().sections[0].paragraphs[1]
            .controls
            .iter()
            .find_map(|ctrl| match ctrl {
                rhwp::model::control::Control::NewNumber(nn) => Some(nn),
                _ => None,
            })
            .expect("new number control");
        assert_eq!(new_number.number, 3);
        assert_eq!(
            new_number.number_type,
            rhwp::model::control::AutoNumberType::Page
        );

        let hidden = set_hwp_page_hide_bytes_for_cli(
            &numbered.bytes,
            0,
            0,
            true,
            false,
            true,
            true,
            false,
            true,
        )
        .expect("set page hide");
        assert_eq!(hidden.details["operation"], "set-page-hide");
        let info = get_hwp_page_hide_json_for_cli(&hidden.bytes, 0, 0).expect("get page hide");
        assert_eq!(info["exists"], true);
        assert_eq!(info["hideHeader"], true);
        assert_eq!(info["hideMasterPage"], true);
        assert_eq!(info["hideBorder"], true);
        assert_eq!(info["hidePageNum"], true);
        assert_eq!(info["hideFooter"], false);
        assert_eq!(info["hideFill"], false);

        let shown = set_hwp_page_hide_bytes_for_cli(
            &hidden.bytes,
            0,
            0,
            false,
            false,
            false,
            false,
            false,
            false,
        )
        .expect("clear page hide");
        let info =
            get_hwp_page_hide_json_for_cli(&shown.bytes, 0, 0).expect("get cleared page hide");
        assert_eq!(info["exists"], false);

        rhwp::document_core::DocumentCore::from_bytes(&shown.bytes)
            .expect("reload page-number/page-hide hwp");
    }

    #[test]
    fn bookmark_controls_roundtrip() {
        let created =
            create_hwp_bytes_from_text_for_cli("사업 앵커 본문", None).expect("create hwp");
        let bookmarked = add_hwp_bookmark_bytes_for_cli(&created.bytes, 0, 0, 3, "biz_body")
            .expect("add bookmark");
        assert_eq!(bookmarked.details["operation"], "add-bookmark");
        assert_eq!(bookmarked.details["name"], "biz_body");

        let bookmarks = get_hwp_bookmarks_json_for_cli(&bookmarked.bytes).expect("list bookmarks");
        assert_eq!(bookmarks["ok"], true);
        assert_eq!(bookmarks["bookmarks"][0]["name"], "biz_body");
        assert_eq!(bookmarks["bookmarks"][0]["sec"], 0);
        assert_eq!(bookmarks["bookmarks"][0]["para"], 0);
        let ctrl_idx = bookmarks["bookmarks"][0]["ctrlIdx"]
            .as_u64()
            .expect("bookmark ctrl idx") as usize;

        let renamed =
            rename_hwp_bookmark_bytes_for_cli(&bookmarked.bytes, 0, 0, ctrl_idx, "biz_result")
                .expect("rename bookmark");
        assert_eq!(renamed.details["operation"], "rename-bookmark");
        let bookmarks = get_hwp_bookmarks_json_for_cli(&renamed.bytes).expect("list renamed");
        assert_eq!(bookmarks["bookmarks"][0]["name"], "biz_result");

        let deleted = delete_hwp_bookmark_bytes_for_cli(&renamed.bytes, 0, 0, ctrl_idx)
            .expect("delete bookmark");
        assert_eq!(deleted.details["operation"], "delete-bookmark");
        let bookmarks = get_hwp_bookmarks_json_for_cli(&deleted.bytes).expect("list deleted");
        assert_eq!(
            bookmarks["bookmarks"].as_array().expect("bookmarks").len(),
            0
        );

        rhwp::document_core::DocumentCore::from_bytes(&deleted.bytes)
            .expect("reload bookmark-edited hwp");
    }

    #[test]
    fn footnote_and_endnote_edits_roundtrip() {
        let created =
            create_hwp_bytes_from_text_for_cli("본문각주테스트", None).expect("create hwp");
        let footnote =
            create_hwp_note_bytes_for_cli(&created.bytes, 0, 0, 2, false, Some("초기 각주"))
                .expect("create footnote");
        assert_eq!(footnote.details["operation"], "create-footnote");
        assert_eq!(footnote.details["kind"], "footnote");
        let control_idx = footnote.details["controlIdx"]
            .as_u64()
            .expect("footnote control idx") as usize;

        let info = get_hwp_footnote_info_json_for_cli(&footnote.bytes, 0, 0, control_idx)
            .expect("get footnote info");
        assert_eq!(info["ok"], true);
        assert!(info["texts"][0]
            .as_str()
            .expect("footnote text")
            .contains("초기 각주"));

        let inserted = insert_hwp_footnote_text_bytes_for_cli(
            &footnote.bytes,
            0,
            0,
            control_idx,
            0,
            2,
            "수정 ",
        )
        .expect("insert footnote text");
        assert_eq!(inserted.details["operation"], "insert-footnote-text");
        let info = get_hwp_footnote_info_json_for_cli(&inserted.bytes, 0, 0, control_idx)
            .expect("get edited footnote info");
        assert!(info["texts"][0]
            .as_str()
            .expect("edited footnote text")
            .contains("수정 초기 각주"));

        let deleted =
            delete_hwp_footnote_text_bytes_for_cli(&inserted.bytes, 0, 0, control_idx, 0, 2, 3)
                .expect("delete footnote text");
        assert_eq!(deleted.details["operation"], "delete-footnote-text");
        let info = get_hwp_footnote_info_json_for_cli(&deleted.bytes, 0, 0, control_idx)
            .expect("get trimmed footnote info");
        assert!(!info["texts"][0]
            .as_str()
            .expect("trimmed footnote text")
            .contains("수정"));

        let split =
            split_hwp_footnote_paragraph_bytes_for_cli(&deleted.bytes, 0, 0, control_idx, 0, 5)
                .expect("split footnote paragraph");
        assert_eq!(split.details["operation"], "split-footnote-paragraph");
        let info = get_hwp_footnote_info_json_for_cli(&split.bytes, 0, 0, control_idx)
            .expect("get split footnote info");
        assert_eq!(info["paraCount"], 2);

        let merged = merge_hwp_footnote_paragraph_bytes_for_cli(&split.bytes, 0, 0, control_idx, 1)
            .expect("merge footnote paragraph");
        assert_eq!(merged.details["operation"], "merge-footnote-paragraph");
        let info = get_hwp_footnote_info_json_for_cli(&merged.bytes, 0, 0, control_idx)
            .expect("get merged footnote info");
        assert_eq!(info["paraCount"], 1);

        let removed = delete_hwp_footnote_bytes_for_cli(&merged.bytes, 0, 0, control_idx)
            .expect("delete footnote");
        assert_eq!(removed.details["operation"], "delete-footnote");
        let core =
            rhwp::document_core::DocumentCore::from_bytes(&removed.bytes).expect("reload removed");
        assert!(!core.document().sections[0].paragraphs[0]
            .controls
            .iter()
            .any(|ctrl| matches!(ctrl, rhwp::model::control::Control::Footnote(_))));

        let endnote =
            create_hwp_note_bytes_for_cli(&removed.bytes, 0, 0, 2, true, Some("미주 내용"))
                .expect("create endnote");
        assert_eq!(endnote.details["operation"], "create-endnote");
        assert_eq!(endnote.details["kind"], "endnote");
        let endnote_control_idx = endnote.details["controlIdx"]
            .as_u64()
            .expect("endnote control idx") as usize;
        let info = get_hwp_footnote_info_json_for_cli(&endnote.bytes, 0, 0, endnote_control_idx)
            .expect("get endnote info");
        assert!(info["texts"][0]
            .as_str()
            .expect("endnote text")
            .contains("미주 내용"));

        let structure =
            extract_hwp_structure_json_for_cli(&endnote.bytes).expect("extract note structure");
        let controls = structure["sections"][0]["paragraphs"][0]["controls"]
            .as_array()
            .expect("paragraph controls");
        let endnote_control = controls
            .iter()
            .find(|control| control["kind"] == "endnote")
            .expect("endnote control in structure");
        assert_eq!(endnote_control["controlIndex"], endnote_control_idx);
        assert!(endnote_control["texts"][0]
            .as_str()
            .expect("endnote structure text")
            .contains("미주 내용"));

        rhwp::document_core::DocumentCore::from_bytes(&endnote.bytes)
            .expect("reload note-edited hwp");
    }

    #[test]
    fn table_structure_edits_roundtrip() {
        let created = create_hwp_bytes_from_text_for_cli("사업 표", None).expect("create hwp");
        let table =
            create_hwp_table_bytes_for_cli(&created.bytes, 0, 0, 4, 2, 2).expect("create table");

        let edited =
            insert_hwp_table_row_bytes_for_cli(&table.bytes, 0, table.para_idx, 0, 0, true)
                .expect("insert row");
        let structure = extract_hwp_structure_json_for_cli(&edited.bytes).expect("extract row");
        assert_eq!(structure["sections"][0]["tables"][0]["rowCount"], 3);

        let edited = delete_hwp_table_row_bytes_for_cli(&edited.bytes, 0, table.para_idx, 0, 1)
            .expect("delete row");
        let structure = extract_hwp_structure_json_for_cli(&edited.bytes).expect("extract row");
        assert_eq!(structure["sections"][0]["tables"][0]["rowCount"], 2);

        let edited =
            insert_hwp_table_column_bytes_for_cli(&edited.bytes, 0, table.para_idx, 0, 0, true)
                .expect("insert column");
        let structure = extract_hwp_structure_json_for_cli(&edited.bytes).expect("extract col");
        assert_eq!(structure["sections"][0]["tables"][0]["colCount"], 3);

        let edited = delete_hwp_table_column_bytes_for_cli(&edited.bytes, 0, table.para_idx, 0, 1)
            .expect("delete column");
        let structure = extract_hwp_structure_json_for_cli(&edited.bytes).expect("extract col");
        assert_eq!(structure["sections"][0]["tables"][0]["colCount"], 2);

        let merged =
            merge_hwp_table_cells_bytes_for_cli(&edited.bytes, 0, table.para_idx, 0, 0, 0, 0, 1)
                .expect("merge cells");
        let structure = extract_hwp_structure_json_for_cli(&merged.bytes).expect("extract merge");
        assert_eq!(
            structure["sections"][0]["tables"][0]["cells"][0]["colSpan"],
            2
        );

        let split = split_hwp_table_cell_bytes_for_cli(&merged.bytes, 0, table.para_idx, 0, 0, 0)
            .expect("split cell");
        let structure = extract_hwp_structure_json_for_cli(&split.bytes).expect("extract split");
        assert_eq!(structure["sections"][0]["tables"][0]["cellCount"], 4);
        assert_eq!(
            structure["sections"][0]["tables"][0]["cells"][0]["colSpan"],
            1
        );

        rhwp::document_core::DocumentCore::from_bytes(&split.bytes)
            .expect("reload structure-edited hwp");
    }

    #[test]
    fn table_row_copy_roundtrip_preserves_cell_text() {
        let created = create_hwp_bytes_from_text_for_cli("보고 표", None).expect("create hwp");
        let table =
            create_hwp_table_bytes_for_cli(&created.bytes, 0, 0, 4, 2, 2).expect("create table");
        let texted = set_hwp_cell_text_by_position_bytes_for_cli(
            &table.bytes,
            table.para_idx,
            table.control_idx,
            1,
            0,
            0,
            "착수보고",
        )
        .expect("set first source cell");
        let texted = set_hwp_cell_text_by_position_bytes_for_cli(
            &texted.bytes,
            table.para_idx,
            table.control_idx,
            1,
            1,
            0,
            "사업수행계획서",
        )
        .expect("set second source cell");

        let copied = copy_hwp_table_row_bytes_for_cli(&texted.bytes, 0, table.para_idx, 0, 1, true)
            .expect("copy row below");
        assert_eq!(copied.details["operation"], "copy-table-row");
        assert_eq!(copied.details["sourceRow"], 1);
        assert_eq!(copied.details["targetRow"], 2);
        assert_eq!(copied.details["rowCount"], 3);

        let structure = extract_hwp_structure_json_for_cli(&copied.bytes).expect("extract copy");
        let table_json = &structure["sections"][0]["tables"][0];
        assert_eq!(table_json["rowCount"], 3);
        let cells = table_json["cells"].as_array().expect("cells");
        let cell_text = |row: u64, col: u64| -> &str {
            cells
                .iter()
                .find(|cell| cell["row"] == row && cell["col"] == col)
                .and_then(|cell| cell["text"].as_str())
                .expect("copied cell text")
        };
        assert_eq!(cell_text(2, 0), "착수보고");
        assert_eq!(cell_text(2, 1), "사업수행계획서");
    }

    #[test]
    fn table_row_copy_can_replace_only_the_copied_row() {
        let created = create_hwp_bytes_from_text_for_cli("LCA 항목 표", None).expect("create hwp");
        let table =
            create_hwp_table_bytes_for_cli(&created.bytes, 0, 0, 4, 2, 2).expect("create table");
        let texted = set_hwp_cell_text_by_position_bytes_for_cli(
            &table.bytes,
            table.para_idx,
            table.control_idx,
            1,
            0,
            0,
            "{{항목}}",
        )
        .expect("set item placeholder");
        let texted = set_hwp_cell_text_by_position_bytes_for_cli(
            &texted.bytes,
            table.para_idx,
            table.control_idx,
            1,
            1,
            0,
            "{{값}}",
        )
        .expect("set value placeholder");

        let copied = copy_hwp_table_row_with_replacements_bytes_for_cli(
            &texted.bytes,
            0,
            table.para_idx,
            0,
            1,
            true,
            &[
                ("{{항목}}".to_string(), "원재료 투입".to_string()),
                ("{{값}}".to_string(), "12.5 kg".to_string()),
            ],
        )
        .expect("copy row with replacements");
        assert_eq!(copied.details["operation"], "copy-table-row");
        assert_eq!(copied.details["replacementCount"], 2);
        assert_eq!(copied.details["sourceRow"], 1);
        assert_eq!(copied.details["targetRow"], 2);

        let structure = extract_hwp_structure_json_for_cli(&copied.bytes).expect("extract copy");
        let table_json = &structure["sections"][0]["tables"][0];
        assert_eq!(table_json["rowCount"], 3);
        let cells = table_json["cells"].as_array().expect("cells");
        let cell_text = |row: u64, col: u64| -> &str {
            cells
                .iter()
                .find(|cell| cell["row"] == row && cell["col"] == col)
                .and_then(|cell| cell["text"].as_str())
                .expect("cell text")
        };
        assert_eq!(cell_text(1, 0), "{{항목}}");
        assert_eq!(cell_text(1, 1), "{{값}}");
        assert_eq!(cell_text(2, 0), "원재료 투입");
        assert_eq!(cell_text(2, 1), "12.5 kg");

        rhwp::document_core::DocumentCore::from_bytes(&copied.bytes)
            .expect("reload table-row-copy-replaced hwp");
    }

    #[test]
    fn table_column_copy_roundtrip_preserves_cell_text() {
        let created = create_hwp_bytes_from_text_for_cli("일정 표", None).expect("create hwp");
        let table =
            create_hwp_table_bytes_for_cli(&created.bytes, 0, 0, 4, 2, 2).expect("create table");
        let texted = set_hwp_cell_text_by_position_bytes_for_cli(
            &table.bytes,
            table.para_idx,
            table.control_idx,
            0,
            1,
            0,
            "시기",
        )
        .expect("set first source cell");
        let texted = set_hwp_cell_text_by_position_bytes_for_cli(
            &texted.bytes,
            table.para_idx,
            table.control_idx,
            1,
            1,
            0,
            "착수보고시",
        )
        .expect("set second source cell");

        let copied =
            copy_hwp_table_column_bytes_for_cli(&texted.bytes, 0, table.para_idx, 0, 1, true)
                .expect("copy column right");
        assert_eq!(copied.details["operation"], "copy-table-column");
        assert_eq!(copied.details["sourceCol"], 1);
        assert_eq!(copied.details["targetCol"], 2);
        assert_eq!(copied.details["colCount"], 3);

        let structure = extract_hwp_structure_json_for_cli(&copied.bytes).expect("extract copy");
        let table_json = &structure["sections"][0]["tables"][0];
        assert_eq!(table_json["colCount"], 3);
        let cells = table_json["cells"].as_array().expect("cells");
        let cell_text = |row: u64, col: u64| -> &str {
            cells
                .iter()
                .find(|cell| cell["row"] == row && cell["col"] == col)
                .and_then(|cell| cell["text"].as_str())
                .expect("copied cell text")
        };
        assert_eq!(cell_text(0, 2), "시기");
        assert_eq!(cell_text(1, 2), "착수보고시");
    }

    #[test]
    fn table_column_copy_can_replace_only_the_copied_column() {
        let created = create_hwp_bytes_from_text_for_cli("월별 표", None).expect("create hwp");
        let table =
            create_hwp_table_bytes_for_cli(&created.bytes, 0, 0, 4, 2, 2).expect("create table");
        let texted = set_hwp_cell_text_by_position_bytes_for_cli(
            &table.bytes,
            table.para_idx,
            table.control_idx,
            0,
            1,
            0,
            "{{월}}",
        )
        .expect("set month placeholder");
        let texted = set_hwp_cell_text_by_position_bytes_for_cli(
            &texted.bytes,
            table.para_idx,
            table.control_idx,
            1,
            1,
            0,
            "{{배출량}}",
        )
        .expect("set emission placeholder");

        let copied = copy_hwp_table_column_with_replacements_bytes_for_cli(
            &texted.bytes,
            0,
            table.para_idx,
            0,
            1,
            true,
            &[
                ("{{월}}".to_string(), "6월".to_string()),
                ("{{배출량}}".to_string(), "42.0 kgCO2e".to_string()),
            ],
        )
        .expect("copy column with replacements");
        assert_eq!(copied.details["operation"], "copy-table-column");
        assert_eq!(copied.details["replacementCount"], 2);
        assert_eq!(copied.details["sourceCol"], 1);
        assert_eq!(copied.details["targetCol"], 2);

        let structure = extract_hwp_structure_json_for_cli(&copied.bytes).expect("extract copy");
        let table_json = &structure["sections"][0]["tables"][0];
        assert_eq!(table_json["colCount"], 3);
        let cells = table_json["cells"].as_array().expect("cells");
        let cell_text = |row: u64, col: u64| -> &str {
            cells
                .iter()
                .find(|cell| cell["row"] == row && cell["col"] == col)
                .and_then(|cell| cell["text"].as_str())
                .expect("cell text")
        };
        assert_eq!(cell_text(0, 1), "{{월}}");
        assert_eq!(cell_text(1, 1), "{{배출량}}");
        assert_eq!(cell_text(0, 2), "6월");
        assert_eq!(cell_text(1, 2), "42.0 kgCO2e");

        rhwp::document_core::DocumentCore::from_bytes(&copied.bytes)
            .expect("reload table-column-copy-replaced hwp");
    }

    #[test]
    fn table_copy_roundtrip_preserves_text_and_clears_duplicate_fields() {
        let created = create_hwp_bytes_from_text_for_cli("반복 표", None).expect("create hwp");
        let table =
            create_hwp_table_bytes_for_cli(&created.bytes, 0, 0, 4, 2, 2).expect("create table");
        let texted = set_hwp_cell_text_by_position_bytes_for_cli(
            &table.bytes,
            table.para_idx,
            table.control_idx,
            0,
            0,
            0,
            "항목",
        )
        .expect("set header cell");
        let fielded = set_hwp_cell_field_by_position_bytes_for_cli(
            &texted.bytes,
            table.para_idx,
            table.control_idx,
            1,
            1,
            Some("biz_amount"),
        )
        .expect("set source field");
        let filled = set_hwp_field_bytes_for_cli(&fielded.bytes, "biz_amount", "1,200만원")
            .expect("fill source field");

        let copied = copy_hwp_table_bytes_for_cli(&filled.bytes, 0, table.para_idx, 0, true)
            .expect("copy table after source");
        assert_eq!(copied.details["operation"], "copy-table");
        assert_eq!(copied.details["sourceParaIdx"], table.para_idx);
        assert_eq!(copied.details["targetParaIdx"], table.para_idx + 1);
        assert_eq!(copied.details["controlIdx"], 0);

        let structure = extract_hwp_structure_json_for_cli(&copied.bytes).expect("extract copy");
        assert_eq!(structure["sections"][0]["tableCount"], 2);
        let tables = structure["sections"][0]["tables"]
            .as_array()
            .expect("tables");
        let copied_table = tables
            .iter()
            .find(|t| t["paragraphIndex"] == table.para_idx + 1)
            .expect("copied table");
        assert_eq!(copied_table["rowCount"], 2);
        assert_eq!(copied_table["colCount"], 2);
        let cells = copied_table["cells"].as_array().expect("cells");
        let cell_text = |row: u64, col: u64| -> &str {
            cells
                .iter()
                .find(|cell| cell["row"] == row && cell["col"] == col)
                .and_then(|cell| cell["text"].as_str())
                .expect("copied cell text")
        };
        assert_eq!(cell_text(0, 0), "항목");
        assert_eq!(cell_text(1, 1), "1,200만원");

        let fields = list_hwp_fields_json_for_cli(&copied.bytes).expect("list copied fields");
        assert_eq!(fields["count"], 1);
        assert_eq!(fields["fields"][0]["name"], "biz_amount");
    }

    #[test]
    fn table_copy_can_replace_only_the_copied_table() {
        let created = create_hwp_bytes_from_text_for_cli("반복 표", None).expect("create hwp");
        let table =
            create_hwp_table_bytes_for_cli(&created.bytes, 0, 0, 4, 2, 2).expect("create table");
        let named = set_hwp_cell_text_by_position_bytes_for_cli(
            &table.bytes,
            table.para_idx,
            table.control_idx,
            0,
            0,
            0,
            "회사",
        )
        .expect("set label");
        let amounted = set_hwp_cell_text_by_position_bytes_for_cli(
            &named.bytes,
            table.para_idx,
            table.control_idx,
            0,
            1,
            0,
            "{{회사}}",
        )
        .expect("set company placeholder");
        let templated = set_hwp_cell_text_by_position_bytes_for_cli(
            &amounted.bytes,
            table.para_idx,
            table.control_idx,
            1,
            1,
            0,
            "{{금액}}",
        )
        .expect("set amount placeholder");

        let copied = copy_hwp_table_with_replacements_bytes_for_cli(
            &templated.bytes,
            0,
            table.para_idx,
            0,
            true,
            &[
                ("{{회사}}".to_string(), "평화오일씰공업".to_string()),
                ("{{금액}}".to_string(), "1,200만원".to_string()),
            ],
        )
        .expect("copy table with replacements");
        assert_eq!(copied.details["operation"], "copy-table");
        assert_eq!(copied.details["replacementCount"], 2);

        let structure = extract_hwp_structure_json_for_cli(&copied.bytes).expect("extract copy");
        let tables = structure["sections"][0]["tables"]
            .as_array()
            .expect("tables");
        let source_table = tables
            .iter()
            .find(|t| t["paragraphIndex"] == table.para_idx)
            .expect("source table");
        let copied_table = tables
            .iter()
            .find(|t| t["paragraphIndex"] == table.para_idx + 1)
            .expect("copied table");
        let cell_text = |table: &serde_json::Value, row: u64, col: u64| -> String {
            table["cells"]
                .as_array()
                .expect("cells")
                .iter()
                .find(|cell| cell["row"] == row && cell["col"] == col)
                .and_then(|cell| cell["text"].as_str())
                .expect("cell text")
                .to_string()
        };
        assert_eq!(cell_text(source_table, 0, 1), "{{회사}}");
        assert_eq!(cell_text(source_table, 1, 1), "{{금액}}");
        assert_eq!(cell_text(copied_table, 0, 1), "평화오일씰공업");
        assert_eq!(cell_text(copied_table, 1, 1), "1,200만원");

        rhwp::document_core::DocumentCore::from_bytes(&copied.bytes)
            .expect("reload table-copy-replaced hwp");
    }

    #[test]
    fn extract_structure_reports_body_textbox_text() {
        let created = create_hwp_bytes_from_text_for_cli("사업 양식", None).expect("create hwp");
        let shape = create_hwp_shape_bytes_for_cli(
            &created.bytes,
            0,
            0,
            0,
            5000,
            1800,
            0,
            0,
            true,
            "Square",
            "textbox",
            false,
            false,
            "[]",
        )
        .expect("create textbox shape");
        let texted = set_hwp_cell_text_bytes_for_cli(
            &shape.bytes,
            shape.para_idx,
            shape.control_idx,
            0,
            0,
            "목표: {{목표}}",
        )
        .expect("set textbox text");

        let structure = extract_hwp_structure_json_for_cli(&texted.bytes).expect("extract");
        let shapes = structure["sections"][0]["shapes"]
            .as_array()
            .expect("shapes array");
        let textbox = shapes
            .iter()
            .find(|item| item["paragraphIndex"] == shape.para_idx)
            .expect("body textbox");
        assert_eq!(textbox["controlIndex"], shape.control_idx);
        assert_eq!(textbox["shapeType"], "TextBox");
        assert_eq!(textbox["textBox"]["paragraphCount"], 1);
        assert_eq!(
            textbox["textBox"]["paragraphs"][0]["text"],
            "목표: {{목표}}"
        );
        assert_eq!(textbox["textBox"]["text"], "목표: {{목표}}");
    }

    #[test]
    fn extract_structure_reports_cell_textbox_text() {
        let created = create_hwp_bytes_from_text_for_cli("사업 양식", None).expect("create hwp");
        let table =
            create_hwp_table_bytes_for_cli(&created.bytes, 0, 0, 0, 2, 2).expect("create table");
        let cell_path = serde_json::json!([
            {"controlIndex": table.control_idx, "cellIndex": 3, "cellParaIndex": 0}
        ])
        .to_string();
        let shape = create_hwp_cell_shape_bytes_for_cli(
            &table.bytes,
            0,
            table.para_idx,
            0,
            &cell_path,
            5000,
            3000,
            1000,
            2000,
            true,
            "InFrontOfText",
            "textbox",
            false,
            false,
            "[]",
        )
        .expect("create cell textbox shape");
        let texted = set_hwp_cell_shape_text_bytes_for_cli(
            &shape.bytes,
            0,
            table.para_idx,
            &cell_path,
            shape.control_idx,
            0,
            "목표: {{목표}}",
        )
        .expect("set cell textbox text");

        let structure = extract_hwp_structure_json_for_cli(&texted.bytes).expect("extract");
        let cells = structure["sections"][0]["tables"][0]["cells"]
            .as_array()
            .expect("cells array");
        let cell = cells
            .iter()
            .find(|item| item["index"] == 3)
            .expect("cell 3");
        let shapes = cell["shapes"].as_array().expect("cell shapes array");
        assert_eq!(shapes.len(), 1);
        let textbox = &shapes[0];
        assert_eq!(textbox["container"], "cell");
        assert_eq!(textbox["tableControl"], table.control_idx);
        assert_eq!(textbox["cellIndex"], 3);
        assert_eq!(textbox["shapeControl"], shape.control_idx);
        assert_eq!(textbox["textBox"]["paragraphCount"], 1);
        assert_eq!(
            textbox["textBox"]["paragraphs"][0]["text"],
            "목표: {{목표}}"
        );
        assert_eq!(textbox["textBox"]["text"], "목표: {{목표}}");
    }

    #[test]
    fn table_delete_roundtrip_removes_table_block_and_following_empty_paragraph() {
        let created = create_hwp_bytes_from_text_for_cli("표 삭제 대상", None).expect("create hwp");
        let table =
            create_hwp_table_bytes_for_cli(&created.bytes, 0, 0, 0, 2, 2).expect("create table");
        let texted = set_hwp_cell_text_by_position_bytes_for_cli(
            &table.bytes,
            table.para_idx,
            table.control_idx,
            0,
            0,
            0,
            "삭제될 셀",
        )
        .expect("set cell text");

        let deleted = delete_hwp_table_bytes_for_cli(&texted.bytes, 0, table.para_idx, 0)
            .expect("delete table");
        assert_eq!(deleted.details["operation"], "delete-table");
        assert_eq!(deleted.details["deletedParaIdx"], table.para_idx);
        assert_eq!(deleted.details["removedFollowingEmptyParagraph"], true);

        let structure = extract_hwp_structure_json_for_cli(&deleted.bytes).expect("extract delete");
        assert_eq!(structure["sections"][0]["tableCount"], 0);
        assert_eq!(structure["sections"][0]["paragraphCount"], 1);
        assert_eq!(
            structure["sections"][0]["paragraphs"][0]["text"],
            "표 삭제 대상"
        );
    }

    #[test]
    fn table_property_edits_roundtrip() {
        let created = create_hwp_bytes_from_text_for_cli("서식 표", None).expect("create hwp");
        let table =
            create_hwp_table_bytes_for_cli(&created.bytes, 0, 0, 4, 2, 2).expect("create table");

        let edited = set_hwp_cell_properties_bytes_for_cli(
            &table.bytes,
            0,
            table.para_idx,
            0,
            0,
            r#"{"width":5000,"height":1800,"paddingLeft":120,"paddingRight":130,"paddingTop":140,"paddingBottom":150,"verticalAlign":1,"isHeader":true,"cellProtect":true}"#,
        )
        .expect("set cell properties");
        let props = get_hwp_cell_properties_json_for_cli(&edited.bytes, 0, table.para_idx, 0, 0)
            .expect("get cell properties");
        assert_eq!(props["width"], 5000);
        assert_eq!(props["height"], 1800);
        assert_eq!(props["paddingLeft"], 120);
        assert_eq!(props["verticalAlign"], 1);
        assert_eq!(props["isHeader"], true);
        assert_eq!(props["cellProtect"], true);

        let fill_edited = set_hwp_cell_properties_bytes_for_cli(
            &edited.bytes,
            0,
            table.para_idx,
            0,
            1,
            r##"{"fillType":"solid","fillColor":"#f5f5f5"}"##,
        )
        .expect("set cell fill properties");
        let props =
            get_hwp_cell_properties_json_for_cli(&fill_edited.bytes, 0, table.para_idx, 0, 1)
                .expect("get cell fill properties");
        assert_eq!(props["fillType"], "solid");
        assert_eq!(props["fillColor"], "#f5f5f5");

        let row_col_edited = set_hwp_cell_properties_at_bytes_for_cli(
            &edited.bytes,
            0,
            table.para_idx,
            0,
            1,
            1,
            r##"{"fillType":"solid","fillColor":"#fff2cc"}"##,
        )
        .expect("set cell fill properties by row col");
        let props = get_hwp_cell_properties_at_json_for_cli(
            &row_col_edited.bytes,
            0,
            table.para_idx,
            0,
            1,
            1,
        )
        .expect("get cell fill properties by row col");
        assert_eq!(props["fillType"], "solid");
        assert_eq!(props["fillColor"], "#fff2cc");

        let edited = set_hwp_table_properties_bytes_for_cli(
            &edited.bytes,
            0,
            table.para_idx,
            0,
            r#"{"cellSpacing":20,"paddingLeft":30,"paddingRight":40,"paddingTop":50,"paddingBottom":60,"repeatHeader":true,"treatAsChar":true,"horzAlign":"Center"}"#,
        )
        .expect("set table properties");
        let props = get_hwp_table_properties_json_for_cli(&edited.bytes, 0, table.para_idx, 0)
            .expect("get table properties");
        assert_eq!(props["cellSpacing"], 20);
        assert_eq!(props["paddingLeft"], 30);
        assert_eq!(props["repeatHeader"], true);
        assert_eq!(props["treatAsChar"], true);
        assert_eq!(props["horzAlign"], "Center");

        let resized = resize_hwp_table_cells_bytes_for_cli(
            &edited.bytes,
            0,
            table.para_idx,
            0,
            r#"[{"cellIdx":0,"widthDelta":250,"heightDelta":300}]"#,
        )
        .expect("resize table cells");
        let props = get_hwp_cell_properties_json_for_cli(&resized.bytes, 0, table.para_idx, 0, 0)
            .expect("get resized cell properties");
        assert_eq!(props["width"], 5250);
        assert_eq!(props["height"], 2100);

        rhwp::document_core::DocumentCore::from_bytes(&resized.bytes)
            .expect("reload property-edited hwp");
    }

    #[test]
    fn text_format_edits_roundtrip() {
        let created = create_hwp_bytes_from_text_for_cli("제목 본문", None).expect("create hwp");
        let edited = set_hwp_char_format_bytes_for_cli(
            &created.bytes,
            0,
            0,
            0,
            2,
            r##"{"bold":true,"fontSize":2400,"textColor":"#ff0000"}"##,
        )
        .expect("set char format");
        let props =
            get_hwp_char_properties_json_for_cli(&edited.bytes, 0, 0, 0).expect("get char props");
        assert_eq!(props["bold"], true);
        assert_eq!(props["fontSize"], 2400);
        assert_eq!(props["textColor"], "#ff0000");

        let edited = set_hwp_para_format_bytes_for_cli(
            &edited.bytes,
            0,
            0,
            r#"{"alignment":"center","lineSpacing":180,"lineSpacingType":"Percent"}"#,
        )
        .expect("set para format");
        let props =
            get_hwp_para_properties_json_for_cli(&edited.bytes, 0, 0).expect("get para props");
        assert_eq!(props["alignment"], "center");
        assert_eq!(props["lineSpacing"], 180.0);

        let table =
            create_hwp_table_bytes_for_cli(&edited.bytes, 0, 0, 5, 1, 1).expect("create table");
        let cell = set_hwp_cell_text_bytes_for_cli(&table.bytes, table.para_idx, 0, 0, 0, "금액")
            .expect("set cell text");
        let cell = set_hwp_cell_char_format_bytes_for_cli(
            &cell.bytes,
            0,
            table.para_idx,
            0,
            0,
            0,
            0,
            2,
            r##"{"bold":true,"textColor":"#0000ff"}"##,
        )
        .expect("set cell char format");
        let props =
            get_hwp_cell_char_properties_json_for_cli(&cell.bytes, 0, table.para_idx, 0, 0, 0, 0)
                .expect("get cell char props");
        assert_eq!(props["bold"], true);
        assert_eq!(props["textColor"], "#0000ff");

        let cell = set_hwp_cell_char_format_at_bytes_for_cli(
            &cell.bytes,
            0,
            table.para_idx,
            0,
            0,
            0,
            0,
            0,
            2,
            r##"{"italic":true,"textColor":"#008000"}"##,
        )
        .expect("set cell char format by row/col");
        let props = get_hwp_cell_char_properties_at_json_for_cli(
            &cell.bytes,
            0,
            table.para_idx,
            0,
            0,
            0,
            0,
            0,
        )
        .expect("get cell char props by row/col");
        assert_eq!(props["italic"], true);
        assert_eq!(props["textColor"], "#008000");
        assert_eq!(props["row"], 0);
        assert_eq!(props["col"], 0);
        assert_eq!(props["cellIndex"], 0);

        let cell = set_hwp_cell_para_format_bytes_for_cli(
            &cell.bytes,
            0,
            table.para_idx,
            0,
            0,
            0,
            r#"{"alignment":"center"}"#,
        )
        .expect("set cell para format");
        let props =
            get_hwp_cell_para_properties_json_for_cli(&cell.bytes, 0, table.para_idx, 0, 0, 0)
                .expect("get cell para props");
        assert_eq!(props["alignment"], "center");

        let cell = set_hwp_cell_para_format_at_bytes_for_cli(
            &cell.bytes,
            0,
            table.para_idx,
            0,
            0,
            0,
            0,
            r#"{"alignment":"right"}"#,
        )
        .expect("set cell para format by row/col");
        let props = get_hwp_cell_para_properties_at_json_for_cli(
            &cell.bytes,
            0,
            table.para_idx,
            0,
            0,
            0,
            0,
        )
        .expect("get cell para props by row/col");
        assert_eq!(props["alignment"], "right");
        assert_eq!(props["row"], 0);
        assert_eq!(props["col"], 0);
        assert_eq!(props["cellIndex"], 0);

        rhwp::document_core::DocumentCore::from_bytes(&cell.bytes)
            .expect("reload text-format-edited hwp");
    }

    #[test]
    fn style_listing_and_application_roundtrip() {
        let created = create_hwp_bytes_from_text_for_cli("제목\n본문", None).expect("create hwp");

        let styles = list_hwp_styles_json_for_cli(&created.bytes).expect("list styles");
        assert_eq!(styles["ok"], true);
        assert!(styles["count"].as_u64().unwrap_or(0) > 0);
        assert_eq!(styles["styles"][0]["id"], 0);

        let styled =
            apply_hwp_style_bytes_for_cli(&created.bytes, 0, 1, 0).expect("apply body style");
        assert_eq!(styled.details["operation"], "apply-style");
        assert_eq!(styled.details["styleId"], 0);

        let core =
            rhwp::document_core::DocumentCore::from_bytes(&styled.bytes).expect("reload styled");
        assert_eq!(core.document().sections[0].paragraphs[1].style_id, 0);
    }

    #[test]
    fn cell_style_application_roundtrip() {
        let created = create_hwp_bytes_from_text_for_cli("표", None).expect("create hwp");
        let table =
            create_hwp_table_bytes_for_cli(&created.bytes, 0, 0, 1, 1, 1).expect("create table");

        let styled = apply_hwp_cell_style_bytes_for_cli(
            &table.bytes,
            0,
            table.para_idx,
            table.control_idx,
            0,
            0,
            0,
        )
        .expect("apply cell style");
        assert_eq!(styled.details["operation"], "apply-cell-style");
        assert_eq!(styled.details["styleId"], 0);

        let styled = apply_hwp_cell_style_at_bytes_for_cli(
            &styled.bytes,
            0,
            table.para_idx,
            table.control_idx,
            0,
            0,
            0,
            0,
        )
        .expect("apply cell style by row/col");
        assert_eq!(styled.details["operation"], "apply-cell-style");
        assert_eq!(styled.details["styleId"], 0);
        assert_eq!(styled.details["row"], 0);
        assert_eq!(styled.details["col"], 0);
        assert_eq!(styled.details["cellIndex"], 0);

        let core =
            rhwp::document_core::DocumentCore::from_bytes(&styled.bytes).expect("reload styled");
        let para = &core.document().sections[0].paragraphs[table.para_idx];
        let table = match &para.controls[table.control_idx] {
            rhwp::model::control::Control::Table(t) => t,
            _ => panic!("expected table"),
        };
        assert_eq!(table.cells[0].paragraphs[0].style_id, 0);
    }

    #[test]
    fn page_and_section_settings_roundtrip() {
        let created = create_hwp_bytes_from_text_for_cli("사업 양식", None).expect("create hwp");

        let page = set_hwp_page_def_bytes_for_cli(
            &created.bytes,
            0,
            r#"{"width":59528,"height":84188,"marginLeft":4000,"marginRight":4000,"marginTop":5000,"marginBottom":5000,"marginHeader":2000,"marginFooter":2000,"landscape":false,"binding":1}"#,
        )
        .expect("set page def");
        let props = get_hwp_page_def_json_for_cli(&page.bytes, 0).expect("get page def");
        assert_eq!(props["width"], 59528);
        assert_eq!(props["marginLeft"], 4000);
        assert_eq!(props["binding"], 1);

        let section = set_hwp_section_def_bytes_for_cli(
            &page.bytes,
            0,
            r#"{"pageNum":3,"tableNum":2,"hideHeader":true,"hideFooter":true,"hideEmptyLine":true}"#,
        )
        .expect("set section def");
        let props = get_hwp_section_def_json_for_cli(&section.bytes, 0).expect("get section def");
        assert_eq!(props["pageNum"], 3);
        assert_eq!(props["tableNum"], 2);
        assert_eq!(props["hideHeader"], true);
        assert_eq!(props["hideEmptyLine"], true);

        let border = set_hwp_page_border_fill_bytes_for_cli(
            &section.bytes,
            0,
            r##"{"spacingLeft":100,"spacingRight":110,"spacingTop":120,"spacingBottom":130,"basis":"paper","fillArea":"paper","borderLeft":{"type":1,"width":1,"color":"#222222"},"borderRight":{"type":1,"width":1,"color":"#222222"},"borderTop":{"type":1,"width":1,"color":"#222222"},"borderBottom":{"type":1,"width":1,"color":"#222222"},"fillType":"solid","fillColor":"#f5f5f5","patternColor":"#000000","patternType":0}"##,
        )
        .expect("set page border fill");
        let props =
            get_hwp_page_border_fill_json_for_cli(&border.bytes, 0).expect("get border fill");
        assert_eq!(props["spacingLeft"], 100);
        assert_eq!(props["spacingBottom"], 130);
        assert_eq!(props["fillType"], "solid");
        assert_eq!(props["fillColor"], "#f5f5f5");

        rhwp::document_core::DocumentCore::from_bytes(&border.bytes)
            .expect("reload page-section-edited hwp");
    }

    #[test]
    fn picture_and_shape_object_edits_roundtrip() {
        let created = create_hwp_bytes_from_text_for_cli("사업 양식", None).expect("create hwp");
        let png_bytes = [
            0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1a, b'\n', 0, 0, 0, 0,
        ];

        let picture = insert_hwp_picture_bytes_for_cli(
            &created.bytes,
            0,
            0,
            0,
            "[]",
            &png_bytes,
            2400,
            1200,
            1,
            1,
            "png",
            "사업 로고",
            Some(1500),
            Some(2500),
        )
        .expect("insert picture");
        let props = get_hwp_picture_properties_json_for_cli(
            &picture.bytes,
            0,
            picture.para_idx,
            picture.control_idx,
        )
        .expect("get picture props");
        assert_eq!(props["width"], 2400);
        assert_eq!(props["height"], 1200);
        assert_eq!(props["description"], "사업 로고");
        let picture_para_idx = picture.para_idx;
        let picture_control_idx = picture.control_idx;

        let picture = set_hwp_picture_properties_bytes_for_cli(
            &picture.bytes,
            0,
            picture_para_idx,
            picture_control_idx,
            r#"{"width":3600,"height":1800,"horzOffset":2200,"vertOffset":3300,"brightness":15,"textWrap":"InFrontOfText"}"#,
        )
        .expect("set picture props");
        let props = get_hwp_picture_properties_json_for_cli(
            &picture.bytes,
            0,
            picture_para_idx,
            picture_control_idx,
        )
        .expect("get changed picture props");
        assert_eq!(props["width"], 3600);
        assert_eq!(props["height"], 1800);
        assert_eq!(props["brightness"], 15);
        assert_eq!(props["textWrap"], "InFrontOfText");

        let picture_deleted = delete_hwp_picture_bytes_for_cli(
            &picture.bytes,
            0,
            picture_para_idx,
            picture_control_idx,
        )
        .expect("delete picture");
        rhwp::document_core::DocumentCore::from_bytes(&picture_deleted.bytes)
            .expect("reload picture-deleted hwp");

        let shape = create_hwp_shape_bytes_for_cli(
            &created.bytes,
            0,
            0,
            0,
            5000,
            3000,
            1000,
            2000,
            false,
            "InFrontOfText",
            "rectangle",
            false,
            false,
            "[]",
        )
        .expect("create shape");
        let props = get_hwp_shape_properties_json_for_cli(
            &shape.bytes,
            0,
            shape.para_idx,
            shape.control_idx,
        )
        .expect("get shape props");
        assert_eq!(props["width"], 5000);
        assert_eq!(props["height"], 3000);
        assert_eq!(props["textWrap"], "InFrontOfText");
        let shape_para_idx = shape.para_idx;
        let shape_control_idx = shape.control_idx;

        let shape = set_hwp_shape_properties_bytes_for_cli(
            &shape.bytes,
            0,
            shape_para_idx,
            shape_control_idx,
            r#"{"width":6200,"height":3400,"horzOffset":1400,"vertOffset":2400,"fillType":"solid","fillBgColor":16776960,"roundRate":20}"#,
        )
        .expect("set shape props");
        let props = get_hwp_shape_properties_json_for_cli(
            &shape.bytes,
            0,
            shape_para_idx,
            shape_control_idx,
        )
        .expect("get shape props");
        assert_eq!(props["width"], 6200);
        assert_eq!(props["height"], 3400);
        assert_eq!(props["fillType"], "solid");
        assert_eq!(props["fillBgColor"], 16776960);
        assert_eq!(props["roundRate"], 20);

        let shape_deleted =
            delete_hwp_shape_bytes_for_cli(&shape.bytes, 0, shape_para_idx, shape_control_idx)
                .expect("delete shape");
        rhwp::document_core::DocumentCore::from_bytes(&shape_deleted.bytes)
            .expect("reload shape-deleted hwp");
    }

    #[test]
    fn object_list_finds_pictures_and_shapes() {
        let created = create_hwp_bytes_from_text_for_cli("사업 양식", None).expect("create hwp");
        let png_bytes = [
            0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1a, b'\n', 0, 0, 0, 0,
        ];
        let picture = insert_hwp_picture_bytes_for_cli(
            &created.bytes,
            0,
            0,
            0,
            "[]",
            &png_bytes,
            2400,
            1200,
            1,
            1,
            "png",
            "사업 로고",
            Some(1500),
            Some(2500),
        )
        .expect("insert picture");
        let shape = create_hwp_shape_bytes_for_cli(
            &picture.bytes,
            0,
            0,
            2,
            5000,
            3000,
            1000,
            2000,
            true,
            "InFrontOfText",
            "textbox",
            false,
            false,
            "[]",
        )
        .expect("create textbox shape");

        let list = list_hwp_objects_json_for_cli(&shape.bytes).expect("list objects");
        assert_eq!(list["ok"], true);
        assert_eq!(list["count"], 2);
        let objects = list["objects"].as_array().expect("objects array");
        let picture_item = objects
            .iter()
            .find(|object| object["kind"] == "picture")
            .expect("picture object");
        assert_eq!(picture_item["container"], "body");
        assert_eq!(picture_item["section"], 0);
        assert_eq!(picture_item["paragraph"], picture.para_idx);
        assert_eq!(picture_item["width"], 2400);
        assert_eq!(picture_item["height"], 1200);
        assert_eq!(picture_item["description"], "사업 로고");
        let listed_picture_control = picture_item["control"]
            .as_u64()
            .expect("listed picture control") as usize;
        let listed_picture_props = get_hwp_picture_properties_json_for_cli(
            &shape.bytes,
            0,
            picture.para_idx,
            listed_picture_control,
        )
        .expect("listed picture location is editable");
        assert_eq!(listed_picture_props["description"], "사업 로고");

        let shape_item = objects
            .iter()
            .find(|object| object["kind"] == "shape")
            .expect("shape object");
        assert_eq!(shape_item["container"], "body");
        assert_eq!(shape_item["section"], 0);
        assert_eq!(shape_item["paragraph"], shape.para_idx);
        assert_eq!(shape_item["control"], shape.control_idx);
        assert_eq!(shape_item["shapeType"], "TextBox");
        assert_eq!(shape_item["width"], 5000);
        assert_eq!(shape_item["height"], 3000);
    }

    #[test]
    fn cell_picture_object_edits_roundtrip_by_path() {
        let created = create_hwp_bytes_from_text_for_cli("사업 양식", None).expect("create hwp");
        let table =
            create_hwp_table_bytes_for_cli(&created.bytes, 0, 0, 0, 2, 2).expect("create table");
        let cell_path = serde_json::json!([
            {"controlIndex": table.control_idx, "cellIndex": 3, "cellParaIndex": 0}
        ])
        .to_string();
        let png_bytes = [
            0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1a, b'\n', 0, 0, 0, 0,
        ];

        let picture = insert_hwp_picture_bytes_for_cli(
            &table.bytes,
            0,
            table.para_idx,
            0,
            &cell_path,
            &png_bytes,
            1400,
            900,
            1,
            1,
            "png",
            "회사 도장",
            None,
            None,
        )
        .expect("insert cell picture");
        assert_eq!(picture.details["operation"], "insert-picture");

        let props = get_hwp_cell_picture_properties_json_for_cli(
            &picture.bytes,
            0,
            table.para_idx,
            &cell_path,
            picture.control_idx,
        )
        .expect("get cell picture props");
        assert_eq!(props["ok"], true);
        assert_eq!(props["container"], "cell");
        assert_eq!(props["description"], "회사 도장");

        let resized = set_hwp_cell_picture_properties_bytes_for_cli(
            &picture.bytes,
            0,
            table.para_idx,
            &cell_path,
            picture.control_idx,
            r#"{"width":1800,"height":1100,"description":"수정 도장"}"#,
        )
        .expect("set cell picture props");
        assert_eq!(resized.details["operation"], "set-picture-properties");
        assert_eq!(resized.details["container"], "cell");

        let props = get_hwp_cell_picture_properties_json_for_cli(
            &resized.bytes,
            0,
            table.para_idx,
            &cell_path,
            picture.control_idx,
        )
        .expect("get resized cell picture props");
        assert_eq!(props["width"], 1800);
        assert_eq!(props["height"], 1100);
        assert_eq!(props["description"], "수정 도장");

        let deleted = delete_hwp_cell_picture_bytes_for_cli(
            &resized.bytes,
            0,
            table.para_idx,
            &cell_path,
            picture.control_idx,
        )
        .expect("delete cell picture");
        assert_eq!(deleted.details["operation"], "delete-picture");
        assert_eq!(deleted.details["container"], "cell");
        rhwp::document_core::DocumentCore::from_bytes(&deleted.bytes)
            .expect("reload cell-picture-deleted hwp");
    }

    fn hwp_bytes_with_cell_textbox_shape() -> (Vec<u8>, usize, usize, String, usize) {
        let created = create_hwp_bytes_from_text_for_cli("사업 양식", None).expect("create hwp");
        let table =
            create_hwp_table_bytes_for_cli(&created.bytes, 0, 0, 0, 2, 2).expect("create table");
        let cell_path = serde_json::json!([
            {"controlIndex": table.control_idx, "cellIndex": 3, "cellParaIndex": 0}
        ])
        .to_string();
        let shape_source = create_hwp_shape_bytes_for_cli(
            &created.bytes,
            0,
            0,
            0,
            5000,
            3000,
            1000,
            2000,
            true,
            "InFrontOfText",
            "textbox",
            false,
            false,
            "[]",
        )
        .expect("create textbox shape source");
        let shape_control = {
            let core = rhwp::document_core::DocumentCore::from_bytes(&shape_source.bytes)
                .expect("reload shape source");
            let para = &core.document().sections[0].paragraphs[shape_source.para_idx];
            match &para.controls[shape_source.control_idx] {
                rhwp::model::control::Control::Shape(shape) => shape.clone(),
                _ => panic!("expected shape control"),
            }
        };

        let mut core =
            rhwp::document_core::DocumentCore::from_bytes(&table.bytes).expect("reload table");
        let shape_control_idx = {
            let section = &mut core.document_mut().sections[0];
            section.raw_stream = None;
            let table_control =
                match &mut section.paragraphs[table.para_idx].controls[table.control_idx] {
                    rhwp::model::control::Control::Table(table) => table,
                    _ => panic!("expected table control"),
                };
            table_control.dirty = true;
            let cell_para = &mut table_control.cells[3].paragraphs[0];
            let control_idx = cell_para.controls.len();
            cell_para
                .controls
                .push(rhwp::model::control::Control::Shape(shape_control));
            cell_para.ctrl_data_records.push(None);
            cell_para.char_count += 8;
            cell_para.control_mask |= 0x00000800;
            cell_para.has_para_text = true;
            control_idx
        };
        let (bytes, _, _) =
            serialize_hwp_verified_for_cli(&mut core).expect("serialize shape cell");
        (
            bytes,
            table.para_idx,
            table.control_idx,
            cell_path,
            shape_control_idx,
        )
    }

    #[test]
    fn cell_shape_object_edits_roundtrip_by_path() {
        let (bytes, table_para_idx, _table_control_idx, cell_path, shape_control_idx) =
            hwp_bytes_with_cell_textbox_shape();

        let props = get_hwp_cell_shape_properties_json_for_cli(
            &bytes,
            0,
            table_para_idx,
            &cell_path,
            shape_control_idx,
        )
        .expect("get cell shape props");
        assert_eq!(props["ok"], true);
        assert_eq!(props["container"], "cell");
        assert_eq!(props["shapeType"], "TextBox");
        assert_eq!(props["width"], 5000);

        let resized = set_hwp_cell_shape_properties_bytes_for_cli(
            &bytes,
            0,
            table_para_idx,
            &cell_path,
            shape_control_idx,
            r#"{"width":6200,"height":3400,"description":"수정 글상자"}"#,
        )
        .expect("set cell shape props");
        assert_eq!(resized.details["operation"], "set-shape-properties");
        assert_eq!(resized.details["container"], "cell");

        let props = get_hwp_cell_shape_properties_json_for_cli(
            &resized.bytes,
            0,
            table_para_idx,
            &cell_path,
            shape_control_idx,
        )
        .expect("get resized cell shape props");
        assert_eq!(props["width"], 6200);
        assert_eq!(props["height"], 3400);
        assert_eq!(props["description"], "수정 글상자");
    }

    #[test]
    fn cell_shape_object_edits_roundtrip_by_row_col() {
        let (bytes, table_para_idx, table_control_idx, _cell_path, shape_control_idx) =
            hwp_bytes_with_cell_textbox_shape();

        let props = get_hwp_cell_shape_properties_at_json_for_cli(
            &bytes,
            0,
            table_para_idx,
            table_control_idx,
            1,
            1,
            0,
            shape_control_idx,
        )
        .expect("get cell shape props by row col");
        assert_eq!(props["container"], "cell");
        assert_eq!(props["row"], 1);
        assert_eq!(props["col"], 1);
        assert_eq!(props["cellIndex"], 3);
        assert_eq!(props["shapeType"], "TextBox");

        let resized = set_hwp_cell_shape_properties_at_bytes_for_cli(
            &bytes,
            0,
            table_para_idx,
            table_control_idx,
            1,
            1,
            0,
            shape_control_idx,
            r#"{"width":6200,"height":3400}"#,
        )
        .expect("set cell shape props by row col");
        assert_eq!(resized.details["container"], "cell");
        assert_eq!(resized.details["row"], 1);
        assert_eq!(resized.details["col"], 1);
        assert_eq!(resized.details["cellIndex"], 3);
        rhwp::document_core::DocumentCore::from_bytes(&resized.bytes)
            .expect("reload row-col-cell-shape-edited hwp");
    }

    #[test]
    fn cell_shape_create_and_delete_roundtrip_by_path() {
        let created = create_hwp_bytes_from_text_for_cli("사업 양식", None).expect("create hwp");
        let table =
            create_hwp_table_bytes_for_cli(&created.bytes, 0, 0, 0, 2, 2).expect("create table");
        let cell_path = serde_json::json!([
            {"controlIndex": table.control_idx, "cellIndex": 3, "cellParaIndex": 0}
        ])
        .to_string();

        let shape = create_hwp_cell_shape_bytes_for_cli(
            &table.bytes,
            0,
            table.para_idx,
            0,
            &cell_path,
            5000,
            3000,
            1000,
            2000,
            true,
            "InFrontOfText",
            "textbox",
            false,
            false,
            "[]",
        )
        .expect("create cell textbox shape");
        assert_eq!(shape.details["operation"], "create-shape");
        assert_eq!(shape.details["container"], "cell");
        assert_eq!(shape.details["cellIndex"], 3);

        let objects = list_hwp_objects_json_for_cli(&shape.bytes).expect("list objects");
        assert_eq!(objects["count"], 1);
        assert_eq!(objects["objects"][0]["container"], "cell");
        assert_eq!(objects["objects"][0]["kind"], "shape");
        assert_eq!(objects["objects"][0]["shapeType"], "TextBox");

        let deleted = delete_hwp_cell_shape_bytes_for_cli(
            &shape.bytes,
            0,
            table.para_idx,
            &cell_path,
            shape.control_idx,
        )
        .expect("delete cell shape");
        assert_eq!(deleted.details["operation"], "delete-shape");
        assert_eq!(deleted.details["container"], "cell");
        let objects = list_hwp_objects_json_for_cli(&deleted.bytes).expect("list after delete");
        assert_eq!(objects["count"], 0);
    }

    #[test]
    fn cell_shape_create_and_delete_roundtrip_by_row_col() {
        let created = create_hwp_bytes_from_text_for_cli("사업 양식", None).expect("create hwp");
        let table =
            create_hwp_table_bytes_for_cli(&created.bytes, 0, 0, 0, 2, 2).expect("create table");

        let shape = create_hwp_cell_shape_at_bytes_for_cli(
            &table.bytes,
            0,
            table.para_idx,
            table.control_idx,
            1,
            1,
            0,
            0,
            5000,
            3000,
            1000,
            2000,
            true,
            "InFrontOfText",
            "textbox",
            false,
            false,
            "[]",
        )
        .expect("create cell textbox shape by row col");
        assert_eq!(shape.details["container"], "cell");
        assert_eq!(shape.details["row"], 1);
        assert_eq!(shape.details["col"], 1);
        assert_eq!(shape.details["cellIndex"], 3);

        let deleted = delete_hwp_cell_shape_at_bytes_for_cli(
            &shape.bytes,
            0,
            table.para_idx,
            table.control_idx,
            1,
            1,
            0,
            shape.control_idx,
        )
        .expect("delete cell shape by row col");
        assert_eq!(deleted.details["container"], "cell");
        assert_eq!(deleted.details["row"], 1);
        assert_eq!(deleted.details["col"], 1);
        assert_eq!(deleted.details["cellIndex"], 3);
        rhwp::document_core::DocumentCore::from_bytes(&deleted.bytes)
            .expect("reload row-col-cell-shape-deleted hwp");
    }

    #[test]
    fn cell_shape_text_roundtrip_by_path() {
        let created = create_hwp_bytes_from_text_for_cli("사업 양식", None).expect("create hwp");
        let table =
            create_hwp_table_bytes_for_cli(&created.bytes, 0, 0, 0, 2, 2).expect("create table");
        let cell_path = serde_json::json!([
            {"controlIndex": table.control_idx, "cellIndex": 3, "cellParaIndex": 0}
        ])
        .to_string();

        let shape = create_hwp_cell_shape_bytes_for_cli(
            &table.bytes,
            0,
            table.para_idx,
            0,
            &cell_path,
            5000,
            3000,
            1000,
            2000,
            true,
            "InFrontOfText",
            "textbox",
            false,
            false,
            "[]",
        )
        .expect("create cell textbox shape");

        let texted = set_hwp_cell_shape_text_bytes_for_cli(
            &shape.bytes,
            0,
            table.para_idx,
            &cell_path,
            shape.control_idx,
            0,
            "안내문",
        )
        .expect("set cell textbox text");
        assert_eq!(texted.details["operation"], "set-cell-shape-text");
        assert_eq!(texted.details["container"], "cell_textbox");
        assert_eq!(texted.details["cellIndex"], 3);
        assert_eq!(texted.details["textboxParagraph"], 0);

        let core =
            rhwp::document_core::DocumentCore::from_bytes(&texted.bytes).expect("reload texted");
        let table_control = match &core.document().sections[0].paragraphs[table.para_idx].controls
            [table.control_idx]
        {
            rhwp::model::control::Control::Table(table) => table,
            _ => panic!("expected table"),
        };
        let shape_control = match &table_control.cells[3].paragraphs[0].controls[shape.control_idx]
        {
            rhwp::model::control::Control::Shape(shape) => shape,
            _ => panic!("expected shape"),
        };
        let textbox = shape_control
            .drawing()
            .and_then(|drawing| drawing.text_box.as_ref())
            .expect("expected textbox");
        assert_eq!(textbox.paragraphs[0].text, "안내문");
    }

    #[test]
    fn cell_shape_text_roundtrip_creates_missing_textbox_paragraph() {
        let created = create_hwp_bytes_from_text_for_cli("사업 양식", None).expect("create hwp");
        let table =
            create_hwp_table_bytes_for_cli(&created.bytes, 0, 0, 0, 2, 2).expect("create table");
        let cell_path = serde_json::json!([
            {"controlIndex": table.control_idx, "cellIndex": 3, "cellParaIndex": 0}
        ])
        .to_string();

        let shape = create_hwp_cell_shape_bytes_for_cli(
            &table.bytes,
            0,
            table.para_idx,
            0,
            &cell_path,
            5000,
            3000,
            1000,
            2000,
            true,
            "InFrontOfText",
            "textbox",
            false,
            false,
            "[]",
        )
        .expect("create cell textbox shape");

        let texted = set_hwp_cell_shape_text_bytes_for_cli(
            &shape.bytes,
            0,
            table.para_idx,
            &cell_path,
            shape.control_idx,
            1,
            "2. 단위는 kg 기준입니다.",
        )
        .expect("set second cell textbox paragraph");
        assert_eq!(texted.details["textboxParagraph"], 1);

        let core =
            rhwp::document_core::DocumentCore::from_bytes(&texted.bytes).expect("reload texted");
        let table_control = match &core.document().sections[0].paragraphs[table.para_idx].controls
            [table.control_idx]
        {
            rhwp::model::control::Control::Table(table) => table,
            _ => panic!("expected table"),
        };
        let shape_control = match &table_control.cells[3].paragraphs[0].controls[shape.control_idx]
        {
            rhwp::model::control::Control::Shape(shape) => shape,
            _ => panic!("expected shape"),
        };
        let textbox = shape_control
            .drawing()
            .and_then(|drawing| drawing.text_box.as_ref())
            .expect("expected textbox");
        assert_eq!(textbox.paragraphs.len(), 2);
        assert_eq!(textbox.paragraphs[1].text, "2. 단위는 kg 기준입니다.");
    }

    #[test]
    fn cell_shape_textbox_format_roundtrip_by_path() {
        let created = create_hwp_bytes_from_text_for_cli("사업 양식", None).expect("create hwp");
        let table =
            create_hwp_table_bytes_for_cli(&created.bytes, 0, 0, 0, 2, 2).expect("create table");
        let cell_path = serde_json::json!([
            {"controlIndex": table.control_idx, "cellIndex": 3, "cellParaIndex": 0}
        ])
        .to_string();

        let shape = create_hwp_cell_shape_bytes_for_cli(
            &table.bytes,
            0,
            table.para_idx,
            0,
            &cell_path,
            5000,
            3000,
            1000,
            2000,
            true,
            "InFrontOfText",
            "textbox",
            false,
            false,
            "[]",
        )
        .expect("create cell textbox shape");
        let texted = set_hwp_cell_shape_text_bytes_for_cli(
            &shape.bytes,
            0,
            table.para_idx,
            &cell_path,
            shape.control_idx,
            1,
            "2. 단위는 kg 기준입니다.",
        )
        .expect("set second cell textbox paragraph");

        let para_formatted = set_hwp_cell_shape_para_format_bytes_for_cli(
            &texted.bytes,
            0,
            table.para_idx,
            &cell_path,
            shape.control_idx,
            1,
            r#"{"alignment":"center"}"#,
        )
        .expect("format cell textbox paragraph");
        assert_eq!(
            para_formatted.details["operation"],
            "set-cell-shape-para-format"
        );
        assert_eq!(para_formatted.details["textboxParagraph"], 1);

        let char_formatted = set_hwp_cell_shape_char_format_bytes_for_cli(
            &para_formatted.bytes,
            0,
            table.para_idx,
            &cell_path,
            shape.control_idx,
            1,
            3,
            5,
            r#"{"bold":true}"#,
        )
        .expect("format cell textbox char range");
        assert_eq!(
            char_formatted.details["operation"],
            "set-cell-shape-char-format"
        );
        assert_eq!(char_formatted.details["textboxParagraph"], 1);

        let core = rhwp::document_core::DocumentCore::from_bytes(&char_formatted.bytes)
            .expect("reload formatted");
        let table_control = match &core.document().sections[0].paragraphs[table.para_idx].controls
            [table.control_idx]
        {
            rhwp::model::control::Control::Table(table) => table,
            _ => panic!("expected table"),
        };
        let shape_control = match &table_control.cells[3].paragraphs[0].controls[shape.control_idx]
        {
            rhwp::model::control::Control::Shape(shape) => shape,
            _ => panic!("expected shape"),
        };
        let textbox = shape_control
            .drawing()
            .and_then(|drawing| drawing.text_box.as_ref())
            .expect("expected textbox");
        let para = &textbox.paragraphs[1];
        let para_shape = &core.document().doc_info.para_shapes[para.para_shape_id as usize];
        assert_eq!(para_shape.alignment, rhwp::model::style::Alignment::Center);
        let char_shape_id = para
            .char_shape_id_at(3)
            .expect("char shape at formatted range");
        assert!(core.document().doc_info.char_shapes[char_shape_id as usize].bold);
    }

    #[test]
    fn cell_shape_textbox_clickhere_field_roundtrips_by_path() {
        let created = create_hwp_bytes_from_text_for_cli("사업 양식", None).expect("create hwp");
        let table =
            create_hwp_table_bytes_for_cli(&created.bytes, 0, 0, 0, 2, 2).expect("create table");
        let cell_path = serde_json::json!([
            {"controlIndex": table.control_idx, "cellIndex": 3, "cellParaIndex": 0}
        ])
        .to_string();

        let shape = create_hwp_cell_shape_bytes_for_cli(
            &table.bytes,
            0,
            table.para_idx,
            0,
            &cell_path,
            5000,
            3000,
            1000,
            2000,
            true,
            "InFrontOfText",
            "textbox",
            false,
            false,
            "[]",
        )
        .expect("create cell textbox shape");
        let texted = set_hwp_cell_shape_text_bytes_for_cli(
            &shape.bytes,
            0,
            table.para_idx,
            &cell_path,
            shape.control_idx,
            0,
            "사업명: ",
        )
        .expect("set cell textbox text");

        let fielded = insert_hwp_clickhere_field_by_path_bytes_for_cli(
            &texted.bytes,
            0,
            table.para_idx,
            &cell_path,
            shape.control_idx,
            0,
            4,
            "biz_name",
            "사업명",
            "사업명을 입력하세요",
            "미입력",
        )
        .expect("insert cell textbox field");
        assert_eq!(fielded.details["operation"], "insert-clickhere-field");
        assert_eq!(fielded.details["container"], "cell_textbox");
        assert_eq!(fielded.details["cellIndex"], 3);
        assert_eq!(fielded.details["textboxParagraph"], 0);

        let fields = list_hwp_fields_json_for_cli(&fielded.bytes).expect("list fields");
        assert_eq!(fields["count"], 1);
        assert_eq!(fields["fields"][0]["name"], "biz_name");
        assert_eq!(fields["fields"][0]["value"], "미입력");
        assert_eq!(fields["fields"][0]["location"]["path"][0]["type"], "cell");
        assert_eq!(
            fields["fields"][0]["location"]["path"][1]["type"],
            "textbox"
        );

        let filled = set_hwp_field_bytes_for_cli(&fielded.bytes, "biz_name", "AI LCA 자동화")
            .expect("fill cell textbox field");
        let fields = list_hwp_fields_json_for_cli(&filled.bytes).expect("list filled fields");
        assert_eq!(fields["fields"][0]["value"], "AI LCA 자동화");

        let full_path = serde_json::json!([
            {"controlIndex": table.control_idx, "cellIndex": 3, "cellParaIndex": 0},
            {"controlIndex": shape.control_idx, "cellIndex": 0, "cellParaIndex": 0}
        ])
        .to_string();
        let core = rhwp::document_core::DocumentCore::from_bytes(&filled.bytes)
            .expect("reload filled cell textbox field");
        let path = parse_cell_path_for_cli(&full_path).expect("parse full cell textbox path");
        let info = parse_json_value(&core.get_field_info_at_by_path(0, table.para_idx, &path, 5));
        assert_eq!(info["inField"], true);
        assert_eq!(info["fieldType"], "clickhere");
        assert_eq!(info["guideName"], "사업명");

        let info = get_hwp_field_info_by_path_json_for_cli(
            &filled.bytes,
            0,
            table.para_idx,
            &cell_path,
            shape.control_idx,
            0,
            5,
        )
        .expect("get cell textbox field info by path");
        assert_eq!(info["inField"], true);
        assert_eq!(info["fieldType"], "clickhere");
        assert_eq!(info["guideName"], "사업명");

        let removed = remove_hwp_field_by_path_bytes_for_cli(
            &filled.bytes,
            0,
            table.para_idx,
            &cell_path,
            shape.control_idx,
            0,
            5,
        )
        .expect("remove cell textbox field by path");
        assert_eq!(removed.details["operation"], "remove-field");
        assert_eq!(removed.details["container"], "cell_textbox");
        let fields = list_hwp_fields_json_for_cli(&removed.bytes).expect("list removed fields");
        assert_eq!(fields["count"], 0);
    }

    #[test]
    fn cell_shape_textbox_clickhere_field_roundtrips_by_row_col() {
        let created = create_hwp_bytes_from_text_for_cli("사업 양식", None).expect("create hwp");
        let table =
            create_hwp_table_bytes_for_cli(&created.bytes, 0, 0, 0, 2, 2).expect("create table");
        let shape = create_hwp_cell_shape_at_bytes_for_cli(
            &table.bytes,
            0,
            table.para_idx,
            table.control_idx,
            1,
            1,
            0,
            0,
            5000,
            3000,
            1000,
            2000,
            true,
            "InFrontOfText",
            "textbox",
            false,
            false,
            "[]",
        )
        .expect("create cell textbox shape by row col");
        let texted = set_hwp_cell_shape_text_at_bytes_for_cli(
            &shape.bytes,
            0,
            table.para_idx,
            table.control_idx,
            1,
            1,
            0,
            shape.control_idx,
            0,
            "사업명: ",
        )
        .expect("set cell textbox text by row col");

        let fielded = insert_hwp_cell_shape_clickhere_field_at_bytes_for_cli(
            &texted.bytes,
            0,
            table.para_idx,
            table.control_idx,
            1,
            1,
            0,
            shape.control_idx,
            0,
            4,
            "biz_name",
            "사업명",
            "사업명을 입력하세요",
            "미입력",
        )
        .expect("insert cell textbox field by row col");
        assert_eq!(fielded.details["operation"], "insert-clickhere-field");
        assert_eq!(fielded.details["container"], "cell_textbox");
        assert_eq!(fielded.details["row"], 1);
        assert_eq!(fielded.details["col"], 1);

        let info = get_hwp_cell_shape_field_info_at_json_for_cli(
            &fielded.bytes,
            0,
            table.para_idx,
            table.control_idx,
            1,
            1,
            0,
            shape.control_idx,
            0,
            5,
        )
        .expect("get cell textbox field info by row col");
        assert_eq!(info["inField"], true);
        assert_eq!(info["fieldType"], "clickhere");

        let removed = remove_hwp_cell_shape_field_at_bytes_for_cli(
            &fielded.bytes,
            0,
            table.para_idx,
            table.control_idx,
            1,
            1,
            0,
            shape.control_idx,
            0,
            5,
        )
        .expect("remove cell textbox field by row col");
        let fields = list_hwp_fields_json_for_cli(&removed.bytes).expect("list removed fields");
        assert_eq!(fields["count"], 0);
    }

    #[test]
    fn cell_picture_insert_roundtrip_by_row_col() {
        let created = create_hwp_bytes_from_text_for_cli("사업 양식", None).expect("create hwp");
        let table =
            create_hwp_table_bytes_for_cli(&created.bytes, 0, 0, 0, 2, 2).expect("create table");
        let png_bytes = [
            0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1a, b'\n', 0, 0, 0, 0,
        ];

        let picture = insert_hwp_cell_picture_at_bytes_for_cli(
            &table.bytes,
            0,
            table.para_idx,
            table.control_idx,
            1,
            1,
            0,
            0,
            &png_bytes,
            1400,
            900,
            1,
            1,
            "png",
            "회사 도장",
            None,
            None,
        )
        .expect("insert cell picture by row col");
        assert_eq!(picture.details["operation"], "insert-picture");
        assert_eq!(picture.details["container"], "cell");
        assert_eq!(picture.details["row"], 1);
        assert_eq!(picture.details["col"], 1);
        assert_eq!(picture.details["cellIndex"], 3);

        let props = get_hwp_cell_picture_properties_at_json_for_cli(
            &picture.bytes,
            0,
            table.para_idx,
            table.control_idx,
            1,
            1,
            0,
            picture.control_idx,
        )
        .expect("get inserted cell picture props by row col");
        assert_eq!(props["container"], "cell");
        assert_eq!(props["row"], 1);
        assert_eq!(props["col"], 1);
        assert_eq!(props["cellIndex"], 3);
        assert_eq!(props["description"], "회사 도장");
    }

    #[test]
    fn cell_picture_list_objects_reports_cell_location_for_floating_insert() {
        let created = create_hwp_bytes_from_text_for_cli("사업 양식", None).expect("create hwp");
        let table =
            create_hwp_table_bytes_for_cli(&created.bytes, 0, 0, 0, 2, 2).expect("create table");
        let png_bytes = [
            0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1a, b'\n', 0, 0, 0, 0,
        ];

        let picture = insert_hwp_cell_picture_at_bytes_for_cli(
            &table.bytes,
            0,
            table.para_idx,
            table.control_idx,
            1,
            1,
            0,
            0,
            &png_bytes,
            1400,
            900,
            1,
            1,
            "png",
            "회사 도장",
            None,
            None,
        )
        .expect("insert cell picture by row col");

        let list = list_hwp_objects_json_for_cli(&picture.bytes).expect("list objects");
        assert_eq!(list["count"], 1);
        let item = &list["objects"][0];
        assert_eq!(item["kind"], "picture");
        assert_eq!(item["container"], "cell");
        assert_eq!(item["section"], 0);
        assert_eq!(item["paragraph"], table.para_idx);
        assert_eq!(item["control"], picture.control_idx);
        assert_eq!(item["tableControl"], table.control_idx);
        assert_eq!(item["row"], 1, "listed item: {}", item);
        assert_eq!(item["col"], 1, "listed item: {}", item);
        assert_eq!(item["cellIndex"], 3, "listed item: {}", item);
        assert_eq!(item["cellParagraph"], 0);
        assert_eq!(item["cellPath"][0]["controlIndex"], table.control_idx);
        assert_eq!(item["cellPath"][0]["cellIndex"], 3);
        assert_eq!(item["cellPath"][0]["cellParaIndex"], 0);
    }

    #[test]
    fn cell_picture_object_edits_roundtrip_by_row_col() {
        let created = create_hwp_bytes_from_text_for_cli("사업 양식", None).expect("create hwp");
        let table =
            create_hwp_table_bytes_for_cli(&created.bytes, 0, 0, 0, 2, 2).expect("create table");
        let png_bytes = [
            0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1a, b'\n', 0, 0, 0, 0,
        ];
        let cell_path = serde_json::json!([
            {"controlIndex": table.control_idx, "cellIndex": 3, "cellParaIndex": 0}
        ])
        .to_string();

        let picture = insert_hwp_picture_bytes_for_cli(
            &table.bytes,
            0,
            table.para_idx,
            0,
            &cell_path,
            &png_bytes,
            1400,
            900,
            1,
            1,
            "png",
            "회사 도장",
            None,
            None,
        )
        .expect("insert cell picture");

        let props = get_hwp_cell_picture_properties_at_json_for_cli(
            &picture.bytes,
            0,
            table.para_idx,
            table.control_idx,
            1,
            1,
            0,
            picture.control_idx,
        )
        .expect("get cell picture props by row col");
        assert_eq!(props["container"], "cell");
        assert_eq!(props["row"], 1);
        assert_eq!(props["col"], 1);
        assert_eq!(props["cellIndex"], 3);
        assert_eq!(props["description"], "회사 도장");

        let resized = set_hwp_cell_picture_properties_at_bytes_for_cli(
            &picture.bytes,
            0,
            table.para_idx,
            table.control_idx,
            1,
            1,
            0,
            picture.control_idx,
            r#"{"width":1800,"height":1100,"description":"수정 도장"}"#,
        )
        .expect("set cell picture props by row col");
        assert_eq!(resized.details["container"], "cell");
        assert_eq!(resized.details["row"], 1);
        assert_eq!(resized.details["col"], 1);
        assert_eq!(resized.details["cellIndex"], 3);

        let deleted = delete_hwp_cell_picture_at_bytes_for_cli(
            &resized.bytes,
            0,
            table.para_idx,
            table.control_idx,
            1,
            1,
            0,
            picture.control_idx,
        )
        .expect("delete cell picture by row col");
        assert_eq!(deleted.details["container"], "cell");
        assert_eq!(deleted.details["row"], 1);
        assert_eq!(deleted.details["col"], 1);
        assert_eq!(deleted.details["cellIndex"], 3);
        rhwp::document_core::DocumentCore::from_bytes(&deleted.bytes)
            .expect("reload row-col-cell-picture-deleted hwp");
    }

    #[test]
    fn shape_z_order_group_and_ungroup_roundtrip() {
        let created =
            create_hwp_bytes_from_text_for_cli("사업 양식\n본문", None).expect("create hwp");

        let first = create_hwp_shape_bytes_for_cli(
            &created.bytes,
            0,
            0,
            0,
            4200,
            2200,
            1000,
            1500,
            false,
            "InFrontOfText",
            "rectangle",
            false,
            false,
            "[]",
        )
        .expect("create first shape");
        let second = create_hwp_shape_bytes_for_cli(
            &first.bytes,
            0,
            1,
            0,
            3600,
            1800,
            5200,
            1800,
            false,
            "InFrontOfText",
            "ellipse",
            false,
            false,
            "[]",
        )
        .expect("create second shape");

        let reordered = change_hwp_shape_z_order_bytes_for_cli(
            &second.bytes,
            0,
            first.para_idx,
            first.control_idx,
            "front",
        )
        .expect("change z order");
        assert_eq!(reordered.details["operation"], "change-shape-z-order");
        assert!(reordered.details["zOrder"].as_i64().unwrap_or(-1) >= 0);

        let targets_json = serde_json::json!([
            {"paraIdx": first.para_idx, "controlIdx": first.control_idx},
            {"paraIdx": second.para_idx, "controlIdx": second.control_idx}
        ])
        .to_string();
        let grouped = group_hwp_shapes_bytes_for_cli(&reordered.bytes, 0, &targets_json)
            .expect("group shapes");
        assert_eq!(grouped.details["operation"], "group-shapes");
        assert_eq!(grouped.para_idx, 0);
        let props = get_hwp_shape_properties_json_for_cli(
            &grouped.bytes,
            0,
            grouped.para_idx,
            grouped.control_idx,
        )
        .expect("get group props");
        assert_eq!(props["description"], "묶음 개체입니다.");
        assert!(props["width"].as_u64().unwrap_or(0) >= 7800);

        let ungrouped = ungroup_hwp_shape_bytes_for_cli(
            &grouped.bytes,
            0,
            grouped.para_idx,
            grouped.control_idx,
        )
        .expect("ungroup shapes");
        assert_eq!(ungrouped.details["operation"], "ungroup-shape");
        rhwp::document_core::DocumentCore::from_bytes(&ungrouped.bytes)
            .expect("reload ungrouped hwp");
    }

    #[test]
    fn header_footer_create_edit_and_delete_roundtrip() {
        let created =
            create_hwp_bytes_from_text_for_cli("사업 양식\n본문", None).expect("create hwp");

        let header = create_hwp_header_footer_bytes_for_cli(&created.bytes, 0, true, 0)
            .expect("create header");
        assert_eq!(header.details["operation"], "create-header-footer");
        assert_eq!(header.details["kind"], "header");
        assert_eq!(header.details["applyTo"], 0);

        let header = insert_hwp_header_footer_text_bytes_for_cli(
            &header.bytes,
            0,
            true,
            0,
            0,
            0,
            "LCA 사업 양식",
        )
        .expect("insert header text");
        assert_eq!(header.details["operation"], "insert-header-footer-text");

        let info =
            get_hwp_header_footer_json_for_cli(&header.bytes, 0, true, 0).expect("get header");
        assert_eq!(info["exists"], true);
        assert_eq!(info["text"], "LCA 사업 양식");
        assert_eq!(info["paraCount"], 1);

        let split =
            split_hwp_header_footer_paragraph_bytes_for_cli(&header.bytes, 0, true, 0, 0, 4)
                .expect("split header paragraph");
        assert_eq!(split.details["operation"], "split-header-footer-paragraph");
        let info = get_hwp_header_footer_para_info_json_for_cli(&split.bytes, 0, true, 0, 1)
            .expect("get split paragraph info");
        assert_eq!(info["paraCount"], 2);

        let merged = merge_hwp_header_footer_paragraph_bytes_for_cli(&split.bytes, 0, true, 0, 1)
            .expect("merge header paragraph");
        assert_eq!(merged.details["operation"], "merge-header-footer-paragraph");

        let trimmed =
            delete_hwp_header_footer_text_bytes_for_cli(&merged.bytes, 0, true, 0, 0, 0, 4)
                .expect("delete header text");
        let info = get_hwp_header_footer_json_for_cli(&trimmed.bytes, 0, true, 0)
            .expect("get trimmed header");
        assert_eq!(info["text"], "사업 양식");

        let footer = create_hwp_header_footer_bytes_for_cli(&trimmed.bytes, 0, false, 2)
            .expect("create odd footer");
        let list = list_hwp_header_footer_json_for_cli(&footer.bytes, 0, false, 2)
            .expect("list header/footer");
        assert_eq!(list["items"].as_array().unwrap().len(), 2);
        assert_eq!(list["currentIndex"], 1);

        let deleted = delete_hwp_header_footer_bytes_for_cli(&footer.bytes, 0, true, 0)
            .expect("delete header");
        let info = get_hwp_header_footer_json_for_cli(&deleted.bytes, 0, true, 0)
            .expect("get deleted header");
        assert_eq!(info["exists"], false);

        rhwp::document_core::DocumentCore::from_bytes(&deleted.bytes)
            .expect("reload header-footer-edited hwp");
    }

    #[test]
    fn header_footer_format_field_and_template_roundtrip() {
        let created =
            create_hwp_bytes_from_text_for_cli("사업 양식\n본문", None).expect("create hwp");

        let footer = create_hwp_header_footer_bytes_for_cli(&created.bytes, 0, false, 0)
            .expect("create footer");
        let formatted = set_hwp_header_footer_para_format_bytes_for_cli(
            &footer.bytes,
            0,
            false,
            0,
            0,
            r#"{"alignment":"center","lineSpacing":170,"lineSpacingType":"Percent"}"#,
        )
        .expect("set footer paragraph format");
        assert_eq!(
            formatted.details["operation"],
            "set-header-footer-para-format"
        );

        let props =
            get_hwp_header_footer_para_properties_json_for_cli(&formatted.bytes, 0, false, 0, 0)
                .expect("get footer paragraph properties");
        assert_eq!(props["alignment"], "center");
        assert_eq!(props["lineSpacing"], 170.0);

        let fielded =
            insert_hwp_header_footer_field_bytes_for_cli(&formatted.bytes, 0, false, 0, 0, 0, 1)
                .expect("insert page number field");
        assert_eq!(fielded.details["operation"], "insert-header-footer-field");
        assert_eq!(fielded.details["charOffset"], 1);

        let templated =
            apply_hwp_header_footer_template_bytes_for_cli(&fielded.bytes, 0, true, 0, 4)
                .expect("apply header template");
        assert_eq!(
            templated.details["operation"],
            "apply-header-footer-template"
        );
        let header_info = get_hwp_header_footer_json_for_cli(&templated.bytes, 0, true, 0)
            .expect("get templated header");
        let text = header_info["text"].as_str().unwrap_or_default();
        assert!(text.contains('\t'));
        let header_props =
            get_hwp_header_footer_para_properties_json_for_cli(&templated.bytes, 0, true, 0, 0)
                .expect("get templated header paragraph properties");
        assert_eq!(header_props["alignment"], "left");
        assert!(!header_props["tabStops"].as_array().unwrap().is_empty());

        rhwp::document_core::DocumentCore::from_bytes(&templated.bytes)
            .expect("reload header-footer-format-edited hwp");
    }

    #[test]
    fn master_page_create_set_text_and_delete_roundtrip() {
        let created =
            create_hwp_bytes_from_text_for_cli("사업 양식\n본문", None).expect("create hwp");

        let created_master = create_hwp_master_page_bytes_for_cli(
            &created.bytes,
            0,
            0,
            false,
            false,
            "CONFIDENTIAL",
        )
        .expect("create master page");
        assert_eq!(created_master.details["operation"], "create-master-page");
        assert_eq!(created_master.details["masterPageIndex"], 0);

        let list = list_hwp_master_pages_json_for_cli(&created_master.bytes, 0)
            .expect("list master pages");
        assert_eq!(list["count"], 1);
        assert_eq!(list["items"][0]["applyTo"], 0);
        assert_eq!(list["items"][0]["text"], "CONFIDENTIAL");

        let updated =
            set_hwp_master_page_text_bytes_for_cli(&created_master.bytes, 0, 0, 0, "LCA 사업 양식")
                .expect("set master page text");
        assert_eq!(updated.details["operation"], "set-master-page-text");
        let list = list_hwp_master_pages_json_for_cli(&updated.bytes, 0)
            .expect("list updated master pages");
        assert_eq!(list["items"][0]["text"], "LCA 사업 양식");

        let deleted =
            delete_hwp_master_page_bytes_for_cli(&updated.bytes, 0, 0).expect("delete master page");
        assert_eq!(deleted.details["operation"], "delete-master-page");
        let list = list_hwp_master_pages_json_for_cli(&deleted.bytes, 0)
            .expect("list deleted master pages");
        assert_eq!(list["count"], 0);

        rhwp::document_core::DocumentCore::from_bytes(&deleted.bytes)
            .expect("reload master-page-edited hwp");
    }
}

#[cfg(test)]
mod tests {
    use super::tab_ext_semantic_differs;

    #[test]
    fn tab_ext_reserved_fields_ignored() {
        // 같은 문서의 HWPX(파서가 [1],[3..6]=0) vs HWP5([1]=leader/fill 슬롯, [3..6]=원본 바이트).
        // 이 포맷 비대칭 슬롯들은 모두 무시 → 의미 차이 없음.
        let hwpx = [1640, 0, 256, 0, 0, 0, 9];
        let hwp5 = [1640, 5, 256, 32, 32, 32, 9];
        assert!(!tab_ext_semantic_differs(&hwpx, &hwp5));
    }

    #[test]
    fn tab_ext_semantic_fields_detected() {
        let base = [1640, 0, 256, 0, 0, 0, 9];
        assert!(!tab_ext_semantic_differs(&base, &base));
        // width([0]) 차이 검출
        assert!(tab_ext_semantic_differs(&base, &[1641, 0, 256, 0, 0, 0, 9]));
        // type([2] high byte) 차이 검출 — 256(0x0100)→512(0x0200)
        assert!(tab_ext_semantic_differs(&base, &[1640, 0, 512, 0, 0, 0, 9]));
        // leader([2] low byte, 두 포맷 공통) 차이 검출 — 256(0x0100)→257(0x0101)
        assert!(tab_ext_semantic_differs(&base, &[1640, 0, 257, 0, 0, 0, 9]));
        // HWP5 leader/fill 슬롯([1], HWPX는 항상 0)은 포맷 비대칭이라 무시 — 차이로 치지 않음
        assert!(!tab_ext_semantic_differs(
            &base,
            &[1640, 1, 256, 0, 0, 0, 9]
        ));
        // marker([6]) 차이 검출
        assert!(tab_ext_semantic_differs(&base, &[1640, 0, 256, 0, 0, 0, 0]));
    }
}
