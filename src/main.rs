use std::env;
use std::fs;
use std::path::Path;
use std::process;

mod agent_profiles;
mod anchor_log;
mod atomic_file;
mod audit_standard;
mod capsule_sign;
mod cli;
mod disclose;
mod lineage_bundle;
mod mcp_serve;
mod policy_gate;
mod settle;
use cli::commands::edit::runtime::{
    check_expect_sha256, edit_output_format, edit_serialize, edit_verify_report, finish_edit_write,
    EditOutputFormat,
};
pub(crate) use cli::commands::edit::{
    measure_cell_overflow, recolor_cell_text_black, resolve_table_cell,
    set_cell_control_char_rejection, CellResolveError,
};
pub(crate) use cli::document_io::{
    classify_hwp_error, cli_output_password, cli_password, load_document, load_document_core,
    LoadError,
};
use cli::document_io::{
    has_global_auth_option, is_batch_invocation, set_cli_output_password, set_cli_password,
    strip_global_auth_options, strip_utf8_bom,
};
use cli::integrity::{
    cas_test_mark_checked_and_wait, cas_test_synchronize_before_lock, sha256_hex_of, CasPathLock,
};
pub(crate) use cli::units::{hu_to_mm, hu_to_mm_i};
use rhwp::provenance;
use rhwp::schema_registry::ENVELOPE_SCHEMA_VERSION;

/// [#2707] CLI 종료 코드 계약 — 성공.
const EXIT_OK: i32 = 0;
/// [#2707] CLI 종료 코드 계약 — 런타임 실패(읽기·파싱·렌더·쓰기).
const EXIT_RUNTIME: i32 = 1;
/// [#2707] CLI 종료 코드 계약 — 사용법 오류(인자 없음, 알 수 없는 옵션/명령, 페이지 범위 초과).
///
/// 3(`--verify` IR 차이)·4(`--verify-pages` 페이지 수 불일치)는
/// `mydocs/manual/cli_commands.md` 에 이미 문서화된 계약이므로 상수화 대상에서 제외하고
/// 기존 `process::exit(3)`/`process::exit(4)` 호출부를 그대로 둔다.
const EXIT_USAGE: i32 = 2;

/// [#2707] 명령 함수가 돌려준 종료 코드를 프로세스 종료 코드로 전파한다.
///
/// 0이면 아무것도 하지 않아 `main` 이 정상 종료하고, 그 외에는 즉시 그 코드로 종료한다.
fn exit_with(exit_code: i32) {
    if exit_code != EXIT_OK {
        process::exit(exit_code);
    }
}

/// 쪽수와 IR 검증은 모두 수행하되, 종료 코드는 쪽수 실패를 우선한다.
fn verification_exit_code(page_failed: bool, ir_failed: bool) -> i32 {
    if page_failed {
        4
    } else if ir_failed {
        3
    } else {
        EXIT_OK
    }
}

#[cfg(test)]
mod verification_exit_code_tests {
    use super::verification_exit_code;

    #[test]
    fn page_failure_keeps_precedence_when_ir_also_fails() {
        assert_eq!(verification_exit_code(false, false), 0);
        assert_eq!(verification_exit_code(false, true), 3);
        assert_eq!(verification_exit_code(true, false), 4);
        assert_eq!(verification_exit_code(true, true), 4);
    }
}

fn main() {
    let raw_args: Vec<String> = env::args().collect();
    if is_batch_invocation(&raw_args) && has_global_auth_option(&raw_args) {
        eprintln!(
            "오류: batch 는 --password·--password-stdin·--output-password·--output-password-stdin 을 지원하지 않습니다. stdin 은 파일 경로 목록 전용입니다."
        );
        process::exit(EXIT_USAGE);
    }
    // 전역 인증 pre-scan: 어느 위치든 입력/출력 비밀번호 옵션을 뽑아낸다.
    // 비밀번호는 pre-scan 안에서 thread-local 상태로 들어가고 여기로는 돌아오지 않는다.
    let args = match strip_global_auth_options(raw_args) {
        Ok(v) => v,
        Err(code) => process::exit(code),
    };

    // [#5791] `rhwp <명령> [<하위명령>] --help` — 그 명령 절만 내고 exit 0.
    // 디스패치보다 앞이다: 하위 명령 판정이나 옵션 파서에 닿기 전에 답해야
    // "알 수 없는 옵션"·"파일을 읽을 수 없습니다"로 떨어지지 않는다.
    if let Some(code) = cli::metadata::help::scoped_help(&args[1..]) {
        process::exit(code);
    }

    match args.get(1).map(|s| s.as_str()) {
        Some("--help") | Some("-h") => cli::metadata::help::print_help(),
        Some("--version") | Some("-V") => println!("rhwp v{}", rhwp::version()),
        Some("export-svg") => exit_with(cli::outputs::vector::export_svg(&args[2..])),
        Some("export-render-tree") => {
            exit_with(cli::outputs::vector::export_render_tree(&args[2..]))
        }
        Some("export-structure") => {
            exit_with(cli::queries::structure::export_structure(&args[2..]))
        }
        Some("export-png") => exit_with(cli::outputs::raster::export_png(&args[2..])),
        // [gym_gpu_raster] GPU 가속 PNG 래스터화 (feature = "gpu"). export-png(native-skia)과
        // 같은 방식으로 feature 게이팅 — 미빌드 바이너리는 사용법 오류(exit 2)로 안내한다.
        Some("export-png-gpu") => exit_with(cli::outputs::raster::export_png_gpu(&args[2..])),
        Some("gpu-info") => exit_with(cli::outputs::raster::gpu_info(&args[2..])),
        Some("export-pdf") => exit_with(cli::outputs::pdf::export_pdf(&args[2..])),
        Some("export-text") => exit_with(cli::outputs::text::export_text(&args[2..])),
        Some("export-markdown") => exit_with(cli::outputs::text::export_markdown(&args[2..])),
        Some("export-tables") => exit_with(cli::outputs::tabular::export_tables(&args[2..])),
        Some("export-llm") => exit_with(cli::outputs::text::export_llm(&args[2..])),
        Some("table-to-csv") => exit_with(cli::outputs::tabular::table_to_csv(&args[2..])),
        Some("csv-to-table") => exit_with(cli::commands::tabular_import::csv_to_table(&args[2..])),
        Some("chart-to-csv") => exit_with(cli::outputs::tabular::chart_to_csv(&args[2..])),
        Some("csv-to-chart") => exit_with(cli::commands::tabular_import::csv_to_chart(&args[2..])),
        Some("export-hwpx") => exit_with(cli::commands::conversion::export_hwpx(&args[2..])),
        Some("export-hml") => cli::commands::conversion::export_hml(&args[2..]),
        Some("export-doclang") => exit_with(cli::outputs::doclang::export_doclang(&args[2..])),
        Some("export-ir-schema") => exit_with(cmd_export_ir_schema(&args[2..])),
        Some("export-capabilities-schema") => exit_with(cmd_export_capabilities_schema(&args[2..])),
        Some("export-ontology") => exit_with(cmd_export_ontology(&args[2..])),
        Some("capabilities") => {
            exit_with(cli::metadata::capabilities::show_capabilities(&args[2..]))
        }
        Some("export-provenance-map") => exit_with(
            cli::metadata::capabilities::export_provenance_map(&args[2..]),
        ),
        Some("export-agent-manifest") => exit_with(cmd_export_agent_manifest(&args[2..])),
        Some("mcp-serve") => exit_with(mcp_serve::run(&args[2..])),
        Some("batch") => exit_with(cli::batch::run(&args[2..])),
        Some("scan") => exit_with(cli::queries::scan::run(&args[2..])),
        Some("threat-scan") => {
            exit_with(cli::queries::security_inspection::threat_scan(&args[2..]))
        }
        Some("info") => exit_with(cli::queries::info::run(&args[2..])),
        Some("word-count") => exit_with(cli::queries::document_inventory::word_count(&args[2..])),
        Some("bookmarks") => exit_with(cli::queries::document_inventory::bookmarks(&args[2..])),
        Some("charts") => exit_with(cli::queries::document_inventory::charts(&args[2..])),
        Some("form-value") => exit_with(cli::queries::structured_objects::form_value(&args[2..])),
        Some("header-footer") => {
            exit_with(cli::queries::structured_objects::header_footer(&args[2..]))
        }
        Some("headers-footers") => exit_with(cli::queries::structured_objects::headers_footers(
            &args[2..],
        )),
        Some("digest") => exit_with(cli::queries::digest::digest_document(&args[2..])),
        Some("dump") => exit_with(cli::queries::control_dump::run(&args[2..])),
        Some("dump-note-shape") => {
            exit_with(cli::queries::diagnostics::dump_note_shape(&args[2..]))
        }
        Some("dump-endnote-lines") => {
            exit_with(cli::queries::diagnostics::dump_endnote_lines(&args[2..]))
        }
        Some("dump-pages") => exit_with(cli::queries::page_dump::run(&args[2..])),
        Some("dump-extents") => exit_with(cli::queries::diagnostics::dump_extents(&args[2..])),
        Some("diag") => exit_with(cli::queries::diagnostics::diag_document(&args[2..])),
        Some("search") => exit_with(cli::queries::search::search_document(&args[2..])),
        Some("inspect") => exit_with(inspect_command(&args[2..])),
        Some("armor") => exit_with(cli::queries::security_inspection::armor_command(&args[2..])),
        Some("extract-data") => exit_with(cli::queries::data_extraction::extract_data_command(
            &args[2..],
        )),
        Some("convert") => exit_with(cli::commands::conversion::convert_hwp(&args[2..])),
        Some("extract-pages") => exit_with(cli::commands::conversion::extract_pages(&args[2..])),
        Some("build-from-ingest") => {
            exit_with(cli::commands::generation::build_from_ingest(&args[2..]))
        }
        Some("scaffold") => exit_with(cli::commands::generation::run_scaffold(&args[2..])),
        Some("hwp5-inventory") => exit_with(rhwp::diagnostics::hwp5_inventory::run(&args[2..])),
        Some("hwp5-inventory-diff") => {
            exit_with(rhwp::diagnostics::hwp5_inventory_diff::run(&args[2..]))
        }
        Some("hwp5-contract-analyze") => {
            exit_with(rhwp::diagnostics::hwp5_contract_analyze::run(&args[2..]))
        }
        Some("hwp5-ctrl-data-trace") => {
            exit_with(rhwp::diagnostics::hwp5_ctrl_data_trace::run(&args[2..]))
        }
        Some("hwp5-contract-probe") => {
            exit_with(rhwp::diagnostics::hwp5_contract_probe::run(&args[2..]))
        }
        Some("hwp5-table-probe") => exit_with(rhwp::diagnostics::hwp5_table_probe::run(&args[2..])),
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
        Some("move-table-to-cell") => move_table_to_cell_cli(&args[2..]),
        Some("move-table-from-cell") => move_table_from_cell_cli(&args[2..]),
        Some("repair-nested-tables") => repair_nested_tables_cli(&args[2..]),
        Some("lint") => lint_cli(&args[2..]),
        Some("copy-table") => table_structure_cli(&args[2..], "copy-table"),
        Some("delete-table") => table_structure_cli(&args[2..], "delete-table"),
        Some("set-cell-text") => set_cell_text_cli(&args[2..]),
        Some("insert-cell-text") => cell_text_edit_cli(&args[2..], false),
        Some("delete-cell-text") => cell_text_edit_cli(&args[2..], true),
        Some("insert-cell-paragraph") => cell_paragraph_edit_cli(&args[2..], false),
        Some("delete-cell-paragraph") => cell_paragraph_edit_cli(&args[2..], true),
        Some("move-cell-paragraphs") => move_cell_paragraphs_cli(&args[2..]),
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
        Some("get-cell-text") => get_cell_text_cli(&args[2..]),
        Some("set-cell-properties") => set_table_properties_cli(&args[2..], true),
        Some("get-table-properties") => get_table_properties_cli(&args[2..], false),
        Some("set-table-properties") => set_table_properties_cli(&args[2..], false),
        Some("resize-table-cells") => resize_table_cells_cli(&args[2..]),
        Some("set-table-column-widths") => set_table_column_widths_cli(&args[2..]),
        Some("apply-table-style") => apply_table_style_cli(&args[2..]),
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
        Some("hwp5-mel-personnel-probe") => {
            exit_with(rhwp::diagnostics::hwp5_mel_personnel_probe::run(&args[2..]))
        }
        Some("hwp5-borderfill-diagonal-probe") => exit_with(
            rhwp::diagnostics::hwp5_borderfill_diagonal_probe::run(&args[2..]),
        ),
        Some("hwp5-first-para-control-probe") => exit_with(
            rhwp::diagnostics::hwp5_first_para_control_probe::run(&args[2..]),
        ),
        Some("hwp5-anchor-trace") => {
            exit_with(rhwp::diagnostics::hwp5_anchor_trace::run(&args[2..]))
        }
        Some("hwp5-char-shape-audit") => {
            exit_with(rhwp::diagnostics::hwp5_char_shape_audit::run(&args[2..]))
        }
        Some("hwp5-cell-header-probe") => {
            exit_with(rhwp::diagnostics::hwp5_cell_header_probe::run(&args[2..]))
        }
        Some("dump-records") => exit_with(cli::queries::diagnostics::dump_raw_records(&args[2..])),
        Some("test-shape") => exit_with(test_shape_roundtrip(&args[2..])),
        Some("test-caption") => exit_with(cli::commands::caption_validation::run(&args[2..])),
        Some("gen-table") => exit_with(gen_table(&args[2..])),
        Some("gen-pua") => exit_with(gen_pua_test(&args[2..])),
        Some("test-field") => exit_with(cli::commands::internal_validation::run(&args[2..])),
        Some("ir-diff") => exit_with(cli::queries::ir_comparison::ir_diff(&args[2..])),
        Some("ir-sweep") => exit_with(cli::queries::ir_comparison::ir_sweep(&args[2..])),
        Some("dump-anchors") => {
            exit_with(cli::queries::position_diagnostics::dump_anchors(&args[2..]))
        }
        Some("dump-carets") => {
            exit_with(cli::queries::position_diagnostics::dump_carets(&args[2..]))
        }
        Some("verify") => exit_with(cli::queries::verification::run(&args[2..])),
        Some("hwpx-roundtrip") => rhwp::diagnostics::hwpx_roundtrip_batch::run(&args[2..]),
        Some("hwp5-roundtrip") => rhwp::diagnostics::hwp5_roundtrip_batch::run(&args[2..]),
        Some("render-diff") => rhwp::diagnostics::render_geom_diff::run(&args[2..]),
        Some("layout-anomaly") => exit_with(rhwp::diagnostics::layout_anomaly::run(&args[2..])),
        Some("measure-width") => exit_with(rhwp::diagnostics::text_width_probe::run(&args[2..])),
        Some("core-pages") => exit_with(rhwp::diagnostics::core_pages_probe::run(&args[2..])),
        Some("bench") => exit_with(rhwp::diagnostics::bench::run(&args[2..])),
        Some("thumbnail") => exit_with(cli::outputs::preview::extract_thumbnail(&args[2..])),
        Some("fields") => exit_with(cli::queries::document_inventory::show_fields(&args[2..])),
        Some("explain") => exit_with(cli::queries::explain::explain_document(&args[2..])),
        Some("explore") => exit_with(cli::queries::explore::explore_document(&args[2..])),
        Some("edit") => exit_with(cli::commands::edit::run(&args[2..])),
        Some("run") => exit_with(cli::protocol::cmd_run_plan(&args[2..])),
        Some("replay") => exit_with(cli::protocol::cmd_replay(&args[2..])),
        Some("audit") => exit_with(cli::protocol::cmd_audit(&args[2..])),
        Some("lineage") => exit_with(cli::protocol::cmd_lineage(&args[2..])),
        Some("keygen") => exit_with(cli::protocol::cmd_keygen(&args[2..])),
        Some("verify-signature") => exit_with(cli::protocol::cmd_verify_signature(&args[2..])),
        Some("harness") => exit_with(cli::protocol::cmd_harness(&args[2..])),
        // [#4537] 통합 판정은 **읽기 전용**이라 쓰기 명령(harness)과 표면을 나눈다 —
        // capabilities 의 category 가 도구 주석(readOnlyHint)의 교차 검증 원천이므로,
        // 한 명령이 쓰기·읽기를 겸하면 MCP 주석 계약이 성립하지 않는다.
        Some("harness-status") => exit_with(cli::protocol::cmd_harness_status(&args[2..])),
        Some("anchor") => exit_with(cli::protocol::cmd_anchor(&args[2..])),
        Some("gate") => exit_with(cli::protocol::cmd_gate(&args[2..])),
        Some("bundle") => exit_with(cli::protocol::cmd_bundle(&args[2..])),
        Some("disclose") => exit_with(cli::protocol::cmd_disclose(&args[2..])),
        Some("settle") => exit_with(cli::protocol::cmd_settle(&args[2..])),
        Some("audit-report") => exit_with(cli::protocol::cmd_audit_report(&args[2..])),
        Some("recall-scope") => exit_with(cli::protocol::cmd_recall_scope(&args[2..])),
        Some("conformance") => exit_with(cli::protocol::cmd_conformance(&args[2..])),
        // [#3719 §6-4] 계획을 *만드는* 쪽의 정답지 — `run` 바로 옆에 둔다.
        Some("export-plan-schema") => exit_with(cmd_export_plan_schema(&args[2..])),
        // [#2707] 알 수 없는 명령·명령 누락은 사용법 오류다. 표준 CLI 관례대로 stderr 로 안내하고
        // 종료 코드 2로 끝낸다(기존에는 stdout + 0이라 오타 낸 명령이 스크립트에서 성공으로 보였다).
        other => {
            // [#4220 T4] 수복 한 줄은 stderr 마지막 줄이어야 하므로(소비자는 마지막
            // `수복: ` 줄 하나만 파싱한다) 산문을 모두 낸 뒤에 방출한다. 두 부류만
            // 결정론적이다: 확신 교정(임계 내 오타)과 명령 누락(발견 경로는 언제나
            // capabilities). 임계 밖 오타는 수복 줄도 침묵한다 — 오제안 0.
            let recovery: Option<(String, &str)> = match other {
                Some(command) => {
                    eprintln!("오류: 알 수 없는 명령입니다 - {}", command);
                    // [#3694] did-you-mean — 후보는 capabilities 단일 출처. 이름 환각을
                    // 교정 단서 없이 돌려보내면 경량 에이전트는 맹목 재시도 루프에 빠진다.
                    let names = cli::metadata::capabilities::capabilities_command_names();
                    let hint = cli::metadata::capabilities::closest_name(
                        command,
                        names.iter().map(String::as_str),
                    );
                    if let Some(hint) = &hint {
                        eprintln!("힌트: 가장 가까운 명령은 '{hint}' 입니다");
                    }
                    hint.map(|h| (h, "요청한 이름이 없음 — 가장 가까운 실존 명령으로 교정"))
                }
                None => {
                    eprintln!("오류: 명령을 지정해주세요.");
                    Some((
                        "capabilities".to_string(),
                        "명령이 지정되지 않음 — 실행 가능한 명령 목록·계약은 capabilities 가 자기서술",
                    ))
                }
            };
            eprintln!("rhwp v{}", rhwp::version());
            eprintln!("사용법: rhwp <명령> [옵션]");
            eprintln!("'rhwp --help'로 자세한 사용법을 확인하세요.");
            if let Some((name, why)) = recovery {
                cli::metadata::capabilities::eprint_usage_recovery(&name, None, why);
            }
            process::exit(EXIT_USAGE);
        }
    }
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

/// 재로드 검증이 어긋났을 때 실제로 편집을 막을지 판단한다.
///
/// 이 검증은 직렬화 정합성을 보는 장치지만, 중첩 표를 최상위로 꺼내 고치고
/// 도로 넣는 왕복처럼 **중간 상태에서 쪽수가 늘어나는 것이 정상인** 작업까지
/// 함께 막는다. `RHWP_ALLOW_PAGE_DELTA` 가 설정돼 있으면 검증은 그대로 돌리되
/// 실패를 경고로 낮춘다.
fn page_verify_should_block(before: u32, after: u32) -> bool {
    let allowed = std::env::var("RHWP_ALLOW_PAGE_DELTA")
        .map(|v| !v.is_empty() && v != "0")
        .unwrap_or(false);
    if allowed {
        eprintln!(
            "경고: 재로드 후 쪽수가 {}→{} 로 달라졌으나 RHWP_ALLOW_PAGE_DELTA 로 통과시킴",
            before, after
        );
        return false;
    }
    true
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
    if !verification.recovered
        && page_verify_should_block(verification.page_count_before, verification.page_count_after)
    {
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
        core.split_paragraph_native(section_idx, para_idx, char_offset, None)
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
            None,
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

/// 중첩 표 셀 텍스트 삽입 (cellPath 경로형).
///
/// `--cell/--row/--col` 은 본문 문단 바로 아래 표 1단계만 지정할 수 있어
/// 표 안의 표(중첩 표) 셀에는 닿지 않는다. core 는 `*_by_path` 로 다단계
/// 경로를 이미 지원하므로 CLI 에서도 같은 경로를 받는다.
fn insert_hwp_cell_text_by_path_bytes_for_cli(
    data: &[u8],
    table_para_idx: usize,
    cell_path_json: &str,
    char_offset: usize,
    text: &str,
) -> Result<HwpEditCliResult, String> {
    let path = parse_cell_path_for_cli(cell_path_json)?;
    if path.is_empty() {
        return Err("--cell-path 가 비어 있습니다.".to_string());
    }
    edit_hwp_table_structure_bytes_for_cli(data, "insert-cell-text", |core| {
        let details = core
            .insert_text_in_cell_by_path(0, table_para_idx, &path, char_offset, text)
            .map_err(|e| format!("셀 텍스트 삽입 실패: {}", e))?;
        Ok(details)
    })
}

/// 중첩 표 셀 텍스트 삭제 (cellPath 경로형).
fn delete_hwp_cell_text_by_path_bytes_for_cli(
    data: &[u8],
    table_para_idx: usize,
    cell_path_json: &str,
    char_offset: usize,
    count: usize,
) -> Result<HwpEditCliResult, String> {
    let path = parse_cell_path_for_cli(cell_path_json)?;
    if path.is_empty() {
        return Err("--cell-path 가 비어 있습니다.".to_string());
    }
    edit_hwp_table_structure_bytes_for_cli(data, "delete-cell-text", |core| {
        let details = core
            .delete_text_in_cell_by_path(0, table_para_idx, &path, char_offset, count)
            .map_err(|e| format!("셀 텍스트 삭제 실패: {}", e))?;
        Ok(details)
    })
}

/// 중첩 표 셀 문단 나눔/합침 (cellPath 경로형).
fn cell_paragraph_by_path_bytes_for_cli(
    data: &[u8],
    table_para_idx: usize,
    cell_path_json: &str,
    char_offset: usize,
    merge: bool,
) -> Result<HwpEditCliResult, String> {
    let path = parse_cell_path_for_cli(cell_path_json)?;
    if path.is_empty() {
        return Err("--cell-path 가 비어 있습니다.".to_string());
    }
    let op = if merge {
        "merge-cell-paragraph"
    } else {
        "split-cell-paragraph"
    };
    edit_hwp_table_structure_bytes_for_cli(data, op, |core| {
        let details = if merge {
            core.merge_paragraph_in_cell_by_path(0, table_para_idx, &path)
        } else {
            core.split_paragraph_in_cell_by_path(0, table_para_idx, &path, char_offset, None)
        }
        .map_err(|e| format!("셀 문단 처리 실패: {}", e))?;
        Ok(details)
    })
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
                None,
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
                None,
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
    path: &[(usize, usize, usize)],
    updates_json: &str,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "resize-table-cells", |core| {
        core.resize_table_cells_by_path_native(section_idx, table_para_idx, path, updates_json)
            .map_err(|e| format!("표 셀 크기 조절 실패: {}", e))
    })
}

fn set_hwp_table_column_widths_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    table_para_idx: usize,
    control_idx: usize,
    widths: Vec<u32>,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "set-table-column-widths", |core| {
        core.set_table_column_widths_native(section_idx, table_para_idx, control_idx, widths.clone())
            .map_err(|e| format!("표 열 폭 설정 실패: {}", e))
    })
}

#[allow(clippy::too_many_arguments)]
fn apply_hwp_table_style_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    table_para_idx: usize,
    control_idx: usize,
    head_fill: String,
    font_size: u32,
    head_height: i32,
    body_height: i32,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "apply-table-style", |core| {
        core.apply_table_style_native(
            section_idx,
            table_para_idx,
            control_idx,
            &head_fill,
            font_size,
            head_height,
            body_height,
        )
        .map_err(|e| format!("표 서식 적용 실패: {}", e))
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

/// 중첩 표(표 안의 표) 셀 문단의 글자 서식. 경로는 [[표ctrl,셀,셀문단],…].
fn set_hwp_cell_char_format_by_path_bytes_for_cli(
    data: &[u8],
    section_idx: usize,
    parent_para_idx: usize,
    cell_path_json: &str,
    start_offset: usize,
    end_offset: usize,
    props_json: &str,
) -> Result<HwpEditCliResult, String> {
    let cell_path = parse_cell_path_for_cli(cell_path_json)?;
    if cell_path.is_empty() {
        return Err("cell-path는 비어 있을 수 없습니다.".to_string());
    }
    edit_hwp_table_structure_bytes_for_cli(data, "set-cell-char-format", |core| {
        core.apply_char_format_in_cell_by_path_native(
            section_idx,
            parent_para_idx,
            &cell_path,
            start_offset,
            end_offset,
            props_json,
        )
        .map_err(|e| format!("중첩 셀 글자 서식 설정 실패: {}", e))
    })
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
            None,
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

/// 본문 표 컨트롤을 답변 칸(셀) 문단으로 옮긴다 — 양식 채움 전용.
/// 사용법: rhwp move-table-to-cell <파일.hwp> --src-para N --src-control N
///         --dst-para N --dst-control N --dst-cell N --dst-cell-para N
///         [--offset N] -o <출력.hwp>
fn move_table_to_cell_cli(args: &[String]) {
    if args.is_empty() {
        exit_cli_error(
            "사용법: rhwp move-table-to-cell <파일.hwp> --src-para N --src-control N --dst-para N --dst-control N --dst-cell N --dst-cell-para N [--offset N] -o <출력.hwp>",
        );
    }
    let input = args[0].clone();
    let mut vals: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    let mut output_path: Option<String> = None;
    let mut i = 1;
    while i < args.len() {
        let key = args[i].as_str();
        match key {
            "--src-para" | "--src-control" | "--dst-para" | "--dst-control" | "--dst-cell"
            | "--dst-cell-para" | "--offset" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error(&format!("{} 뒤에 정수가 필요합니다.", key));
                }
                let v: usize = args[i]
                    .parse()
                    .unwrap_or_else(|_| exit_cli_error(&format!("{} 값이 정수가 아닙니다.", key)));
                vals.insert(key, v);
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
    let need = |k: &str| -> usize {
        *vals
            .get(k)
            .unwrap_or_else(|| exit_cli_error(&format!("{} 가 필요합니다.", k)))
    };
    let src_para = need("--src-para");
    let src_control = need("--src-control");
    let dst_para = need("--dst-para");
    let dst_control = need("--dst-control");
    let dst_cell = need("--dst-cell");
    let dst_cell_para = need("--dst-cell-para");
    let offset = vals.get("--offset").copied().unwrap_or(0);
    let output = output_path.unwrap_or_else(|| input.clone());

    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let mut core = rhwp::document_core::DocumentCore::from_bytes(&data)
        .unwrap_or_else(|e| exit_cli_error(&format!("HWP 파싱 실패: {}", e)));
    core.convert_to_editable_native()
        .unwrap_or_else(|e| exit_cli_error(&format!("편집 가능 변환 실패: {}", e)));
    let cell_path = [(dst_control, dst_cell, dst_cell_para)];
    let details_json = core
        .move_table_to_cell_native(0, src_para, src_control, dst_para, &cell_path, offset)
        .unwrap_or_else(|e| exit_cli_error(&format!("표 이동 실패: {}", e)));
    let verification = core
        .serialize_hwp_with_verify()
        .unwrap_or_else(|e| exit_cli_error(&format!("HWP 직렬화/재로드 검증 실패: {}", e)));
    if !verification.recovered {
        exit_cli_error(&format!(
            "HWP 재로드 검증 실패: page_count_before={}, page_count_after={}",
            verification.page_count_before, verification.page_count_after
        ));
    }
    write_hwp_cli_output(&output, &verification.bytes).unwrap_or_else(|e| exit_cli_error(&e));
    println!(
        "{}",
        serde_json::json!({
            "ok": true,
            "path": output,
            "details": parse_json_value(&details_json),
            "pageCountBefore": verification.page_count_before,
            "pageCountAfter": verification.page_count_after,
        })
    );
}

/// 한컴 호환성 lint — 저장은 되는데 한컴에서만 깨지는 결함의 사전 진단.
fn lint_cli(args: &[String]) {
    if args.is_empty() {
        exit_cli_error("사용법: rhwp lint <파일.hwp>");
    }
    let input = args[0].clone();
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let core = rhwp::document_core::DocumentCore::from_bytes(&data)
        .unwrap_or_else(|e| exit_cli_error(&format!("HWP 파싱 실패: {}", e)));
    let report = core
        .lint_native()
        .unwrap_or_else(|e| exit_cli_error(&format!("lint 실패: {}", e)));
    println!("{}", report);
}

/// 셀 안 중첩 표들의 tac 플래그·instance id 를 한컴 표준으로 수리한다.
fn repair_nested_tables_cli(args: &[String]) {
    if args.is_empty() {
        exit_cli_error("사용법: rhwp repair-nested-tables <파일.hwp> [-o <출력.hwp>]");
    }
    let input = args[0].clone();
    let mut output_path: Option<String> = None;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
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
    let output = output_path.unwrap_or_else(|| input.clone());
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let mut core = rhwp::document_core::DocumentCore::from_bytes(&data)
        .unwrap_or_else(|e| exit_cli_error(&format!("HWP 파싱 실패: {}", e)));
    core.convert_to_editable_native()
        .unwrap_or_else(|e| exit_cli_error(&format!("편집 가능 변환 실패: {}", e)));
    let details_json = core
        .repair_nested_table_attrs_native()
        .unwrap_or_else(|e| exit_cli_error(&format!("수리 실패: {}", e)));
    let verification = core
        .serialize_hwp_with_verify()
        .unwrap_or_else(|e| exit_cli_error(&format!("HWP 직렬화/재로드 검증 실패: {}", e)));
    write_hwp_cli_output(&output, &verification.bytes).unwrap_or_else(|e| exit_cli_error(&e));
    println!(
        "{}",
        serde_json::json!({
            "ok": true,
            "path": output,
            "details": parse_json_value(&details_json),
            "pageCountBefore": verification.page_count_before,
            "pageCountAfter": verification.page_count_after,
        })
    );
}

/// 셀 안 표를 본문으로 꺼낸다 — move-table-to-cell 의 역방향 (수선용).
fn move_table_from_cell_cli(args: &[String]) {
    if args.is_empty() {
        exit_cli_error(
            "사용법: rhwp move-table-from-cell <파일.hwp> --src-para N --src-control N --src-cell N --src-cell-para N -o <출력.hwp>",
        );
    }
    let input = args[0].clone();
    let mut vals: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    let mut output_path: Option<String> = None;
    let mut i = 1;
    while i < args.len() {
        let key = args[i].as_str();
        match key {
            "--src-para" | "--src-control" | "--src-cell" | "--src-cell-para" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error(&format!("{} 뒤에 정수가 필요합니다.", key));
                }
                let v: usize = args[i]
                    .parse()
                    .unwrap_or_else(|_| exit_cli_error(&format!("{} 값이 정수가 아닙니다.", key)));
                vals.insert(key, v);
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
    let need = |k: &str| -> usize {
        *vals
            .get(k)
            .unwrap_or_else(|| exit_cli_error(&format!("{} 가 필요합니다.", k)))
    };
    let src_para = need("--src-para");
    let src_control = need("--src-control");
    let src_cell = need("--src-cell");
    let src_cell_para = need("--src-cell-para");
    let output = output_path.unwrap_or_else(|| input.clone());

    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let mut core = rhwp::document_core::DocumentCore::from_bytes(&data)
        .unwrap_or_else(|e| exit_cli_error(&format!("HWP 파싱 실패: {}", e)));
    core.convert_to_editable_native()
        .unwrap_or_else(|e| exit_cli_error(&format!("편집 가능 변환 실패: {}", e)));
    let cell_path = [(src_control, src_cell, src_cell_para)];
    let details_json = core
        .move_table_from_cell_native(0, src_para, &cell_path)
        .unwrap_or_else(|e| exit_cli_error(&format!("표 꺼내기 실패: {}", e)));
    let verification = core
        .serialize_hwp_with_verify()
        .unwrap_or_else(|e| exit_cli_error(&format!("HWP 직렬화/재로드 검증 실패: {}", e)));
    write_hwp_cli_output(&output, &verification.bytes).unwrap_or_else(|e| exit_cli_error(&e));
    println!(
        "{}",
        serde_json::json!({
            "ok": true,
            "path": output,
            "details": parse_json_value(&details_json),
            "pageCountBefore": verification.page_count_before,
            "pageCountAfter": verification.page_count_after,
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
    let mut cell_path: Option<String> = None;
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
            "--cell-path" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--cell-path 뒤에 JSON 문자열이 필요합니다.");
                }
                cell_path = Some(args[i].clone());
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
    // --cell-path 는 경로에 ctrl/셀/셀문단이 모두 들어 있어 개별 인자가 필요 없다.
    let ctrl = if cell_path.is_some() {
        0
    } else {
        parse_usize_cli(ctrl, "--ctrl")
    };
    let cell_para = if cell_path.is_some() {
        0
    } else {
        parse_usize_cli(cell_para, "--cell-para")
    };
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
    } else if let Some(path_json) = cell_path.clone() {
        // 중첩 표 경로 지정 — --cell/--row/--col 대신 다단계 경로 사용
        if delete {
            let count = parse_usize_cli(count, "--count");
            delete_hwp_cell_text_by_path_bytes_for_cli(&data, para, &path_json, offset, count)
        } else {
            let text = read_optional_text_argument(inline_text, text_file)
                .unwrap_or_else(|e| exit_cli_error(&e))
                .unwrap_or_else(|| exit_cli_error("--text 또는 --text-file 값이 필요합니다."));
            insert_hwp_cell_text_by_path_bytes_for_cli(&data, para, &path_json, offset, &text)
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
    let mut cell_path: Option<String> = None;
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
            "--cell-path" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--cell-path 뒤에 JSON 문자열이 필요합니다.");
                }
                cell_path = Some(args[i].clone());
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
    let ctrl = if cell_path.is_some() { 0 } else { parse_usize_cli(ctrl, "--ctrl") };
    let cell_para = if cell_path.is_some() { 0 } else { parse_usize_cli(cell_para, "--cell-para") };
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
    } else if let Some(path_json) = cell_path.clone() {
        // 중첩 표 경로 지정
        let split_offset = if merge {
            0
        } else {
            parse_usize_cli(offset, "--offset")
        };
        cell_paragraph_by_path_bytes_for_cli(&data, para, &path_json, split_offset, merge)
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

fn move_hwp_cell_paragraphs_bytes_for_cli(
    data: &[u8],
    table_para_idx: usize,
    control_idx: usize,
    src: (u16, u16),
    start: usize,
    end: usize,
    dst: (u16, u16),
    dst_at: usize,
) -> Result<HwpEditCliResult, String> {
    edit_hwp_table_structure_bytes_for_cli(data, "move-cell-paragraphs", |core| {
        let src_idx = core
            .get_table_cell_index_native(0, table_para_idx, control_idx, src.0, src.1)
            .map_err(|e| format!("출발 셀 좌표 조회 실패: {}", e))?;
        let dst_idx = core
            .get_table_cell_index_native(0, table_para_idx, control_idx, dst.0, dst.1)
            .map_err(|e| format!("도착 셀 좌표 조회 실패: {}", e))?;
        core.move_cell_paragraphs_native(
            0,
            table_para_idx,
            control_idx,
            src_idx,
            start,
            end,
            dst_idx,
            dst_at,
        )
        .map_err(|e| format!("셀 문단 이동 실패: {}", e))
    })
}

fn move_cell_paragraphs_cli(args: &[String]) {
    const USAGE: &str = "사용법: rhwp move-cell-paragraphs <파일.hwp> --para N --ctrl N --from-row R --from-col C --start N --end N --to-row R --to-col C [--at N] -o <출력.hwp>";
    if args.is_empty() {
        exit_cli_error(USAGE);
    }
    let input = args[0].clone();
    let mut para: Option<String> = None;
    let mut ctrl: Option<String> = None;
    let mut from_row: Option<String> = None;
    let mut from_col: Option<String> = None;
    let mut to_row: Option<String> = None;
    let mut to_col: Option<String> = None;
    let mut start: Option<String> = None;
    let mut end: Option<String> = None;
    let mut at: Option<String> = None;
    let mut output_path: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        let need = |i: usize, flag: &str| -> String {
            if i >= args.len() {
                exit_cli_error(&format!("{} 뒤에 값이 필요합니다.", flag));
            }
            args[i].clone()
        };
        match args[i].as_str() {
            "--para" => {
                i += 1;
                para = Some(need(i, "--para"));
            }
            "--ctrl" => {
                i += 1;
                ctrl = Some(need(i, "--ctrl"));
            }
            "--from-row" => {
                i += 1;
                from_row = Some(need(i, "--from-row"));
            }
            "--from-col" => {
                i += 1;
                from_col = Some(need(i, "--from-col"));
            }
            "--to-row" => {
                i += 1;
                to_row = Some(need(i, "--to-row"));
            }
            "--to-col" => {
                i += 1;
                to_col = Some(need(i, "--to-col"));
            }
            "--start" => {
                i += 1;
                start = Some(need(i, "--start"));
            }
            "--end" => {
                i += 1;
                end = Some(need(i, "--end"));
            }
            "--at" => {
                i += 1;
                at = Some(need(i, "--at"));
            }
            "-o" | "--output" => {
                i += 1;
                output_path = Some(need(i, "-o/--output"));
            }
            _ => exit_cli_error(&format!("알 수 없는 옵션: {}", args[i])),
        }
        i += 1;
    }

    let para = parse_usize_cli(para, "--para");
    let ctrl = parse_usize_cli(ctrl, "--ctrl");
    let from_row = parse_u16_cli(from_row, "--from-row");
    let from_col = parse_u16_cli(from_col, "--from-col");
    let to_row = parse_u16_cli(to_row, "--to-row");
    let to_col = parse_u16_cli(to_col, "--to-col");
    let start = parse_usize_cli(start, "--start");
    let end = parse_usize_cli(end, "--end");
    let at = at
        .map(|v| {
            v.parse::<usize>()
                .unwrap_or_else(|_| exit_cli_error("--at 은 0 이상의 정수여야 합니다."))
        })
        .unwrap_or(usize::MAX);
    let output = output_path.unwrap_or_else(|| input.clone());
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));

    let result = move_hwp_cell_paragraphs_bytes_for_cli(
        &data,
        para,
        ctrl,
        (from_row, from_col),
        start,
        end,
        (to_row, to_col),
        at,
    )
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

/// 셀 안 문단의 글자 수와 텍스트를 조회한다 (중첩 표 포함).
///
/// 편집 명령의 `--offset` / `--count` 를 정확히 계산하기 위한 읽기 전용
/// 명령. 길이를 눈대중으로 넘겨 옆 문단이 잘려 나가는 사고를 막는다.
fn get_cell_text_cli(args: &[String]) {
    if args.is_empty() {
        exit_cli_error(
            "사용법: rhwp get-cell-text <파일.hwp> --para N [--ctrl N --cell N | --cell-path <JSON>]",
        );
    }
    let input = args[0].clone();
    let mut para: Option<String> = None;
    let mut ctrl: Option<String> = None;
    let mut cell: Option<String> = None;
    let mut cell_path: Option<String> = None;

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
            "--cell-path" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--cell-path 뒤에 JSON 문자열이 필요합니다.");
                }
                cell_path = Some(args[i].clone());
            }
            other => exit_cli_error(&format!("알 수 없는 옵션: {}", other)),
        }
        i += 1;
    }

    let para = parse_usize_cli(para, "--para");
    let data = std::fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));

    // --cell-path 가 없으면 --ctrl/--cell 로 1 단계 경로를 만든다.
    let path_json = match cell_path {
        Some(p) => p,
        None => {
            let ctrl = parse_usize_cli(ctrl, "--ctrl");
            let cell = parse_usize_cli(cell, "--cell");
            format!("[[{},{},0]]", ctrl, cell)
        }
    };
    let path = parse_cell_path_for_cli(&path_json).unwrap_or_else(|e| exit_cli_error(&e));

    let core = rhwp::document_core::DocumentCore::from_bytes(&data)
        .unwrap_or_else(|e| exit_cli_error(&format!("HWP 파싱 실패: {}", e)));
    let out = core
        .get_cell_paragraphs_by_path(0, para, &path)
        .unwrap_or_else(|e| exit_cli_error(&format!("셀 문단 조회 실패: {}", e)));
    println!("{}", out);
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
        exit_cli_error("사용법: rhwp resize-table-cells <파일.hwp> --section N --para N [--cell-path <JSON>|--ctrl N] --json <변경배열JSON> -o <출력.hwp>");
    }
    let input = args[0].clone();
    let mut section: Option<String> = None;
    let mut para: Option<String> = None;
    let mut ctrl: Option<String> = None;
    let mut cell_path: Option<String> = None;
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
                    exit_cli_error("--cell-path 뒤에 JSON 경로가 필요합니다.");
                }
                cell_path = Some(args[i].clone());
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
    // --cell-path 를 주면 중첩 표를, 안 주면 --ctrl 로 최상위 표를 가리킨다.
    let path = match &cell_path {
        Some(json) => parse_cell_path_for_cli(json).unwrap_or_else(|e| exit_cli_error(&e)),
        None => vec![(parse_usize_cli(ctrl, "--ctrl"), 0, 0)],
    };
    if path.is_empty() {
        exit_cli_error("--cell-path 는 비어 있을 수 없습니다.");
    }
    let updates_json =
        read_json_argument(inline_json, json_file).unwrap_or_else(|e| exit_cli_error(&e));
    let output = output_path.unwrap_or_else(|| input.clone());
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let result = resize_hwp_table_cells_bytes_for_cli(&data, section, para, &path, &updates_json)
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

fn set_table_column_widths_cli(args: &[String]) {
    if args.is_empty() {
        exit_cli_error("사용법: rhwp set-table-column-widths <파일.hwp> --section N --para N --ctrl N --widths w1,w2,... -o <출력.hwp>");
    }
    let input = args[0].clone();
    let mut section: Option<String> = None;
    let mut para: Option<String> = None;
    let mut ctrl: Option<String> = None;
    let mut widths_arg: Option<String> = None;
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
            "--widths" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--widths 뒤에 쉼표로 구분한 열 폭이 필요합니다.");
                }
                widths_arg = Some(args[i].clone());
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
    let widths: Vec<u32> = widths_arg
        .unwrap_or_else(|| exit_cli_error("--widths 가 필요합니다."))
        .split(',')
        .map(|t| {
            t.trim()
                .parse::<u32>()
                .unwrap_or_else(|_| exit_cli_error("--widths 는 쉼표로 구분한 양의 정수여야 합니다."))
        })
        .collect();
    if widths.is_empty() {
        exit_cli_error("--widths 가 비어 있습니다.");
    }
    let output = output_path.unwrap_or_else(|| input.clone());
    let data = fs::read(&input)
        .unwrap_or_else(|e| exit_cli_error(&format!("파일 읽기 실패 - {}: {}", input, e)));
    let result = set_hwp_table_column_widths_bytes_for_cli(&data, section, para, ctrl, widths)
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

/// 표에 심사 문서용 「집 서식」을 한 번에 입힌다.
///
/// 기본값은 배포된 「작성 예시」(중기부 Part2)를 실측해 정한 값이다 —
/// 머리행 음영 `#d9d9d9`, 12pt, 머리행 높이 2232 / 본문 행 높이 2882 HWPU.
fn apply_table_style_cli(args: &[String]) {
    if args.is_empty() {
        exit_cli_error("사용법: rhwp apply-table-style <파일.hwp> --section N --para N --ctrl N [--head-fill #d9d9d9] [--font-size 1200] [--head-height 2232] [--body-height 2882] -o <출력.hwp>");
    }
    let input = args[0].clone();
    let mut section: Option<String> = None;
    let mut para: Option<String> = None;
    let mut ctrl: Option<String> = None;
    let mut head_fill = "#d9d9d9".to_string();
    let mut font_size: u32 = 1200;
    let mut head_height: i32 = 2232;
    let mut body_height: i32 = 2882;
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
            "--head-fill" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--head-fill 뒤에 색(#rrggbb)이 필요합니다.");
                }
                head_fill = args[i].clone();
            }
            "--font-size" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--font-size 뒤에 값(HWPUNIT, 12pt=1200)이 필요합니다.");
                }
                font_size = args[i]
                    .trim()
                    .parse::<u32>()
                    .unwrap_or_else(|_| exit_cli_error("--font-size 는 양의 정수여야 합니다."));
            }
            "--head-height" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--head-height 뒤에 값(HWPUNIT)이 필요합니다.");
                }
                head_height = args[i]
                    .trim()
                    .parse::<i32>()
                    .unwrap_or_else(|_| exit_cli_error("--head-height 는 정수여야 합니다."));
            }
            "--body-height" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--body-height 뒤에 값(HWPUNIT)이 필요합니다.");
                }
                body_height = args[i]
                    .trim()
                    .parse::<i32>()
                    .unwrap_or_else(|_| exit_cli_error("--body-height 는 정수여야 합니다."));
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
    let result = apply_hwp_table_style_bytes_for_cli(
        &data,
        section,
        para,
        ctrl,
        head_fill,
        font_size,
        head_height,
        body_height,
    )
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
    let mut cell_path: Option<String> = None;
    let mut start: Option<String> = None;
    let mut end: Option<String> = None;
    let mut inline_json: Option<String> = None;
    let mut json_file: Option<String> = None;
    let mut output_path: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--cell-path" => {
                i += 1;
                if i >= args.len() {
                    exit_cli_error("--cell-path 뒤에 JSON 이 필요합니다.");
                }
                cell_path = Some(args[i].clone());
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
        "cell-char" if cell_path.is_some() => {
            // 중첩 표 셀: 경로가 바깥 표 ctrl 부터 담으므로 --ctrl/--cell 과 같이 쓰지 않는다.
            if ctrl.is_some() || cell.is_some() || row.is_some() || col.is_some() {
                exit_cli_error("--cell-path 는 --ctrl/--cell/--row/--col 과 함께 사용할 수 없습니다.");
            }
            let start = parse_usize_cli(start, "--start");
            let end = parse_usize_cli(end, "--end");
            set_hwp_cell_char_format_by_path_bytes_for_cli(
                &data,
                section,
                para,
                cell_path.as_deref().unwrap_or_default(),
                start,
                end,
                &props_json,
            )
        }
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
    // 미지정 시 0 을 넘겨 core 가 이미지 헤더에서 원본 픽셀 크기를 판독하게 한다.
    // 예전 기본값 (width/75) 은 "원본 = 표시 크기" 가정이라, 실제로는 이미지의
    // 좌상단 일부만 crop 되어 확대 표시됐다 (1895x830 PNG 를 41100x18000 으로
    // 넣으면 좌상단 29% 만 보임).
    let natural_width = natural_width
        .map(|v| parse_u32_cli(Some(v), "--natural-width"))
        .unwrap_or(0);
    let natural_height = natural_height
        .map(|v| parse_u32_cli(Some(v), "--natural-height"))
        .unwrap_or(0);
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


// [#5511] 최상위 dispatch 끝 — 소유 모듈 이동과 무관한 characterization 경계다.

/// [#3346] `export-tables --json` 과 `batch export-tables` 가 공유하는 봉투.
fn tables_json_value(
    file_path: &str,
    tables: &[rhwp::document_core::queries::table_extract::TableGrid],
) -> serde_json::Value {
    provenance::marked(
        serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "source": file_path,
            "tableCount": tables.len(),
            "tables": tables,
        }),
        "export-tables",
    )
}

/// [#3346] `fields --json` 과 `batch fields` 가 공유하는 봉투.
fn fields_json_value(file_path: &str, fields: &[serde_json::Value]) -> serde_json::Value {
    let names: Vec<String> = fields
        .iter()
        .filter_map(|f| f["name"].as_str().map(String::from))
        .collect();
    provenance::marked(
        serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "source": file_path,
            "fieldCount": fields.len(),
            "fields": fields,
            "textSecurity": text_security_value(&names),
        }),
        "fields",
    )
}

/// 누름틀 이름 축의 유니코드 기만 판정 봉투.
///
/// 봉투에 담기는 이름은 **공격자가 내용을 정할 수 있는 문서**에서 온다. 에이전트는
/// 그 이름으로 "이 칸을 채워라"를 지목하므로, 화면상 같지만 바이트가 다른 이름 쌍이
/// 있으면 엉뚱한 칸이 채워지고도 `filledCount` 는 성공을 보고한다(#3707).
///
/// 판정만 하고 이름을 고치지 않는다 — 문서 엔진이 사용자 문자열을 조용히 바꾸는 것은
/// 어떤 보안 이득으로도 정당화되지 않는다. `status` 는 `clean`/`warning` 2단이고,
/// 항상 실려 나간다: 필드가 없으면 `clean`, 옛 바이너리면 키 자체가 없다 —
/// 소비자가 "검사했는데 깨끗함"과 "검사하지 않음"을 구별할 수 있어야 한다.
fn text_security_value(names: &[String]) -> serde_json::Value {
    use rhwp::document_core::text_security as ts;

    let mut findings: Vec<serde_json::Value> = Vec::new();

    // ① 화면상 같은 이름 쌍 — 실제 공격 서명이다.
    for (_, group) in ts::confusable_collisions(names) {
        findings.push(serde_json::json!({
            "kind": "confusableFieldName",
            "scope": "fieldName",
            "names": group,
            "note": "이름이 화면상 구별되지 않는 누름틀이 둘 이상입니다 — 이름으로 지목해 채우면 의도와 다른 칸이 채워질 수 있습니다. occurrence 대신 hwp_fields 가 돌려준 바이트를 그대로 쓰거나, 사람 확인을 거치세요.",
        }));
    }

    // ② 이름 하나하나의 혼합 스크립트·보이지 않는 문자.
    for name in names {
        for risk in ts::scan_identifier(name) {
            findings.push(serde_json::json!({
                "kind": risk.kind.label(),
                "scope": "fieldName",
                "names": [name],
                "codepoints": risk.codepoints.iter().map(|c| ts::format_codepoint(*c))
                    .collect::<Vec<_>>(),
                "note": risk.kind.describe(),
            }));
        }
    }

    if findings.is_empty() {
        return serde_json::json!({ "status": "clean" });
    }
    serde_json::json!({
        "status": "warning",
        "findingCount": findings.len(),
        "findings": findings,
    })
}

/// [#3346] `search --json` 과 `batch search` 가 공유하는 봉투.
fn search_json_value(
    file_path: &str,
    query: &str,
    case_sensitive: bool,
    matches: &[rhwp::document_core::queries::grep::GrepMatch],
    total_match_count: usize,
) -> serde_json::Value {
    provenance::marked(
        serde_json::json!({
        "schemaVersion": ENVELOPE_SCHEMA_VERSION,
        "source": file_path,
        "query": query,
        "caseSensitive": case_sensitive,
        "matchCount": matches.len(),
        "totalMatchCount": total_match_count,
        "truncated": matches.len() < total_match_count,
        // [#3787 S7] 절단 축의 어휘를 텍스트 축(`export-text --max-chars`)과 맞춘다.
        // `totalMatchCount - matchCount` 로 유도할 수 있는 값이지만, 유도를 요구하면
        // "전부 봤다"는 오독이 그대로 남는다 — 생략량은 명시가 계약이다.
        "omittedCount": total_match_count.saturating_sub(matches.len()),
        "matches": matches,
        }),
        "search",
    )
}

/// [#3787 S7] 페이지 텍스트 산출의 문자 예산 절단 — CLI `export-text --json` 과
/// MCP `hwp_doc_text` 가 같은 규칙을 공유한다.
///
/// **조용히 자르지 않는다.** 거대 문서가 에이전트 컨텍스트를 밀어내는 것을 막는 게
/// 목적이지만, 잘랐다는 사실을 숨기면 그 절단이 "전부 읽었다"는 거짓말이 된다.
/// 그래서 두 가지를 지킨다.
///
/// 1. **쪽 주소를 보존한다** — 예산이 떨어져도 `pages[]` 에서 항목을 빼지 않는다.
///    빼면 `pageCount` 가 줄어 문서가 실제보다 짧아 보인다.
/// 2. **생략량을 남긴다** — 잘린 페이지마다 `truncated:true`·`omittedCount`(생략된
///    문자 수)를 싣고, 봉투 최상위에 합계를 싣는다. 최상위 `truncated` 는 절단이
///    없어도 항상 나가고(false), 페이지 항목의 두 필드는 잘린 페이지에만 붙는다.
///
/// `max_chars` 가 `None` 이면 무제한이다(기본값 — 종전 동작 무변경).
fn truncate_page_texts(
    pages: &[(u32, String)],
    max_chars: Option<usize>,
) -> (Vec<serde_json::Value>, usize) {
    let mut objs = Vec::with_capacity(pages.len());
    let mut budget = max_chars;
    let mut omitted_total = 0usize;
    for (page, text) in pages {
        let total = text.chars().count();
        let keep = match budget {
            Some(remaining) => remaining.min(total),
            None => total,
        };
        if let Some(remaining) = budget.as_mut() {
            *remaining -= keep;
        }
        let omitted = total - keep;
        omitted_total += omitted;
        let kept: String = if omitted == 0 {
            text.clone()
        } else {
            text.chars().take(keep).collect()
        };
        let mut obj = serde_json::json!({ "page": page, "text": kept });
        if omitted > 0 {
            obj["truncated"] = serde_json::json!(true);
            obj["omittedCount"] = serde_json::json!(omitted);
        }
        objs.push(obj);
    }
    (objs, omitted_total)
}

/// [#3407] `title` 이 훑는 앞쪽 페이지 수 상한 — 표지가 이미지·빈 쪽인 문서의
/// fallback 범위. digest 발췌(`DIGEST_EXCERPT_PAGES`)와 같은 "앞 3쪽" 어휘를 쓴다.
const TITLE_SCAN_PAGES: u32 = 3;

/// [#3407] 문서 제목 best-effort 추출 — 대량 아카이브 1-pass 대장화용.
///
/// 렌더된 페이지 텍스트(`extract_page_text_native`, `export-text --json` 과 같은
/// 원천)의 첫 의미 줄(trim 후 비어있지 않은 첫 줄)을 돌려준다. 종전 2-pass
/// 대장화(`batch info` + 문서별 `export-text` 첫 줄 파싱)가 소비자 쪽에서 하던
/// 규칙을 엔진이 한 번만 정의한다. 표지가 이미지라 첫 쪽 텍스트가 비면 다음
/// 쪽으로 내려가며(앞 `TITLE_SCAN_PAGES` 쪽까지), 그래도 없으면 `None`(JSON
/// null)이다. 값 자체는 계약이 아닌 best-effort 필드이고, 추출 실패도 문서
/// 메타 조회를 막지 않도록 조용히 다음 쪽으로 넘어간다.
fn document_title(doc: &rhwp::wasm_api::HwpDocument) -> Option<String> {
    for page in 0..doc.page_count().min(TITLE_SCAN_PAGES) {
        let Ok(text) = doc.extract_page_text_native(page) else {
            continue;
        };
        if let Some(line) = text.lines().map(str::trim).find(|l| !l.is_empty()) {
            return Some(line.to_string());
        }
    }
    None
}

/// [#3237] `info --json`·`batch info --json` 이 공유하는 문서 메타 JSON 레코드.
/// `schemaVersion` 이 계약이며 필드 추가는 허용, 변경·삭제는 계약 테스트가 잡는다.
fn info_json_value(
    file_path: &str,
    file_size: usize,
    detected_format: rhwp::parser::FileFormat,
    doc: &rhwp::wasm_api::HwpDocument,
) -> serde_json::Value {
    let document = doc.document();
    let format_str = match detected_format {
        rhwp::parser::FileFormat::Hwp => "hwp5",
        rhwp::parser::FileFormat::Hwpx => "hwpx",
        rhwp::parser::FileFormat::Hwp3 => "hwp3",
        rhwp::parser::FileFormat::Hml => "hml",
        // 파싱이 성공한 뒤에는 도달하지 않지만, 계약상 문자열은 고정해 둔다.
        rhwp::parser::FileFormat::DrmProtected => "drm-protected",
        rhwp::parser::FileFormat::Empty => "empty",
        rhwp::parser::FileFormat::Unknown => "unknown",
    };
    let version = if detected_format == rhwp::parser::FileFormat::Hml {
        serde_json::Value::Null
    } else {
        serde_json::Value::String(format!(
            "{}.{}.{}.{}",
            document.header.version.major,
            document.header.version.minor,
            document.header.version.build,
            document.header.version.revision,
        ))
    };
    // DOCINFO는 한글·영어·한자·일어·기타·기호·사용자 글꼴군을 따로 보관한다.
    // `info --json`은 문서 인벤토리 용도이므로 첫 번째(한글) 군만이 아니라 선언된
    // 모든 글꼴군을 문서 순서대로 평탄화해 내보낸다. 같은 이름이 여러 군에 있으면
    // 소비자가 출처별 필요에 따라 중복을 보존하거나 제거할 수 있게 그대로 남긴다.
    let fonts: Vec<String> = document
        .doc_info
        .font_faces
        .iter()
        .flatten()
        .map(|face| face.name.clone())
        .collect();
    let para_count: usize = document.sections.iter().map(|s| s.paragraphs.len()).sum();
    let last_saved_with = match detected_format {
        rhwp::parser::FileFormat::Hwp => {
            rhwp::parser::hwp_summary::last_saved_with(&document.extra_streams)
        }
        rhwp::parser::FileFormat::Hwpx => {
            rhwp::parser::hwp_summary::hwpx_last_saved_with(&document.hwpx_aux_entries)
        }
        _ => None,
    }
    .map(|save_version| {
        serde_json::json!({
            "product": save_version.product,
            "version": save_version.version,
            "confidence": "metadata",
        })
    });
    provenance::marked(
        serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "source": file_path,
            "format": format_str,
            "sizeBytes": file_size,
            "version": version,
            "sections": document.sections.len(),
            "pageCount": doc.page_count(),
            "paraCount": para_count,
            "fonts": fonts,
            // [#3407] best-effort 문서 제목 — 없으면 null. batch info 로 자동 전파.
            "title": document_title(doc),
            // HWP5 summary 또는 HWPX version.xml 메타데이터다. 원 작성 제품이 아니라
            // 마지막 저장 제품을 가리키며 없거나 수정될 수 있다.
            "lastSavedWith": last_saved_with,
            // [#6208] 문서에 실린 인쇄 방식(모아 찍기 등). rhwp 는 이 값을 **출력에
            // 반영하지 않으므로**, 한글 오라클 PDF 와 대조할 때 `impliesNup` 이 true
            // 면 한글 쪽 장 수·용지 방향이 달라 좌표를 그대로 견주면 오판한다.
            // 값이 문서에 없으면 `printMethod: null`.
            "printMethod": doc.document().doc_info.print_method,
            "printMethodImpliesNup": rhwp::model::document::print_method_implies_nup(
                doc.document().doc_info.print_method,
            ),
            // [#3880 T1] 파싱 중 건너뛴 것을 봉투가 스스로 밝힌다.
            //
            // 인간 출력은 `warnings: N` 과 상세를 stderr 로 내는데 JSON 분기는 그
            // 앞에서 `return EXIT_OK` 로 끝나 도달하지 못했다. 그래서 리소스가 조용히
            // 잘린 문서가 **exit 0 + 완전해 보이는 봉투**를 냈다 — `fonts` 가 부분
            // 목록인데 봉투는 그렇다고 말하지 않았다(#3719 "부분 목록 금지" 위반).
            //
            // 경고가 없으면 빈 배열이다. 키를 빼면 소비자가 "경고 없음"과 "이 빌드는
            // 경고를 모름"을 구별할 수 없다.
            "warnings": info_warnings_value(doc),
        }),
        "info",
    )
}

/// [#3880 T1] `info --json` 의 `warnings[]` — 파싱이 건너뛴 것의 기계 판정용.
///
/// 현재 원천은 HML 파서의 `hml_metadata().warnings` 하나다. 다른 포맷이 같은 기구를
/// 갖추면 여기에 합류시킨다 — 그때까지 이 배열이 비어 있다고 해서 "문서가 온전하다"는
/// 뜻은 아니며, 그 한계는 `mydocs/manual/cli_commands.md` 에 적는다.
fn info_warnings_value(doc: &rhwp::wasm_api::HwpDocument) -> serde_json::Value {
    let Some(metadata) = doc.hml_metadata() else {
        return serde_json::Value::Array(Vec::new());
    };
    serde_json::Value::Array(
        metadata
            .warnings
            .iter()
            .map(|w| {
                serde_json::json!({
                    "code": format!("{:?}", w.code),
                    "xmlPath": w.xml_path,
                    "message": w.message,
                })
            })
            .collect(),
    )
}

/// [#3719 §6-10] `extract-data --json` 봉투.
///
/// `counts` 는 **요청한 종류에 대한 문서 전체 건수**다(`--limit` 절단 전). 요청하지 않은
/// 종류의 키는 아예 넣지 않는다 — `--kind date` 인데 `"amount": 0` 이 보이면 "금액이 없다"로
/// 오독되기 때문이다. `itemCount` 는 실제 반환된 건수이고, `totalItemCount`·`truncated` 가
/// 절단 사실을 드러낸다(#3353 의 `search` 와 같은 어휘).
fn extract_data_json_value(
    file_path: &str,
    kind: &str,
    items: &[rhwp::document_core::queries::extract_data::DataItem],
    total_item_count: usize,
    counts: &serde_json::Value,
) -> serde_json::Value {
    provenance::marked(
        serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "source": file_path,
            "kind": kind,
            "itemCount": items.len(),
            "totalItemCount": total_item_count,
            "truncated": items.len() < total_item_count,
            "counts": counts,
            "items": items,
        }),
        "extract-data",
    )
}

#[derive(Debug, Default, Clone, Copy)]
struct ConversionVerifyOptions {
    verify: bool,
    verify_pages: bool,
    /// [#3596] 봉투를 stdout 순수 JSON 으로. export-hwpx 만 허용한다(`allow_json`).
    json: bool,
}

impl ConversionVerifyOptions {
    fn enabled(self) -> bool {
        self.verify || self.verify_pages
    }
}

fn paths_refer_to_same_file(input: &Path, output: &Path) -> bool {
    input == output
        || paths_have_same_file_identity(input, output)
        || match (input.canonicalize(), output.canonicalize()) {
            (Ok(input), Ok(output)) => input == output,
            _ => false,
        }
}

#[cfg(unix)]
fn paths_have_same_file_identity(input: &Path, output: &Path) -> bool {
    use std::os::unix::fs::MetadataExt;

    match (input.metadata(), output.metadata()) {
        (Ok(input), Ok(output)) => input.dev() == output.dev() && input.ino() == output.ino(),
        _ => false,
    }
}

#[cfg(not(unix))]
fn paths_have_same_file_identity(_input: &Path, _output: &Path) -> bool {
    false
}

/// 옵션을 받지 않는 내부 개발 명령의 위치 인자를 엄격히 검증한다.
///
/// 이 명령들은 capabilities 에도 노출되어 있다. 플래그처럼 보이는 값을 위치 인자로
/// 삼키거나 여분 인자를 무시하면, 호출자는 오타 난 자동화를 성공으로 오인한다.
fn validate_internal_positionals(command: &str, args: &[String], max: usize) -> Result<(), i32> {
    if let Some(flag) = args.iter().find(|arg| arg.starts_with('-')) {
        eprintln!("오류: {command} 은 알 수 없는 옵션을 받지 않습니다 - {flag}");
        return Err(EXIT_USAGE);
    }
    if args.len() > max {
        eprintln!("오류: {command} 은 위치 인자를 최대 {max}개만 받습니다.");
        return Err(EXIT_USAGE);
    }
    Ok(())
}

fn test_shape_roundtrip(args: &[String]) -> i32 {
    if let Err(code) = validate_internal_positionals("test-shape", args, 2) {
        return code;
    }
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
            return EXIT_RUNTIME;
        }
    };

    let mut doc = match rhwp::wasm_api::HwpDocument::from_bytes(&data) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("HWP 파싱 오류: {:?}", e);
            return EXIT_RUNTIME;
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
            return EXIT_RUNTIME;
        }
    }

    match doc.export_hwp_native() {
        Ok(bytes) => {
            if let Err(e) = fs::write(output, &bytes) {
                eprintln!("파일 저장 오류: {}", e);
                return EXIT_RUNTIME;
            }
            eprintln!("저장 완료: {} ({}KB)", output, bytes.len() / 1024);
            EXIT_OK
        }
        Err(e) => {
            eprintln!("직렬화 오류: {:?}", e);
            EXIT_RUNTIME
        }
    }
}

fn gen_table(args: &[String]) -> i32 {
    if let Err(code) = validate_internal_positionals("gen-table", args, 3) {
        return code;
    }
    let rows = match args.first() {
        Some(value) => match value.parse::<u16>() {
            Ok(value) => value,
            Err(_) => {
                eprintln!("오류: gen-table 행 수는 0~65535 정수여야 합니다 - {value}");
                return EXIT_USAGE;
            }
        },
        None => 1000,
    };
    let cols = match args.get(1) {
        Some(value) => match value.parse::<u16>() {
            Ok(value) => value,
            Err(_) => {
                eprintln!("오류: gen-table 열 수는 0~65535 정수여야 합니다 - {value}");
                return EXIT_USAGE;
            }
        },
        None => 6,
    };
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
    if let Err(e) = fs::write(out_path, bytes) {
        // 종료 코드 계약: 쓰기 실패는 런타임 오류(1)다. 종전에는 .expect() 로 패닉해
        // 계약에 없는 101 로 끝났다.
        eprintln!("오류: 파일 저장 실패 - {}: {}", output, e);
        return EXIT_RUNTIME;
    }
    println!("저장 완료: {} ({}행 × {}열)", output, rows, cols);
    EXIT_OK
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
fn gen_pua_test(args: &[String]) -> i32 {
    if let Err(code) = validate_internal_positionals("gen-pua", args, 1) {
        return code;
    }
    // gen-pua 의 positional 은 입력이 아니라 **출력** 경로다. capabilities 가 다른
    // 진단 명령과 나란히 노출하는 탓에 `rhwp gen-pua 문서.hwp` 를 "이 파일을 조사"로
    // 읽은 호출이 실제로 원본을 말없이 덮어썼다(#3691 조사 중 발생). 사용자가 명시한
    // 경로가 이미 있으면 거부한다 — 기본 경로는 재생성 대상이라 검사에서 제외한다.
    let explicit = args.first().map(|s| s.as_str());
    if let Some(path) = explicit {
        if Path::new(path).exists() {
            eprintln!("오류: gen-pua 의 인자는 생성할 **출력** 경로입니다 (입력 파일이 아닙니다).");
            eprintln!("      이미 존재하는 파일을 덮어쓰지 않습니다: {}", path);
            eprintln!("사용법: rhwp gen-pua [출력경로]   # 기본 output/pua-test.hwp");
            return EXIT_USAGE;
        }
    }
    let output = explicit.unwrap_or("output/pua-test.hwp");

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
    if let Err(e) = fs::write(out_path, bytes) {
        // 종료 코드 계약: 쓰기 실패는 런타임 오류(1)다. 종전에는 .expect() 로 패닉해
        // 계약에 없는 101 로 끝났다.
        eprintln!("오류: 파일 저장 실패 - {}: {}", output, e);
        return EXIT_RUNTIME;
    }
    println!("저장 완료: {} ({} 종 PUA)", output, pua_set.len());
    println!();
    println!("다음 단계:");
    println!("  1. 한컴 2022 편집기에서 본 파일 열기 → PDF 출력 (정답지)");
    println!("  2. rhwp export-svg {} → SVG 출력 비교", output);
    println!("  3. 시각 비교로 매핑 정합 확정");
    EXIT_OK
}

/// [#3346] `fields --json` 과 `batch fields` 가 공유하는 필드 레코드 수집.
///
/// 단건/배치가 같은 스키마를 내도록 한 곳에서 만든다.
pub(crate) fn collect_field_records(doc: &rhwp::wasm_api::HwpDocument) -> Vec<serde_json::Value> {
    use rhwp::document_core::queries::field_query::NestedEntry;

    doc.collect_all_fields()
        .iter()
        .map(|fi| {
            // 중첩 경로: 표 셀·글상자 안의 필드가 어디에 있는지 — 후속 편집의 좌표다.
            let nested: Vec<serde_json::Value> = fi
                .location
                .nested_path
                .iter()
                .map(|e| match e {
                    NestedEntry::TableCell {
                        control_index,
                        cell_index,
                        para_index,
                    } => serde_json::json!({
                        "kind": "tableCell",
                        "control": control_index,
                        "cell": cell_index,
                        "paragraph": para_index,
                    }),
                    NestedEntry::TextBox {
                        control_index,
                        para_index,
                    } => serde_json::json!({
                        "kind": "textBox",
                        "control": control_index,
                        "paragraph": para_index,
                    }),
                })
                .collect();

            serde_json::json!({
                "fieldId": fi.field.field_id,
                "fieldType": format!("{:?}", fi.field.field_type),
                "name": fi.field.field_name().unwrap_or(""),
                "guide": fi.field.guide_text().unwrap_or(""),
                "memo": fi.field.memo_text().unwrap_or_default(),
                "command": fi.field.command,
                "value": fi.value,
                "editableInForm": fi.field.is_editable_in_form(),
                "location": {
                    "section": fi.location.section_index,
                    "paragraph": fi.location.para_index,
                    "nested": nested,
                },
            })
        })
        .collect()
}

/// [#3762] `export-ir-schema` — 공개 IR 의 JSON Schema 를 낸다 (M18 바인딩 착수 조건).
///
/// 문서를 입력으로 받지 않는다 — 스키마는 **타입의 자기서술**이지 특정 문서의
/// 속성이 아니다. capabilities 가 명령 표면을 설명하듯, 이 명령은 문서 모델을
/// 설명한다. 외부 바인딩 세대가 코드 생성의 단일 출처로 쓴다.
fn cmd_export_ir_schema(args: &[String]) -> i32 {
    let mut out_path: Option<&str> = None;
    let mut json_mode = false;
    let mut bare = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json_mode = true,
            // 봉투 없이 스키마 본문만 — JSON Schema 도구에 바로 먹이려는 용도.
            "--bare" => bare = true,
            "-o" | "--out" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.as_str()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            other => {
                eprintln!("오류: 알 수 없는 옵션입니다 - {}", other);
                return EXIT_USAGE;
            }
        }
        i += 1;
    }

    let payload = if bare {
        // --bare 는 JSON Schema 검증기에 그대로 먹이는 본문이다 — 봉투 표지를 섞지 않는다.
        rhwp::ir_schema::ir_schema()
    } else {
        // [#3885] "표지는 항상 실린다" — 문서를 열지 않는 명령의 봉투도
        // untrustedContent:false 를 명시한다. 키 부재는 "안전"이 아니라
        // "이 빌드는 표지를 모른다"로 읽히기 때문이다.
        provenance::marked(rhwp::ir_schema::envelope(), "export-ir-schema")
    };
    let text = match serde_json::to_string_pretty(&payload) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("오류: 스키마 직렬화 실패 - {}", e);
            return EXIT_RUNTIME;
        }
    };

    if let Some(path) = out_path {
        if let Err(e) = fs::write(path, text.as_bytes()) {
            eprintln!("오류: 스키마를 쓸 수 없습니다 - {}: {}", path, e);
            return EXIT_RUNTIME;
        }
        if json_mode {
            // 파일로 뺐어도 stdout 은 기계 계약을 유지한다 — 어디에 썼는지 알려준다.
            println!(
                "{}",
                provenance::marked(
                    serde_json::json!({
                        "schemaVersion": ENVELOPE_SCHEMA_VERSION,
                        "irSchemaVersion": rhwp::ir_schema::IR_SCHEMA_VERSION,
                        "output": path,
                        "bytes": text.len(),
                    }),
                    "export-ir-schema"
                )
            );
        } else {
            println!("IR 스키마 저장: {} ({} bytes)", path, text.len());
        }
        return EXIT_OK;
    }

    println!("{text}");
    EXIT_OK
}

/// [#3719 §6-4] `export-plan-schema` — `run` 계획서 문법의 JSON Schema 를 낸다.
///
/// 문서를 입력으로 받지 않는다 — 스키마는 **계획서 문법의 자기서술**이지 특정 문서의
/// 속성이 아니다. `run --json` 이 이미 쓴 계획을 검사한다면, 이 명령은 계획을 **쓰기
/// 전에** 읽는 정답지다. 필드명을 지어내고 `invalid[]` 로 되돌아오는 왕복이 계획 생성
/// 실패의 대부분이라, 그 왕복을 없애는 것이 목적이다.
fn cmd_export_plan_schema(args: &[String]) -> i32 {
    let mut out_path: Option<&str> = None;
    let mut json_mode = false;
    let mut bare = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json_mode = true,
            // 봉투 없이 스키마 본문만 — JSON Schema 검증기에 바로 먹이려는 용도.
            "--bare" => bare = true,
            "-o" | "--out" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.as_str()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            other => {
                eprintln!("오류: 알 수 없는 옵션입니다 - {}", other);
                return EXIT_USAGE;
            }
        }
        i += 1;
    }

    let payload = if bare {
        // --bare 는 JSON Schema 검증기에 그대로 먹이는 본문이다 — 봉투 표지를 섞지 않는다.
        rhwp::plan_schema::plan_schema()
    } else {
        // [#3787 S1] "표지는 항상 실린다" — 문서를 열지 않는 명령의 봉투도
        // untrustedContent:false 를 명시한다는 것이 capabilities 의 선언이다.
        provenance::marked(rhwp::plan_schema::envelope(), "export-plan-schema")
    };
    let text = match serde_json::to_string_pretty(&payload) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("오류: 스키마 직렬화 실패 - {}", e);
            return EXIT_RUNTIME;
        }
    };

    if let Some(path) = out_path {
        if let Err(e) = fs::write(path, text.as_bytes()) {
            eprintln!("오류: 스키마를 쓸 수 없습니다 - {}: {}", path, e);
            return EXIT_RUNTIME;
        }
        if json_mode {
            // 파일로 뺐어도 stdout 은 기계 계약을 유지한다 — 어디에 썼는지 알려준다.
            println!(
                "{}",
                provenance::marked(
                    serde_json::json!({
                        "schemaVersion": ENVELOPE_SCHEMA_VERSION,
                        "planSchemaVersion": rhwp::plan_schema::PLAN_SCHEMA_VERSION,
                        "output": path,
                        "bytes": text.len(),
                    }),
                    "export-plan-schema"
                )
            );
        } else {
            println!("계획 스키마 저장: {} ({} bytes)", path, text.len());
        }
        return EXIT_OK;
    }

    println!("{text}");
    EXIT_OK
}

/// [#3776] `export-capabilities-schema` — capabilities 자체의 JSON Schema 를 낸다.
fn cmd_export_capabilities_schema(args: &[String]) -> i32 {
    let mut out_path: Option<&str> = None;
    let mut json_mode = false;
    let mut bare = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json_mode = true,
            "--bare" => bare = true,
            "-o" | "--out" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.as_str()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            other => {
                eprintln!("오류: 알 수 없는 옵션입니다 - {}", other);
                return EXIT_USAGE;
            }
        }
        i += 1;
    }

    let payload = if bare {
        // --bare 는 JSON Schema 검증기에 그대로 먹이는 본문이다 — 봉투 표지를 섞지 않는다.
        rhwp::capabilities_schema::capabilities_schema()
    } else {
        // [#3885] export-ir-schema 와 같은 사유 — 문서를 열지 않아도 표지는 싣는다.
        provenance::marked(
            rhwp::capabilities_schema::envelope(),
            "export-capabilities-schema",
        )
    };
    let text = match serde_json::to_string_pretty(&payload) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("오류: 스키마 직렬화 실패 - {}", e);
            return EXIT_RUNTIME;
        }
    };

    if let Some(path) = out_path {
        if let Err(e) = fs::write(path, text.as_bytes()) {
            eprintln!("오류: 스키마를 쓸 수 없습니다 - {}: {}", path, e);
            return EXIT_RUNTIME;
        }
        if json_mode {
            println!(
                "{}",
                provenance::marked(
                    serde_json::json!({
                        "schemaVersion": ENVELOPE_SCHEMA_VERSION,
                        "capabilitiesSchemaVersion":
                            rhwp::capabilities_schema::CAPABILITIES_SCHEMA_VERSION,
                        "output": path,
                        "bytes": text.len(),
                    }),
                    "export-capabilities-schema"
                )
            );
        } else {
            println!("capabilities 스키마 저장: {} ({} bytes)", path, text.len());
        }
        return EXIT_OK;
    }

    println!("{text}");
    EXIT_OK
}

/// [#3907 O1] `export-ontology` — 자기서술에서 JSON-LD 온톨로지를 기계 유도한다.
///
/// 문서를 입력으로 받지 않는다 — 온톨로지는 rhwp 라는 **도구 자신**(IR 타입·명령
/// 표면·신뢰 경계)의 서술이지 특정 문서의 속성이 아니다. 유도 원천은 전부 같은
/// 크레이트의 단일 출처 함수다: `ir_schema()`·`cli::metadata::capabilities::capabilities_value()`·
/// `cli::metadata::mcp::mcp_tool_definitions()`·`provenance::MAP`. 손 나열 상수가 없으므로 원천이
/// 바뀌면 온톨로지가 함께 바뀐다 — 드리프트 구조적 불가능이 이 명령의 논지다.
/// 문서 인스턴스 모드(O2)는 후속이다.
fn cmd_export_ontology(args: &[String]) -> i32 {
    let mut out_path: Option<&str> = None;
    let mut json_mode = false;
    let mut bare = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json_mode = true,
            // 봉투 없이 JSON-LD 본문만 — RDF/JSON-LD 도구에 바로 먹이려는 용도.
            "--bare" => bare = true,
            "-o" | "--out" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.as_str()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            other => {
                eprintln!("오류: 알 수 없는 옵션입니다 - {}", other);
                return EXIT_USAGE;
            }
        }
        i += 1;
    }

    let caps = cli::metadata::capabilities::capabilities_value();
    let tools = cli::metadata::mcp::mcp_tool_definitions();
    let payload = if bare {
        // --bare 는 JSON-LD 처리기에 그대로 먹이는 본문이다 — 봉투 표지를 섞지 않는다.
        rhwp::ontology::ontology(&caps, &tools)
    } else {
        // [#3885] "표지는 항상 실린다" — 문서를 열지 않는 명령의 봉투도
        // untrustedContent:false 를 명시한다.
        provenance::marked(rhwp::ontology::envelope(&caps, &tools), "export-ontology")
    };
    let text = match serde_json::to_string_pretty(&payload) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("오류: 온톨로지 직렬화 실패 - {}", e);
            return EXIT_RUNTIME;
        }
    };

    if let Some(path) = out_path {
        if let Err(e) = fs::write(path, text.as_bytes()) {
            eprintln!("오류: 온톨로지를 쓸 수 없습니다 - {}: {}", path, e);
            return EXIT_RUNTIME;
        }
        if json_mode {
            // 파일로 뺐어도 stdout 은 기계 계약을 유지한다 — 어디에 썼는지 알려준다.
            println!(
                "{}",
                provenance::marked(
                    serde_json::json!({
                        "schemaVersion": ENVELOPE_SCHEMA_VERSION,
                        "ontologyVersion": rhwp::ontology::ONTOLOGY_VERSION,
                        "output": path,
                        "bytes": text.len(),
                    }),
                    "export-ontology"
                )
            );
        } else {
            println!("온톨로지 저장: {} ({} bytes)", path, text.len());
        }
        return EXIT_OK;
    }

    println!("{text}");
    EXIT_OK
}

/// [#3828 B2] `export-agent-manifest` 조립 코어 — capabilities·irSchema·provenanceMap·
/// planSchema 를 왕복 1회로 묶는다.
///
/// 각 서브필드는 해당 명령의 기존 산출 함수를 그대로 불러 조립만 한다 — 스키마·지도
/// 로직을 여기서 다시 만들지 않는다. `missingAxes` 는 네 축이 모두 실린 지금 빈
/// 배열이지만 필드 자체는 남긴다 — 앞으로 축이 늘 때 "아직 없는 축"을 이 배열로
/// 알리는 것이 B2 의 계약이고, null 로 채우면 "값이 비었다"와 "명령이 아직 없다"를
/// 소비자가 구분할 수 없다.
fn agent_manifest_value(bare: bool) -> serde_json::Value {
    let mut fields = serde_json::Map::new();
    fields.insert(
        "capabilities".to_string(),
        provenance::marked(
            cli::metadata::capabilities::capabilities_value(),
            "capabilities",
        ),
    );
    fields.insert("irSchema".to_string(), rhwp::ir_schema::ir_schema());
    fields.insert(
        "provenanceMap".to_string(),
        provenance::marked(
            provenance::map_json(&rhwp::version()),
            "export-provenance-map",
        ),
    );
    // [#3808] planSchema 축 — irSchema 처럼 bare 본문을 싣는다. 본문이 `$id`·
    // `planSchemaVersion` 을 자체 내장하므로 봉투 메타를 중복하지 않는다.
    fields.insert("planSchema".to_string(), rhwp::plan_schema::plan_schema());
    fields.insert("missingAxes".to_string(), serde_json::json!([]));

    if bare {
        return serde_json::Value::Object(fields);
    }
    let mut envelope = serde_json::Map::new();
    envelope.insert(
        "schemaVersion".to_string(),
        serde_json::json!(ENVELOPE_SCHEMA_VERSION),
    );
    envelope.extend(fields);
    serde_json::Value::Object(envelope)
}

/// [#3828 B2] `export-agent-manifest` — 처음 붙는 에이전트가 capabilities →
/// export-ir-schema → export-provenance-map → export-plan-schema 를 각각 따로
/// 호출하던 왕복 4회를 1회로 줄인다.
fn cmd_export_agent_manifest(args: &[String]) -> i32 {
    let mut json_mode = false;
    let mut bare = false;
    for arg in args {
        match arg.as_str() {
            "--json" => json_mode = true,
            "--bare" => bare = true,
            other => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
        }
    }

    let manifest = provenance::marked(agent_manifest_value(bare), "export-agent-manifest");

    if json_mode {
        let text = match serde_json::to_string_pretty(&manifest) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("오류: 매니페스트 직렬화 실패 - {}", e);
                return EXIT_RUNTIME;
            }
        };
        println!("{text}");
        return EXIT_OK;
    }

    println!("rhwp 에이전트 매니페스트 (capabilities + irSchema + provenanceMap 조립)");
    println!();
    println!("  capabilities     포함");
    println!("  irSchema         포함");
    println!("  provenanceMap    포함");
    println!("  planSchema       포함");
    println!();
    println!("기계 계약은 --json 을 쓰세요 (--bare 로 최상위 표지 없이).");
    EXIT_OK
}

/// [#3787 S2] `tool_directive` 판정에 쓰는 **도구 이름 등록부**.
///
/// 이름을 탐지 모듈에 하드코딩하지 않는다. 도구가 늘어도 목록이 따라오지 않으면
/// 새 도구를 부르는 주입문이 조용히 통과하기 때문이다. 원천은 이 저장소가 이미
/// 가진 두 등록부다 — 무상태 도구는 `cli::metadata::mcp::mcp_tool_definitions()`(= `capabilities --mcp`
/// 의 stdout), 세션 도구는 `agent_profiles::ALL_SESSION_TOOLS`(= `mcp-serve` 가 여는
/// 집합). 둘 중 어디에 도구를 더해도 탐지가 함께 자란다.
fn mcp_tool_name_registry() -> Vec<String> {
    let mut names: Vec<String> = cli::metadata::mcp::mcp_tool_definitions()
        .iter()
        .filter_map(|t| t["name"].as_str().map(String::from))
        .collect();
    names.extend(
        agent_profiles::ALL_SESSION_TOOLS
            .iter()
            .map(|s| s.to_string()),
    );
    names.sort();
    names.dedup();
    names
}

/// `inspect` — 문서를 **읽기만** 하는 보안 검사 명령군.
///
/// `hidden-text`·`injection`·`unicode`는 각각 조판 은닉, 문장형 지시, 화면과 바이트의
/// 불일치를 판정한다. 어느 축도 문서를 고치지 않는다.
fn inspect_command(args: &[String]) -> i32 {
    const USAGE: &str =
        "사용법: rhwp inspect <hidden-text|injection|unicode|watermark> <파일.hwp|파일.hwpx> [각 축 옵션]";

    match args.first().map(|s| s.as_str()) {
        Some("hidden-text") => cli::queries::security_inspection::inspect_hidden_text(&args[1..]),
        Some("injection") => cli::queries::security_inspection::inspect_injection(&args[1..]),
        Some("unicode") => cli::queries::security_inspection::inspect_unicode(&args[1..]),
        Some("watermark") => cli::queries::security_inspection::inspect_watermark(&args[1..]),
        Some(other) => {
            eprintln!("오류: 알 수 없는 inspect 하위 명령입니다 - {other}");
            let hint = cli::metadata::capabilities::closest_name(
                other,
                ["hidden-text", "injection", "unicode", "watermark"],
            );
            if let Some(hint) = &hint {
                eprintln!("혹시 이것인가요? inspect {hint}");
            }
            eprintln!("{USAGE}");
            // [#4220 T4] 확신 교정(#3694 임계 내)일 때만 정형 수복 줄 — 임계 밖은 침묵.
            if let Some(hint) = hint {
                cli::metadata::capabilities::eprint_usage_recovery(
                    "inspect",
                    Some(&hint),
                    "요청한 이름이 없음 — 가장 가까운 실존 하위 명령으로 교정",
                );
            }
            EXIT_USAGE
        }
        None => {
            // [#4220 T4] 하위 명령 누락은 어느 축을 원했는지 결정론적으로 알 수 없다 —
            // 수복 줄을 지어내지 않는다(오제안 0).
            eprintln!(
                "오류: inspect 하위 명령을 지정해주세요 (hidden-text|injection|unicode|watermark)."
            );
            eprintln!("{USAGE}");
            EXIT_USAGE
        }
    }
}

/// 현재 스캔이 실제로 훑는 영역 이름 — 봉투와 사람 출력이 같은 목록을 쓴다.
fn injection_scan_scopes(include_fields: bool) -> Vec<&'static str> {
    let mut scopes = vec![
        "body",
        "tableCell",
        "textBox",
        "equation",
        "footnote",
        "endnote",
        "header",
        "footer",
        "caption",
    ];
    if include_fields {
        scopes.extend([
            "fieldName",
            "fieldGuide",
            "fieldCommand",
            "hiddenComment",
            "fieldMemo",
        ]);
    }
    scopes
}

/// 터미널로 나가는 발췌의 제어문자를 보이는 기호로 바꾼다.
///
/// 문서 텍스트는 고치지 않는다 — 여기서 바뀌는 것은 **화면 표시**뿐이다(`--json` 봉투는
/// serde 가 `\u001b` 로 이스케이프하므로 손대지 않는다). 주입 문서가 ANSI 이스케이프를
/// 함께 심으면 경고 줄 자체를 지우거나 색으로 덮어 사람을 속일 수 있다.
fn display_safe(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '\u{1b}' => '␛',
            '\n' | '\r' => '⏎',
            '\t' => '⇥',
            c if (c as u32) < 0x20 => '␀',
            c => c,
        })
        .collect()
}

#[cfg(test)]
mod doc_mcp_hwp_write_cli_tests {
    use super::*;

    /// samples/issue_1133.hwp 문단 29 = 8×2 표, 셀 3 문단 5 안의 1×1 표 "※ 지원시 유의사항".
    fn nested_notice_color_at(bytes: &[u8], offset: usize) -> rhwp::model::ColorRef {
        use rhwp::model::control::Control;
        let core = rhwp::document_core::DocumentCore::from_bytes(bytes).expect("parse");
        let document = core.document();
        let Control::Table(outer) = &document.sections[0].paragraphs[29].controls[0] else {
            panic!("outer table");
        };
        let Control::Table(inner) = &outer.cells[3].paragraphs[5].controls[0] else {
            panic!("inner table");
        };
        let paragraph = &inner.cells[0].paragraphs[0];
        assert!(paragraph.text.starts_with("※ 지원시"), "{}", paragraph.text);
        let id = paragraph.char_shape_id_at(offset).expect("char shape") as usize;
        document.doc_info.char_shapes[id].text_color
    }

    #[test]
    fn set_cell_char_format_reaches_a_nested_table_cell() {
        let data = fs::read("samples/issue_1133.hwp").expect("sample");
        let before_head = nested_notice_color_at(&data, 0);
        let before_tail = nested_notice_color_at(&data, 8);
        let target = if before_head == 0 { "#FF0000" } else { "#000000" };
        let result = set_hwp_cell_char_format_by_path_bytes_for_cli(
            &data,
            0,
            29,
            "[[0,3,5],[0,0,0]]",
            0,
            3,
            &format!(r#"{{"textColor":"{target}"}}"#),
        )
        .expect("nested char format");
        assert_ne!(nested_notice_color_at(&result.bytes, 0), before_head, "range recolored");
        assert_eq!(nested_notice_color_at(&result.bytes, 8), before_tail, "outside the range kept");
        assert!(set_hwp_cell_char_format_by_path_bytes_for_cli(&data, 0, 29, "[]", 0, 1, "{}").is_err());
    }

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
            &[(0, 0, 0)],
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
    use crate::cli::outputs::allows_implicit_sibling_resources;
    use crate::cli::protocol::{
        collect_audit_capsules, replay_scratch_dir, with_replay_input_snapshot,
    };

    use super::{
        cli_output_password, cli_password, set_cli_output_password, set_cli_password,
        strip_global_auth_options, strip_utf8_bom, EXIT_USAGE,
    };
    use rhwp::parser::FileFormat;

    #[test]
    fn hml_does_not_implicitly_load_sibling_resources() {
        assert!(!allows_implicit_sibling_resources(FileFormat::Hml));
        assert!(allows_implicit_sibling_resources(FileFormat::Hwp));
        assert!(allows_implicit_sibling_resources(FileFormat::Hwpx));
    }

    #[test]
    fn replay_engine_receives_the_hashed_input_snapshot() {
        let original =
            std::env::temp_dir().join(format!("rhwp-replay-original-{}.hwp", std::process::id()));
        std::fs::write(&original, b"original bytes").expect("원본 작성");
        let mut plan = serde_json::json!({ "input": original.to_string_lossy() });
        let scratch = replay_scratch_dir("unit").expect("전용 임시 폴더");
        let scratch_path = scratch.0.clone();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                std::fs::metadata(&scratch_path)
                    .expect("전용 임시 폴더 metadata")
                    .permissions()
                    .mode()
                    & 0o777,
                0o700
            );
        }
        let seen = with_replay_input_snapshot(
            &mut plan,
            b"hashed snapshot",
            &scratch.0,
            |snapshot_plan| {
                std::fs::write(&original, b"changed after hashing").expect("원본 교체");
                let snapshot_path = snapshot_plan["input"].as_str().expect("스냅샷 경로");
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    assert_eq!(
                        std::fs::metadata(snapshot_path)
                            .expect("입력 스냅샷 metadata")
                            .permissions()
                            .mode()
                            & 0o777,
                        0o600
                    );
                }
                std::fs::read(snapshot_path).expect("스냅샷 읽기")
            },
        )
        .expect("스냅샷 실행");
        assert_eq!(seen, b"hashed snapshot");
        assert_eq!(plan["input"], original.to_string_lossy().as_ref());
        drop(scratch);
        assert!(!scratch_path.exists(), "전용 임시 폴더는 RAII 정리");
        let _ = std::fs::remove_file(original);
    }

    #[test]
    fn audit_directory_entry_errors_are_not_silently_dropped() {
        let entries: [std::io::Result<std::path::PathBuf>; 1] = [Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "denied",
        ))];
        let error = collect_audit_capsules(entries).expect_err("항목 오류는 fail-closed");
        assert!(error.contains("폴더 항목 읽기 실패"));
    }

    #[test]
    fn global_password_option_is_removed_from_any_position() {
        let args = vec![
            "rhwp".to_string(),
            "info".to_string(),
            "sample.hwp".to_string(),
            "--password".to_string(),
            "secret".to_string(),
        ];
        set_cli_password(None);
        let clean = strip_global_auth_options(args).unwrap();
        assert_eq!(clean, ["rhwp", "info", "sample.hwp"]);
        // 비밀번호는 반환값이 아니라 CLI_PASSWORD(thread_local)로 전달된다.
        assert_eq!(cli_password().as_deref(), Some("secret"));
        set_cli_password(None);
    }

    #[test]
    fn password_stdin_ignores_only_a_leading_utf8_bom() {
        assert_eq!(strip_utf8_bom("\u{feff}123456\n"), "123456\n");
        assert_eq!(strip_utf8_bom("123456\n"), "123456\n");
        assert_eq!(
            strip_utf8_bom("123456\n\u{feff}next"),
            "123456\n\u{feff}next"
        );
    }

    #[test]
    fn duplicate_global_password_options_are_rejected() {
        let args = vec![
            "rhwp".to_string(),
            "--password".to_string(),
            "first".to_string(),
            "info".to_string(),
            "sample.hwp".to_string(),
            "--password".to_string(),
            "second".to_string(),
        ];
        assert!(matches!(
            strip_global_auth_options(args),
            Err(code) if code == EXIT_USAGE
        ));
    }

    #[test]
    fn global_output_password_is_removed_without_leaking_into_command_args() {
        let args = vec![
            "rhwp".to_string(),
            "convert".to_string(),
            "source.hwp".to_string(),
            "output.hwp".to_string(),
            "--output-password".to_string(),
            "protected".to_string(),
        ];
        set_cli_password(None);
        set_cli_output_password(None);
        let clean = strip_global_auth_options(args).unwrap();
        assert_eq!(clean, ["rhwp", "convert", "source.hwp", "output.hwp"]);
        assert_eq!(cli_output_password().as_deref(), Some("protected"));
        set_cli_output_password(None);
    }

    #[test]
    fn duplicate_global_output_password_options_are_rejected() {
        let args = vec![
            "rhwp".to_string(),
            "--output-password".to_string(),
            "first".to_string(),
            "convert".to_string(),
            "source.hwp".to_string(),
            "output.hwp".to_string(),
            "--output-password".to_string(),
            "second".to_string(),
        ];
        assert!(matches!(
            strip_global_auth_options(args),
            Err(code) if code == EXIT_USAGE
        ));
    }
}
