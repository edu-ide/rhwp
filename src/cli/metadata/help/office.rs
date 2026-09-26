//! Office MCP fork command help retained alongside the upstream catalog.

use super::sink::println;

pub(super) fn print() {
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
    println!("  list-objects <파일.hwp>");
    println!("      본문/표 셀 내부 그림·도형·글상자 객체 목록을 JSON으로 조회");
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
    println!("      * --cell-path 는 표 안의 표(중첩 표) 셀까지 지정한다: [[표ctrl,셀,셀문단],[안쪽표ctrl,셀,셀문단]]");
    println!();
    println!("  insert-cell-text <파일.hwp> --para N [--ctrl N (--cell N|--row N --col N) [--cell-para N] | --cell-path <JSON>] --offset N --text <텍스트> -o <출력.hwp>");
    println!("      표 셀 문단의 지정 문자 오프셋에 텍스트를 삽입");
    println!();
    println!("  delete-cell-text <파일.hwp> --para N [--ctrl N (--cell N|--row N --col N) [--cell-para N] | --cell-path <JSON>] --offset N --count N -o <출력.hwp>");
    println!("      표 셀 문단의 지정 문자 범위를 삭제");
    println!();
    println!("  insert-cell-paragraph <파일.hwp> --para N --ctrl N (--cell N|--row N --col N) --cell-para N [--text <텍스트>] -o <출력.hwp>");
    println!("      표 셀 내부의 지정 위치에 새 문단을 삽입");
    println!();
    println!("  delete-cell-paragraph <파일.hwp> --para N --ctrl N (--cell N|--row N --col N) --cell-para N -o <출력.hwp>");
    println!("      표 셀 내부 문단을 삭제");
    println!();
    println!("  move-cell-paragraphs <파일.hwp> --para N --ctrl N --from-row R --from-col C --start N --end N --to-row R --to-col C [--at N] -o <출력.hwp>");
    println!("  split-cell-paragraph <파일.hwp> --para N [--ctrl N (--cell N|--row N --col N) [--cell-para N] | --cell-path <JSON>] --offset N -o <출력.hwp>");
    println!("      표 셀 내부 문단을 문자 오프셋에서 분할");
    println!();
    println!("  merge-cell-paragraph <파일.hwp> --para N [--ctrl N (--cell N|--row N --col N) --cell-para N | --cell-path <JSON>] -o <출력.hwp>");
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
    println!("  get-cell-text <파일.hwp> --para N [--ctrl N --cell N | --cell-path <JSON>]");
    println!("      셀 안 문단의 글자 수와 텍스트를 조회 (중첩 표 포함)");
    println!("      * 편집 명령의 --offset/--count 를 눈대중으로 넘기면 옆 문단이 잘린다.");
    println!("        먼저 이 명령으로 문단별 실제 길이를 확인할 것");
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
    println!("  resize-table-cells <파일.hwp> --section N --para N [--cell-path <JSON>|--ctrl N] --json <변경배열JSON> -o <출력.hwp>");
    println!("  set-table-column-widths <파일.hwp> --section N --para N --ctrl N --widths w1,w2,... -o <출력.hwp>");
    println!("      여러 표 셀의 폭/높이를 델타 값으로 조절");
    println!();
    println!("  apply-table-style <파일.hwp> --section N --para N --ctrl N [--head-fill #d9d9d9] [--font-size 1200] [--head-height 2232] [--body-height 2882] -o <출력.hwp>");
    println!("      표에 심사 문서용 「집 서식」을 한 번에 적용 (머리행 음영+이중선, 정렬, 12pt, 행 높이)");
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
    println!("  set-cell-char-format <파일.hwp> --section N --para N (--ctrl N (--cell N|--row R --col C) [--cell-para N] | --cell-path <JSON>) --start N --end N --json <서식JSON> -o <출력.hwp>");
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
    println!("      --natural-width/--natural-height 생략 시 이미지 헤더에서 자동 판독");
    println!("      --inline  표 셀 안에 글자처럼(인라인) 삽입 — 셀 높이가 그림에 맞춰 늘어난다.");
    println!(
        "                생략하면 한컴 default 인 표 옆 floating 배치 (셀 밖으로 넘칠 수 있음)"
    );
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
    println!("  delete-header-footer <파일.hwp> --section N --kind header|footer --apply-to both|even|odd -o <출력.hwp>");
    println!("      머리말/꼬리말 컨트롤 삭제");
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
}

#[cfg(test)]
mod tests {
    #[test]
    fn office_nested_cell_commands_keep_scoped_help() {
        for command in [
            "create-hwp",
            "set-cell-text",
            "set-cell-char-format",
            "get-cell-properties",
        ] {
            let lines = super::super::sink::collect(command, None, super::print);
            assert!(!lines.is_empty(), "missing help: {command}");
        }
    }
}
