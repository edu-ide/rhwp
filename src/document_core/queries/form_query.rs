//! 양식 개체 조회/설정 API (Task 233)
//!
//! 렌더 트리에서 양식 개체를 좌표로 찾거나, 문서 트리에서 직접 값을 조회/설정한다.

use crate::document_core::helpers::{find_control_text_positions, get_textbox_from_shape_mut};
use crate::document_core::DocumentCore;
use crate::model::control::{Control, FormObject, FormType};
use crate::model::document::Section;
use crate::model::paragraph::Paragraph;
use crate::model::table::Table;
use crate::renderer::render_tree::{FormObjectNode, RenderNode, RenderNodeType};
use std::collections::HashMap;

impl DocumentCore {
    /// 본문 문단에 양식 개체를 생성한다.
    pub fn create_form_object_native(
        &mut self,
        sec: usize,
        para: usize,
        char_offset: usize,
        form_type: FormType,
        name: &str,
        caption: &str,
        text: &str,
        width: u32,
        height: u32,
        value: i32,
        enabled: bool,
        properties_json: &str,
    ) -> Result<String, crate::error::HwpError> {
        let section = self
            .document
            .sections
            .get_mut(sec)
            .ok_or_else(|| crate::error::HwpError::RenderError("구역 위치 초과".into()))?;
        section.raw_stream = None;
        let paragraph = section
            .paragraphs
            .get_mut(para)
            .ok_or_else(|| crate::error::HwpError::RenderError("문단 위치 초과".into()))?;
        let insert_idx = insert_form_into_paragraph(
            paragraph,
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
        self.recompose_section(sec);

        Ok(format!(
            r#"{{"ok":true,"operation":"create-form","formType":"{}","name":"{}","caption":"{}","text":"{}","value":{},"enabled":{},"width":{},"height":{},"paraIdx":{},"controlIdx":{}}}"#,
            form_type_to_str(form_type),
            escape_json(name.trim()),
            escape_json(caption),
            escape_json(text),
            value,
            enabled,
            width,
            height,
            para,
            insert_idx,
        ))
    }

    /// 표 셀/글상자 내부 문단에 양식 개체를 생성한다.
    pub fn create_cell_form_object_native(
        &mut self,
        sec: usize,
        parent_para: usize,
        cell_path: &[(usize, usize, usize)],
        char_offset: usize,
        form_type: FormType,
        name: &str,
        caption: &str,
        text: &str,
        width: u32,
        height: u32,
        value: i32,
        enabled: bool,
        properties_json: &str,
    ) -> Result<String, crate::error::HwpError> {
        if cell_path.is_empty() {
            return Err(crate::error::HwpError::RenderError(
                "cell_path가 비어 있습니다.".to_string(),
            ));
        }
        let section = self
            .document
            .sections
            .get_mut(sec)
            .ok_or_else(|| crate::error::HwpError::RenderError("구역 위치 초과".into()))?;
        section.raw_stream = None;
        let paragraph = resolve_form_cell_paragraph_mut(section, parent_para, cell_path)?;
        let insert_idx = insert_form_into_paragraph(
            paragraph,
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
        self.recompose_section(sec);

        Ok(format!(
            r#"{{"ok":true,"operation":"create-form","container":"cell","formType":"{}","name":"{}","caption":"{}","text":"{}","value":{},"enabled":{},"width":{},"height":{},"paraIdx":{},"controlIdx":{},"cellPath":{}}}"#,
            form_type_to_str(form_type),
            escape_json(name.trim()),
            escape_json(caption),
            escape_json(text),
            value,
            enabled,
            width,
            height,
            parent_para,
            insert_idx,
            cell_path_json(cell_path),
        ))
    }

    /// 페이지 좌표에서 양식 개체를 찾는다.
    ///
    /// 렌더 트리를 순회하며 FormObject 노드의 bbox와 좌표 충돌 검사.
    /// 반환: JSON 문자열
    pub fn get_form_object_at_native(
        &self,
        page_num: u32,
        x: f64,
        y: f64,
    ) -> Result<String, crate::error::HwpError> {
        let tree = self.build_page_tree_cached(page_num)?;

        if let Some((form, bbox)) = find_form_node_at(&tree.root, x, y) {
            let form_type_str = form_type_to_str(form.form_type);
            // 셀 내부 위치 정보 직렬화
            let cell_loc_json = if let Some((tpi, tci, ci_idx, cp_idx)) = form.cell_location {
                format!(
                    r#","inCell":true,"tablePara":{},"tableCi":{},"cellIdx":{},"cellPara":{}"#,
                    tpi, tci, ci_idx, cp_idx
                )
            } else {
                String::new()
            };
            // sec/para는 최상위 문단 인덱스로 반환
            // cell_location이 있으면 table_para_index를 para로 사용
            let (ret_para, ret_ci) = if let Some((tpi, _tci, _ci_idx, _cp_idx)) = form.cell_location
            {
                (tpi, form.control_index)
            } else {
                (form.para_index, form.control_index)
            };
            Ok(format!(
                r#"{{"found":true,"sec":{},"para":{},"ci":{},"formType":"{}","name":"{}","value":{},"caption":"{}","text":"{}","bbox":{{"x":{},"y":{},"w":{},"h":{}}}{}}}"#,
                form.section_index,
                ret_para,
                ret_ci,
                form_type_str,
                escape_json(&form.name),
                form.value,
                escape_json(&form.caption),
                escape_json(&form.text),
                bbox.0,
                bbox.1,
                bbox.2,
                bbox.3,
                cell_loc_json,
            ))
        } else {
            Ok(r#"{"found":false}"#.to_string())
        }
    }

    /// 양식 개체 값을 조회한다.
    pub fn get_form_value_native(
        &self,
        sec: usize,
        para: usize,
        ci: usize,
    ) -> Result<String, crate::error::HwpError> {
        let control = self
            .document
            .sections
            .get(sec)
            .and_then(|s| s.paragraphs.get(para))
            .and_then(|p| p.controls.get(ci));

        match control {
            Some(Control::Form(f)) => {
                let form_type_str = form_type_to_str(f.form_type);
                Ok(format!(
                    r#"{{"ok":true,"formType":"{}","name":"{}","value":{},"text":"{}","caption":"{}","enabled":{}}}"#,
                    form_type_str,
                    escape_json(&f.name),
                    f.value,
                    escape_json(&f.text),
                    escape_json(&f.caption),
                    f.enabled,
                ))
            }
            _ => Ok(r#"{"ok":false,"error":"not a form object"}"#.to_string()),
        }
    }

    /// 양식 개체 값을 설정한다 (최상위 문단의 Form).
    ///
    /// value_json: `{"value":1}` 또는 `{"text":"입력값"}` 또는 둘 다
    pub fn set_form_value_native(
        &mut self,
        sec: usize,
        para: usize,
        ci: usize,
        value_json: &str,
    ) -> Result<String, crate::error::HwpError> {
        let section = self
            .document
            .sections
            .get_mut(sec)
            .ok_or_else(|| crate::error::HwpError::RenderError("구역 위치 초과".into()))?;
        section.raw_stream = None;
        let control = section
            .paragraphs
            .get_mut(para)
            .and_then(|p| p.controls.get_mut(ci));

        match control {
            Some(Control::Form(f)) => {
                apply_form_value(f, value_json);
                let value = f.value;
                let text = f.text.clone();
                let caption = f.caption.clone();
                self.recompose_section(sec);
                Ok(format!(
                    r#"{{"ok":true,"operation":"set-form","sec":{},"para":{},"control":{},"value":{},"text":"{}","caption":"{}"}}"#,
                    sec,
                    para,
                    ci,
                    value,
                    escape_json(&text),
                    escape_json(&caption),
                ))
            }
            _ => Ok(r#"{"ok":false,"error":"not a form object"}"#.to_string()),
        }
    }

    /// 최상위 문단의 양식 개체를 삭제한다.
    pub fn delete_form_object_native(
        &mut self,
        sec: usize,
        para: usize,
        ci: usize,
    ) -> Result<String, crate::error::HwpError> {
        let section = self
            .document
            .sections
            .get_mut(sec)
            .ok_or_else(|| crate::error::HwpError::RenderError("구역 위치 초과".into()))?;
        section.raw_stream = None;
        let paragraph = section
            .paragraphs
            .get_mut(para)
            .ok_or_else(|| crate::error::HwpError::RenderError("문단 위치 초과".into()))?;
        delete_form_from_paragraph(paragraph, ci)?;
        self.recompose_section(sec);
        self.paginate_if_needed();
        self.invalidate_page_tree_cache();
        Ok(format!(
            r#"{{"ok":true,"operation":"delete-form","sec":{},"para":{},"control":{}}}"#,
            sec, para, ci
        ))
    }

    /// 셀 내부 양식 개체 값을 설정한다.
    ///
    /// table_para: 표를 포함한 최상위 문단 인덱스
    /// table_ci: 표 컨트롤 인덱스
    /// cell_idx: 셀 인덱스
    /// cell_para: 셀 내 문단 인덱스
    /// form_ci: 셀 내 양식 컨트롤 인덱스
    pub fn set_form_value_in_cell_native(
        &mut self,
        sec: usize,
        table_para: usize,
        table_ci: usize,
        cell_idx: usize,
        cell_para: usize,
        form_ci: usize,
        value_json: &str,
    ) -> Result<String, crate::error::HwpError> {
        let form = self
            .document
            .sections
            .get_mut(sec)
            .and_then(|s| s.paragraphs.get_mut(table_para))
            .and_then(|p| p.controls.get_mut(table_ci))
            .and_then(|c| {
                if let Control::Table(ref mut t) = c {
                    Some(t.as_mut())
                } else {
                    None
                }
            })
            .and_then(|t: &mut Table| t.cells.get_mut(cell_idx))
            .and_then(|cell| cell.paragraphs.get_mut(cell_para))
            .and_then(|p| p.controls.get_mut(form_ci))
            .and_then(|c| {
                if let Control::Form(ref mut f) = c {
                    Some(f)
                } else {
                    None
                }
            });

        match form {
            Some(f) => {
                apply_form_value(f, value_json);
                let value = f.value;
                let text = f.text.clone();
                let caption = f.caption.clone();
                if let Some(section) = self.document.sections.get_mut(sec) {
                    section.raw_stream = None;
                }
                self.recompose_section(sec);
                Ok(format!(
                    r#"{{"ok":true,"operation":"set-form","container":"cell","sec":{},"para":{},"control":{},"cellIndex":{},"cellPara":{},"value":{},"text":"{}","caption":"{}"}}"#,
                    sec,
                    table_para,
                    form_ci,
                    cell_idx,
                    cell_para,
                    value,
                    escape_json(&text),
                    escape_json(&caption),
                ))
            }
            None => Ok(r#"{"ok":false,"error":"cell form not found"}"#.to_string()),
        }
    }

    /// cell_path 기반으로 셀 내부 양식 개체 상세 정보를 반환한다.
    pub fn get_cell_form_object_info_native(
        &self,
        sec: usize,
        parent_para: usize,
        cell_path: &[(usize, usize, usize)],
        ci: usize,
    ) -> Result<String, crate::error::HwpError> {
        let paragraph = self.resolve_paragraph_by_path(sec, parent_para, cell_path)?;
        let control = paragraph.controls.get(ci);

        match control {
            Some(Control::Form(f)) => {
                let mut json = format_form_info_json(f, &self.document.extra_streams);
                if let Some(pos) = json.rfind('}') {
                    json.insert_str(
                        pos,
                        &format!(
                            r#","container":"cell","paraIdx":{},"controlIdx":{},"cellPath":{}"#,
                            parent_para,
                            ci,
                            cell_path_json(cell_path),
                        ),
                    );
                }
                Ok(json)
            }
            _ => Ok(r#"{"ok":false,"error":"not a cell form object"}"#.to_string()),
        }
    }

    /// cell_path 기반으로 셀 내부 양식 개체 값을 설정한다.
    pub fn set_cell_form_value_by_path_native(
        &mut self,
        sec: usize,
        parent_para: usize,
        cell_path: &[(usize, usize, usize)],
        ci: usize,
        value_json: &str,
    ) -> Result<String, crate::error::HwpError> {
        let section = self
            .document
            .sections
            .get_mut(sec)
            .ok_or_else(|| crate::error::HwpError::RenderError("구역 위치 초과".into()))?;
        section.raw_stream = None;
        let paragraph = resolve_form_cell_paragraph_mut(section, parent_para, cell_path)?;
        let control = paragraph.controls.get_mut(ci);

        match control {
            Some(Control::Form(f)) => {
                apply_form_value(f, value_json);
                let value = f.value;
                let text = f.text.clone();
                let caption = f.caption.clone();
                self.recompose_section(sec);
                Ok(format!(
                    r#"{{"ok":true,"operation":"set-form","container":"cell","sec":{},"para":{},"control":{},"cellPath":{},"value":{},"text":"{}","caption":"{}"}}"#,
                    sec,
                    parent_para,
                    ci,
                    cell_path_json(cell_path),
                    value,
                    escape_json(&text),
                    escape_json(&caption),
                ))
            }
            _ => Ok(r#"{"ok":false,"error":"not a cell form object"}"#.to_string()),
        }
    }

    /// cell_path 기반으로 셀 내부 양식 개체를 삭제한다.
    pub fn delete_cell_form_object_by_path_native(
        &mut self,
        sec: usize,
        parent_para: usize,
        cell_path: &[(usize, usize, usize)],
        ci: usize,
    ) -> Result<String, crate::error::HwpError> {
        let section = self
            .document
            .sections
            .get_mut(sec)
            .ok_or_else(|| crate::error::HwpError::RenderError("구역 위치 초과".into()))?;
        section.raw_stream = None;
        let paragraph = resolve_form_cell_paragraph_mut(section, parent_para, cell_path)?;
        delete_form_from_paragraph(paragraph, ci)?;
        self.recompose_section(sec);
        self.paginate_if_needed();
        self.invalidate_page_tree_cache();
        Ok(format!(
            r#"{{"ok":true,"operation":"delete-form","container":"cell","sec":{},"para":{},"control":{},"cellPath":{}}}"#,
            sec,
            parent_para,
            ci,
            cell_path_json(cell_path),
        ))
    }

    /// 양식 개체 상세 정보를 반환한다 (properties HashMap 포함).
    /// ComboBox인 경우 스크립트에서 추출한 항목 목록도 포함.
    pub fn get_form_object_info_native(
        &self,
        sec: usize,
        para: usize,
        ci: usize,
    ) -> Result<String, crate::error::HwpError> {
        let control = self
            .document
            .sections
            .get(sec)
            .and_then(|s| s.paragraphs.get(para))
            .and_then(|p| p.controls.get(ci));

        match control {
            Some(Control::Form(f)) => Ok(format_form_info_json(f, &self.document.extra_streams)),
            _ => Ok(r#"{"ok":false,"error":"not a form object"}"#.to_string()),
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn insert_form_into_paragraph(
    paragraph: &mut Paragraph,
    char_offset: usize,
    form_type: FormType,
    name: &str,
    caption: &str,
    text: &str,
    width: u32,
    height: u32,
    value: i32,
    enabled: bool,
    properties_json: &str,
) -> Result<usize, crate::error::HwpError> {
    let text_len = paragraph.text.chars().count();
    if char_offset > text_len {
        return Err(crate::error::HwpError::RenderError(format!(
            "문자 오프셋 {}이 문단 길이 {}를 초과합니다.",
            char_offset, text_len
        )));
    }

    let form = FormObject {
        form_type,
        name: name.trim().to_string(),
        caption: caption.to_string(),
        text: text.to_string(),
        width,
        height,
        fore_color: 0,
        back_color: 0x00ff_ffff,
        value,
        enabled,
        properties: parse_form_properties_json(properties_json),
    };

    let mut positions = find_control_text_positions(paragraph);
    let insert_idx = positions
        .iter()
        .position(|pos| *pos > char_offset)
        .unwrap_or(paragraph.controls.len());
    for fr in &mut paragraph.field_ranges {
        if fr.control_idx >= insert_idx {
            fr.control_idx += 1;
        }
    }
    if positions.len() < paragraph.controls.len() {
        positions.resize(paragraph.controls.len(), text_len);
    }
    positions.insert(insert_idx, char_offset);
    paragraph
        .controls
        .insert(insert_idx, Control::Form(Box::new(form)));
    if paragraph.ctrl_data_records.len() >= insert_idx {
        paragraph.ctrl_data_records.insert(insert_idx, None);
    }
    rebuild_offsets_from_control_positions(paragraph, &positions);
    Ok(insert_idx)
}

fn delete_form_from_paragraph(
    paragraph: &mut Paragraph,
    control_idx: usize,
) -> Result<(), crate::error::HwpError> {
    if control_idx >= paragraph.controls.len() {
        return Err(crate::error::HwpError::RenderError(format!(
            "컨트롤 인덱스 {} 범위 초과",
            control_idx
        )));
    }
    if !matches!(paragraph.controls.get(control_idx), Some(Control::Form(_))) {
        return Err(crate::error::HwpError::RenderError(
            "지정된 컨트롤이 양식 개체가 아닙니다".to_string(),
        ));
    }

    let text_len = paragraph.text.chars().count();
    let mut positions = find_control_text_positions(paragraph);
    if positions.len() < paragraph.controls.len() {
        positions.resize(paragraph.controls.len(), text_len);
    }
    if control_idx < positions.len() {
        positions.remove(control_idx);
    }

    paragraph.controls.remove(control_idx);
    if control_idx < paragraph.ctrl_data_records.len() {
        paragraph.ctrl_data_records.remove(control_idx);
    }
    for fr in &mut paragraph.field_ranges {
        if fr.control_idx > control_idx {
            fr.control_idx -= 1;
        }
    }
    rebuild_offsets_from_control_positions(paragraph, &positions);
    Ok(())
}

fn resolve_form_cell_paragraph_mut<'a>(
    section: &'a mut Section,
    parent_para_idx: usize,
    path: &[(usize, usize, usize)],
) -> Result<&'a mut Paragraph, crate::error::HwpError> {
    let mut current_para = section.paragraphs.get_mut(parent_para_idx).ok_or_else(|| {
        crate::error::HwpError::RenderError(format!("문단 인덱스 {} 범위 초과", parent_para_idx))
    })?;
    for (i, &(ctrl_idx, cell_idx, cell_para_idx)) in path.iter().enumerate() {
        let ctrl = current_para.controls.get_mut(ctrl_idx).ok_or_else(|| {
            crate::error::HwpError::RenderError(format!(
                "경로[{}]: controls[{}] 범위 초과",
                i, ctrl_idx
            ))
        })?;
        current_para = match ctrl {
            Control::Table(t) => {
                let cell = t.cells.get_mut(cell_idx).ok_or_else(|| {
                    crate::error::HwpError::RenderError(format!(
                        "경로[{}]: cells[{}] 범위 초과",
                        i, cell_idx
                    ))
                })?;
                cell.paragraphs.get_mut(cell_para_idx).ok_or_else(|| {
                    crate::error::HwpError::RenderError(format!(
                        "경로[{}]: paragraphs[{}] 범위 초과",
                        i, cell_para_idx
                    ))
                })?
            }
            Control::Shape(shape) => {
                if cell_idx != 0 {
                    return Err(crate::error::HwpError::RenderError(format!(
                        "경로[{}]: 글상자의 cell_index는 0이어야 합니다 ({})",
                        i, cell_idx
                    )));
                }
                let text_box = get_textbox_from_shape_mut(shape).ok_or_else(|| {
                    crate::error::HwpError::RenderError(format!(
                        "경로[{}]: controls[{}]가 텍스트 글상자가 아닙니다",
                        i, ctrl_idx
                    ))
                })?;
                text_box.paragraphs.get_mut(cell_para_idx).ok_or_else(|| {
                    crate::error::HwpError::RenderError(format!(
                        "경로[{}]: 글상자문단 {} 범위 초과",
                        i, cell_para_idx
                    ))
                })?
            }
            _ => {
                return Err(crate::error::HwpError::RenderError(format!(
                    "경로[{}]: controls[{}] 가 표/글상자가 아닙니다",
                    i, ctrl_idx
                )))
            }
        };
    }
    Ok(current_para)
}

fn cell_path_json(path: &[(usize, usize, usize)]) -> String {
    let items = path
        .iter()
        .map(|(control, cell, cell_para)| {
            format!(
                r#"{{"controlIndex":{},"cellIndex":{},"cellParaIndex":{}}}"#,
                control, cell, cell_para
            )
        })
        .collect::<Vec<_>>();
    format!("[{}]", items.join(","))
}

fn format_form_info_json(
    f: &crate::model::control::FormObject,
    extra_streams: &[(String, Vec<u8>)],
) -> String {
    let form_type_str = form_type_to_str(f.form_type);
    let props: Vec<String> = f
        .properties
        .iter()
        .map(|(k, v)| format!(r#""{}":"{}""#, escape_json(k), escape_json(v)))
        .collect();
    let props_json = format!("{{{}}}", props.join(","));

    let items_json = if f.form_type == FormType::ComboBox {
        let items = extract_combobox_items_from_script(extra_streams, &f.name);
        if items.is_empty() {
            "[]".to_string()
        } else {
            let arr: Vec<String> = items
                .iter()
                .map(|s| format!(r#""{}""#, escape_json(s)))
                .collect();
            format!("[{}]", arr.join(","))
        }
    } else {
        "[]".to_string()
    };

    format!(
        r#"{{"ok":true,"formType":"{}","name":"{}","value":{},"text":"{}","caption":"{}","enabled":{},"width":{},"height":{},"foreColor":{},"backColor":{},"properties":{},"items":{}}}"#,
        form_type_str,
        escape_json(&f.name),
        f.value,
        escape_json(&f.text),
        escape_json(&f.caption),
        f.enabled,
        f.width,
        f.height,
        f.fore_color,
        f.back_color,
        props_json,
        items_json,
    )
}

/// form value/text/caption 적용 헬퍼
fn apply_form_value(f: &mut crate::model::control::FormObject, value_json: &str) {
    if let Some(v) = extract_json_int(value_json, "value") {
        f.value = v;
    }
    if let Some(t) = extract_json_string(value_json, "text") {
        f.text = t;
    }
    if let Some(c) = extract_json_string(value_json, "caption") {
        f.caption = c;
    }
}

fn parse_form_properties_json(json: &str) -> HashMap<String, String> {
    let mut properties = HashMap::new();
    let Ok(value) = serde_json::from_str::<serde_json::Value>(json) else {
        return properties;
    };
    let Some(object) = value.as_object() else {
        return properties;
    };
    for (key, value) in object {
        let value_string = match value {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Bool(v) => {
                if *v {
                    "1".to_string()
                } else {
                    "0".to_string()
                }
            }
            serde_json::Value::Number(n) => n.to_string(),
            _ => value.to_string(),
        };
        properties.insert(key.clone(), value_string);
    }
    properties
}

fn rebuild_offsets_from_control_positions(para: &mut Paragraph, positions: &[usize]) {
    let text_chars: Vec<char> = para.text.chars().collect();
    let text_len = text_chars.len();
    if text_len == 0 {
        para.char_offsets = Vec::new();
        return;
    }

    let mut controls_at = vec![0usize; text_len + 1];
    for pos in positions {
        let idx = (*pos).min(text_len);
        controls_at[idx] += 1;
    }

    let mut offset = 0u32;
    let mut new_offsets = Vec::with_capacity(text_len);
    for (i, ch) in text_chars.iter().enumerate() {
        offset += controls_at[i] as u32 * 8;
        new_offsets.push(offset);
        let char_size = match *ch {
            '\t' => 8,
            '\n' | '\u{00A0}' => 1,
            c => {
                let mut buf = [0u16; 2];
                c.encode_utf16(&mut buf).len() as u32
            }
        };
        offset += char_size;
    }

    para.char_offsets = new_offsets;
}

/// 렌더 트리를 재귀 순회하여 좌표에 해당하는 FormObject 노드를 찾는다.
fn find_form_node_at(
    node: &RenderNode,
    x: f64,
    y: f64,
) -> Option<(&FormObjectNode, (f64, f64, f64, f64))> {
    // 자식 먼저 (더 구체적인 노드 우선)
    for child in &node.children {
        if let Some(result) = find_form_node_at(child, x, y) {
            return Some(result);
        }
    }
    // 현재 노드 확인
    if let RenderNodeType::FormObject(ref form) = node.node_type {
        let b = &node.bbox;
        if x >= b.x && x <= b.x + b.width && y >= b.y && y <= b.y + b.height {
            return Some((form, (b.x, b.y, b.width, b.height)));
        }
    }
    None
}

fn form_type_to_str(ft: FormType) -> &'static str {
    match ft {
        FormType::PushButton => "PushButton",
        FormType::CheckBox => "CheckBox",
        FormType::ComboBox => "ComboBox",
        FormType::RadioButton => "RadioButton",
        FormType::Edit => "Edit",
    }
}

fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// 간단한 JSON에서 정수값 추출: `"key":123`
fn extract_json_int(json: &str, key: &str) -> Option<i32> {
    let pattern = format!(r#""{}":"#, key);
    if let Some(pos) = json.find(&pattern) {
        let start = pos + pattern.len();
        let rest = &json[start..];
        let end = rest
            .find(|c: char| !c.is_ascii_digit() && c != '-')
            .unwrap_or(rest.len());
        rest[..end].parse().ok()
    } else {
        None
    }
}

/// 간단한 JSON에서 문자열값 추출: `"key":"value"`
fn extract_json_string(json: &str, key: &str) -> Option<String> {
    let pattern = format!(r#""{}":""#, key);
    if let Some(pos) = json.find(&pattern) {
        let start = pos + pattern.len();
        let rest = &json[start..];
        // 이스케이프되지 않은 닫는 따옴표 찾기
        let mut end = 0;
        let chars: Vec<char> = rest.chars().collect();
        while end < chars.len() {
            if chars[end] == '"' && (end == 0 || chars[end - 1] != '\\') {
                break;
            }
            end += 1;
        }
        Some(chars[..end].iter().collect())
    } else {
        None
    }
}

/// extra_streams에서 Scripts/DefaultJScript 스트림을 찾아 디코딩한다.
/// HWP 스크립트는 zlib 압축 + UTF-16LE로 저장됨.
fn decode_hwp_script(extra_streams: &[(String, Vec<u8>)]) -> Option<String> {
    let data = extra_streams
        .iter()
        .find(|(path, _)| path == "/Scripts/DefaultJScript" || path == "Scripts/DefaultJScript")
        .map(|(_, data)| data)?;

    if data.is_empty() {
        return None;
    }

    // zlib 해제 (raw deflate, no header)
    use std::io::Read;
    let mut decoder = flate2::read::DeflateDecoder::new(&data[..]);
    let mut decompressed = Vec::new();
    if decoder.read_to_end(&mut decompressed).is_err() {
        return None;
    }

    // UTF-16LE 디코딩
    if decompressed.len() < 2 {
        return None;
    }
    let u16s: Vec<u16> = decompressed
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .collect();
    Some(String::from_utf16_lossy(&u16s))
}

/// 스크립트에서 ComboBox InsertString 패턴을 추출하여 항목 목록을 반환한다.
///
/// 패턴: `컨트롤이름.InsertString("항목텍스트", 인덱스);`
fn extract_combobox_items_from_script(
    extra_streams: &[(String, Vec<u8>)],
    control_name: &str,
) -> Vec<String> {
    let script = match decode_hwp_script(extra_streams) {
        Some(s) => s,
        None => return Vec::new(),
    };

    let mut items: Vec<(usize, String)> = Vec::new();

    // 패턴: ControlName.InsertString("text", index)
    // 또는: ControlName.InsertString("text",index)
    let prefix = format!("{}.InsertString(", control_name);
    for line in script.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix(&prefix) {
            // rest: `"봄",0);` 또는 `"봄", 0);`
            if let Some((text, idx)) = parse_insert_string_args(rest) {
                items.push((idx, text));
            }
        }
    }

    // 인덱스 순으로 정렬
    items.sort_by_key(|(idx, _)| *idx);
    items.into_iter().map(|(_, text)| text).collect()
}

/// InsertString 인자 파싱: `"텍스트", 인덱스);` → (텍스트, 인덱스)
fn parse_insert_string_args(args: &str) -> Option<(String, usize)> {
    // "텍스트" 추출
    let rest = args.strip_prefix('"')?;
    let end_quote = rest.find('"')?;
    let text = rest[..end_quote].to_string();
    let after_quote = &rest[end_quote + 1..];

    // , 인덱스 추출
    let after_comma = after_quote.trim_start().strip_prefix(',')?;
    let idx_str: String = after_comma
        .trim()
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    let idx = idx_str.parse().unwrap_or(0);

    Some((text, idx))
}
