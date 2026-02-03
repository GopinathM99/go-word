//! Content Controls (SDT) Export for DOCX
//!
//! Writes ContentControl structures to OOXML w:sdt elements.
//! Supports all standard content control types and preserves
//! unknown elements for round-trip fidelity.

use doc_model::{
    CalendarType, CheckboxSymbol, ContentControl, ContentControlAppearance, ContentControlType,
    ControlProperties, DataBinding, ListItem,
};

// =============================================================================
// Content Control Writer
// =============================================================================

/// Writer for content controls in DOCX export
pub struct ContentControlWriter;

impl ContentControlWriter {
    /// Create a new content control writer
    pub fn new() -> Self {
        Self
    }

    /// Write a content control to XML string
    pub fn write(&self, control: &ContentControl, content_xml: &str) -> String {
        let mut xml = String::new();

        xml.push_str("<w:sdt>");

        // Write properties (w:sdtPr)
        self.write_properties(&mut xml, control);

        // Write end run if needed (for inline SDTs)
        xml.push_str("<w:sdtEndPr/>");

        // Write content (w:sdtContent)
        xml.push_str("<w:sdtContent>");
        xml.push_str(content_xml);
        xml.push_str("</w:sdtContent>");

        xml.push_str("</w:sdt>");

        xml
    }

    /// Write SDT properties (w:sdtPr)
    fn write_properties(&self, xml: &mut String, control: &ContentControl) {
        xml.push_str("<w:sdtPr>");

        // Write ID
        if let Some(sdt_id) = control.sdt_id {
            xml.push_str(&format!("<w:id w:val=\"{}\"/>", sdt_id));
        }

        // Write tag
        if !control.tag.is_empty() {
            xml.push_str(&format!(
                "<w:tag w:val=\"{}\"/>",
                escape_xml(&control.tag)
            ));
        }

        // Write alias/title
        if !control.title.is_empty() {
            xml.push_str(&format!(
                "<w:alias w:val=\"{}\"/>",
                escape_xml(&control.title)
            ));
        }

        // Write lock settings
        if control.locked && control.contents_locked {
            xml.push_str("<w:lock w:val=\"sdtContentLocked\"/>");
        } else if control.locked {
            xml.push_str("<w:lock w:val=\"sdtLocked\"/>");
        } else if control.contents_locked {
            xml.push_str("<w:lock w:val=\"contentLocked\"/>");
        }

        // Write placeholder
        if !control.placeholder.is_empty() {
            xml.push_str("<w:placeholder>");
            xml.push_str(&format!(
                "<w:docPart w:val=\"{}\"/>",
                escape_xml(&control.placeholder)
            ));
            xml.push_str("</w:placeholder>");
        }

        // Write data binding
        if let Some(ref binding) = control.data_binding {
            self.write_data_binding(xml, binding);
        }

        // Write appearance
        if control.appearance != ContentControlAppearance::BoundingBox {
            xml.push_str(&format!(
                "<w15:appearance w15:val=\"{}\"/>",
                control.appearance.ooxml_value()
            ));
        }

        // Write color
        if let Some(ref color) = control.color {
            xml.push_str(&format!("<w15:color w:val=\"{}\"/>", escape_xml(color)));
        }

        // Write temporary flag
        if control.temporary {
            xml.push_str("<w:temporary/>");
        }

        // Write type-specific properties
        self.write_type_properties(xml, control);

        // Write unknown/preserved elements for round-trip
        for unknown in &control.unknown_elements {
            xml.push_str(unknown);
        }

        xml.push_str("</w:sdtPr>");
    }

    /// Write data binding element
    fn write_data_binding(&self, xml: &mut String, binding: &DataBinding) {
        xml.push_str("<w:dataBinding");

        if !binding.xpath.is_empty() {
            xml.push_str(&format!(" w:xpath=\"{}\"", escape_xml(&binding.xpath)));
        }

        if !binding.store_id.is_empty() {
            xml.push_str(&format!(
                " w:storeItemID=\"{}\"",
                escape_xml(&binding.store_id)
            ));
        }

        let prefix_str = binding.prefix_mappings_string();
        if !prefix_str.is_empty() {
            xml.push_str(&format!(" w:prefixMappings=\"{}\"", escape_xml(&prefix_str)));
        }

        xml.push_str("/>");
    }

    /// Write type-specific properties
    fn write_type_properties(&self, xml: &mut String, control: &ContentControl) {
        match &control.properties {
            ControlProperties::RichText {
                allow_carriage_return,
            } => {
                // RichText is the default, no specific element needed
                // unless we need to specify carriage return behavior
                if !*allow_carriage_return {
                    xml.push_str("<w:richText/>");
                }
            }

            ControlProperties::PlainText { multiline, max_chars } => {
                if *multiline {
                    xml.push_str("<w:text w:multiLine=\"true\"/>");
                } else {
                    xml.push_str("<w:text/>");
                }
            }

            ControlProperties::Checkbox {
                checked,
                checked_symbol,
                unchecked_symbol,
            } => {
                self.write_checkbox(xml, *checked, checked_symbol, unchecked_symbol);
            }

            ControlProperties::DropdownList {
                items,
                selected_index,
            } => {
                self.write_dropdown_list(xml, items, *selected_index);
            }

            ControlProperties::ComboBox { items, text } => {
                self.write_combo_box(xml, items, text);
            }

            ControlProperties::DatePicker {
                date,
                format,
                calendar_type,
                storage_format,
                locale,
            } => {
                self.write_date_picker(xml, date, format, calendar_type, storage_format, locale);
            }

            ControlProperties::Picture { .. } => {
                xml.push_str("<w:picture/>");
            }

            ControlProperties::RepeatingSection {
                section_title,
                min_count,
                max_count,
                allow_insert,
                allow_delete,
            } => {
                self.write_repeating_section(
                    xml,
                    section_title,
                    *allow_insert,
                    *allow_delete,
                );
            }

            ControlProperties::RepeatingSectionItem => {
                xml.push_str("<w15:repeatingSectionItem/>");
            }

            ControlProperties::BuildingBlockGallery { category, gallery } => {
                self.write_building_block_gallery(xml, gallery, category);
            }

            ControlProperties::Citation { .. } => {
                xml.push_str("<w:citation/>");
            }

            ControlProperties::Bibliography { .. } => {
                xml.push_str("<w:bibliography/>");
            }

            ControlProperties::Equation => {
                xml.push_str("<w:equation/>");
            }

            ControlProperties::Group => {
                xml.push_str("<w:group/>");
            }
        }
    }

    /// Write checkbox properties
    fn write_checkbox(
        &self,
        xml: &mut String,
        checked: bool,
        checked_symbol: &CheckboxSymbol,
        unchecked_symbol: &CheckboxSymbol,
    ) {
        xml.push_str("<w14:checkbox>");

        xml.push_str(&format!(
            "<w14:checked w14:val=\"{}\"/>",
            if checked { "1" } else { "0" }
        ));

        // Write checked state
        xml.push_str("<w14:checkedState");
        xml.push_str(&format!(
            " w14:val=\"{:04X}\"",
            checked_symbol.character as u32
        ));
        if let Some(ref font) = checked_symbol.font {
            xml.push_str(&format!(" w14:font=\"{}\"", escape_xml(font)));
        }
        xml.push_str("/>");

        // Write unchecked state
        xml.push_str("<w14:uncheckedState");
        xml.push_str(&format!(
            " w14:val=\"{:04X}\"",
            unchecked_symbol.character as u32
        ));
        if let Some(ref font) = unchecked_symbol.font {
            xml.push_str(&format!(" w14:font=\"{}\"", escape_xml(font)));
        }
        xml.push_str("/>");

        xml.push_str("</w14:checkbox>");
    }

    /// Write dropdown list properties
    fn write_dropdown_list(
        &self,
        xml: &mut String,
        items: &[ListItem],
        selected_index: Option<usize>,
    ) {
        xml.push_str("<w:dropDownList");
        if let Some(idx) = selected_index {
            if let Some(item) = items.get(idx) {
                xml.push_str(&format!(" w:lastValue=\"{}\"", escape_xml(&item.value)));
            }
        }
        xml.push_str(">");

        for item in items {
            xml.push_str(&format!(
                "<w:listItem w:displayText=\"{}\" w:value=\"{}\"/>",
                escape_xml(&item.display_text),
                escape_xml(&item.value)
            ));
        }

        xml.push_str("</w:dropDownList>");
    }

    /// Write combo box properties
    fn write_combo_box(&self, xml: &mut String, items: &[ListItem], text: &str) {
        xml.push_str("<w:comboBox");
        if !text.is_empty() {
            xml.push_str(&format!(" w:lastValue=\"{}\"", escape_xml(text)));
        }
        xml.push_str(">");

        for item in items {
            xml.push_str(&format!(
                "<w:listItem w:displayText=\"{}\" w:value=\"{}\"/>",
                escape_xml(&item.display_text),
                escape_xml(&item.value)
            ));
        }

        xml.push_str("</w:comboBox>");
    }

    /// Write date picker properties
    fn write_date_picker(
        &self,
        xml: &mut String,
        date: &Option<chrono::NaiveDate>,
        format: &str,
        calendar_type: &CalendarType,
        storage_format: &Option<String>,
        locale: &Option<String>,
    ) {
        xml.push_str("<w:date");
        if let Some(d) = date {
            xml.push_str(&format!(
                " w:fullDate=\"{}T00:00:00Z\"",
                d.format("%Y-%m-%d")
            ));
        }
        xml.push_str(">");

        xml.push_str(&format!(
            "<w:dateFormat w:val=\"{}\"/>",
            escape_xml(format)
        ));

        if let Some(ref loc) = locale {
            xml.push_str(&format!("<w:lid w:val=\"{}\"/>", escape_xml(loc)));
        }

        if let Some(ref sf) = storage_format {
            xml.push_str(&format!(
                "<w:storeMappedDataAs w:val=\"{}\"/>",
                escape_xml(sf)
            ));
        }

        xml.push_str(&format!(
            "<w:calendar w:val=\"{}\"/>",
            calendar_type.ooxml_value()
        ));

        xml.push_str("</w:date>");
    }

    /// Write repeating section properties
    fn write_repeating_section(
        &self,
        xml: &mut String,
        section_title: &str,
        allow_insert: bool,
        allow_delete: bool,
    ) {
        xml.push_str("<w15:repeatingSection>");

        if !section_title.is_empty() {
            xml.push_str(&format!(
                "<w15:sectionTitle w15:val=\"{}\"/>",
                escape_xml(section_title)
            ));
        }

        if !allow_insert || !allow_delete {
            xml.push_str("<w15:doNotAllowInsertDeleteSection w15:val=\"true\"/>");
        }

        xml.push_str("</w15:repeatingSection>");
    }

    /// Write building block gallery properties
    fn write_building_block_gallery(&self, xml: &mut String, gallery: &str, category: &str) {
        xml.push_str("<w:docPartList>");

        if !gallery.is_empty() {
            xml.push_str(&format!(
                "<w:docPartGallery w:val=\"{}\"/>",
                escape_xml(gallery)
            ));
        }

        if !category.is_empty() {
            xml.push_str(&format!(
                "<w:docPartCategory w:val=\"{}\"/>",
                escape_xml(category)
            ));
        }

        xml.push_str("</w:docPartList>");
    }

    /// Write a simple content control wrapping text
    pub fn write_simple_text_control(&self, control: &ContentControl, text: &str) -> String {
        let content_xml = format!(
            "<w:p><w:r><w:t>{}</w:t></w:r></w:p>",
            escape_xml(text)
        );
        self.write(control, &content_xml)
    }

    /// Write just the SDT properties without content (for streaming)
    pub fn write_properties_only(&self, control: &ContentControl) -> String {
        let mut xml = String::new();
        self.write_properties(&mut xml, control);
        xml
    }
}

impl Default for ContentControlWriter {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Escape XML special characters
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use doc_model::ValidationRule;

    #[test]
    fn test_writer_creation() {
        let writer = ContentControlWriter::new();
        // Writer should be created without error
    }

    #[test]
    fn test_write_plain_text_control() {
        let writer = ContentControlWriter::new();
        let control = ContentControl::plain_text()
            .with_tag("test_tag")
            .with_title("Test Title");

        let xml = writer.write_simple_text_control(&control, "Hello");

        assert!(xml.contains("<w:sdt>"));
        assert!(xml.contains("</w:sdt>"));
        assert!(xml.contains("<w:sdtPr>"));
        assert!(xml.contains("<w:tag w:val=\"test_tag\"/>"));
        assert!(xml.contains("<w:alias w:val=\"Test Title\"/>"));
        assert!(xml.contains("<w:text/>"));
        assert!(xml.contains("<w:sdtContent>"));
        assert!(xml.contains("Hello"));
    }

    #[test]
    fn test_write_checkbox_control() {
        let writer = ContentControlWriter::new();
        let mut control = ContentControl::checkbox();
        control.set_checked(true);

        let xml = writer.write(&control, "");

        assert!(xml.contains("<w14:checkbox>"));
        assert!(xml.contains("<w14:checked w14:val=\"1\"/>"));
        assert!(xml.contains("<w14:checkedState"));
        assert!(xml.contains("<w14:uncheckedState"));
        assert!(xml.contains("</w14:checkbox>"));
    }

    #[test]
    fn test_write_dropdown_list() {
        let writer = ContentControlWriter::new();
        let mut control = ContentControl::dropdown_list();
        control.add_list_item(ListItem::new("Option 1"));
        control.add_list_item(ListItem::with_value("Option 2", "opt2"));
        control.set_selected_index(Some(0));

        let xml = writer.write(&control, "");

        assert!(xml.contains("<w:dropDownList"));
        assert!(xml.contains("<w:listItem w:displayText=\"Option 1\" w:value=\"Option 1\"/>"));
        assert!(xml.contains("<w:listItem w:displayText=\"Option 2\" w:value=\"opt2\"/>"));
        assert!(xml.contains("</w:dropDownList>"));
    }

    #[test]
    fn test_write_combo_box() {
        let writer = ContentControlWriter::new();
        let mut control = ContentControl::combo_box();
        control.add_list_item(ListItem::new("Item 1"));

        let xml = writer.write(&control, "");

        assert!(xml.contains("<w:comboBox"));
        assert!(xml.contains("<w:listItem"));
        assert!(xml.contains("</w:comboBox>"));
    }

    #[test]
    fn test_write_date_picker() {
        let writer = ContentControlWriter::new();
        let mut control = ContentControl::date_picker();
        let date = chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        control.set_selected_date(Some(date));

        let xml = writer.write(&control, "");

        assert!(xml.contains("<w:date"));
        assert!(xml.contains("fullDate=\"2025-01-15T00:00:00Z\""));
        assert!(xml.contains("<w:dateFormat"));
        assert!(xml.contains("<w:calendar"));
        assert!(xml.contains("</w:date>"));
    }

    #[test]
    fn test_write_picture_control() {
        let writer = ContentControlWriter::new();
        let control = ContentControl::picture();

        let xml = writer.write(&control, "");

        assert!(xml.contains("<w:picture/>"));
    }

    #[test]
    fn test_write_rich_text_control() {
        let writer = ContentControlWriter::new();
        let control = ContentControl::rich_text();

        let xml = writer.write(&control, "<w:p><w:r><w:t>Rich text</w:t></w:r></w:p>");

        assert!(xml.contains("<w:sdt>"));
        assert!(xml.contains("Rich text"));
        // RichText shouldn't have explicit type element (it's default)
    }

    #[test]
    fn test_write_lock_settings() {
        let writer = ContentControlWriter::new();

        // Both locked
        let control = ContentControl::plain_text()
            .with_locked(true)
            .with_contents_locked(true);
        let xml = writer.write(&control, "");
        assert!(xml.contains("<w:lock w:val=\"sdtContentLocked\"/>"));

        // Only SDT locked
        let control = ContentControl::plain_text().with_locked(true);
        let xml = writer.write(&control, "");
        assert!(xml.contains("<w:lock w:val=\"sdtLocked\"/>"));

        // Only content locked
        let control = ContentControl::plain_text().with_contents_locked(true);
        let xml = writer.write(&control, "");
        assert!(xml.contains("<w:lock w:val=\"contentLocked\"/>"));
    }

    #[test]
    fn test_write_data_binding() {
        let writer = ContentControlWriter::new();
        let binding = DataBinding::with_store("/root/element", "{12345}");
        let control = ContentControl::plain_text().with_data_binding(binding);

        let xml = writer.write(&control, "");

        assert!(xml.contains("<w:dataBinding"));
        assert!(xml.contains("w:xpath=\"/root/element\""));
        assert!(xml.contains("w:storeItemID=\"{12345}\""));
    }

    #[test]
    fn test_write_placeholder() {
        let writer = ContentControlWriter::new();
        let control = ContentControl::plain_text().with_placeholder("Enter text here");

        let xml = writer.write(&control, "");

        assert!(xml.contains("<w:placeholder>"));
        assert!(xml.contains("<w:docPart w:val=\"Enter text here\"/>"));
        assert!(xml.contains("</w:placeholder>"));
    }

    #[test]
    fn test_write_sdt_id() {
        let writer = ContentControlWriter::new();
        let mut control = ContentControl::plain_text();
        control.sdt_id = Some(12345);

        let xml = writer.write(&control, "");

        assert!(xml.contains("<w:id w:val=\"12345\"/>"));
    }

    #[test]
    fn test_write_appearance_hidden() {
        let writer = ContentControlWriter::new();
        let mut control = ContentControl::plain_text();
        control.appearance = ContentControlAppearance::Hidden;

        let xml = writer.write(&control, "");

        assert!(xml.contains("<w15:appearance w15:val=\"hidden\"/>"));
    }

    #[test]
    fn test_write_temporary_flag() {
        let writer = ContentControlWriter::new();
        let mut control = ContentControl::plain_text();
        control.temporary = true;

        let xml = writer.write(&control, "");

        assert!(xml.contains("<w:temporary/>"));
    }

    #[test]
    fn test_write_color() {
        let writer = ContentControlWriter::new();
        let mut control = ContentControl::plain_text();
        control.color = Some("FF0000".to_string());

        let xml = writer.write(&control, "");

        assert!(xml.contains("<w15:color w:val=\"FF0000\"/>"));
    }

    #[test]
    fn test_write_repeating_section() {
        let writer = ContentControlWriter::new();
        let control = ContentControl::repeating_section();

        let xml = writer.write(&control, "");

        assert!(xml.contains("<w15:repeatingSection>"));
        assert!(xml.contains("</w15:repeatingSection>"));
    }

    #[test]
    fn test_write_group_control() {
        let writer = ContentControlWriter::new();
        let control = ContentControl::new(ContentControlType::Group);

        let xml = writer.write(&control, "");

        assert!(xml.contains("<w:group/>"));
    }

    #[test]
    fn test_write_citation_control() {
        let writer = ContentControlWriter::new();
        let control = ContentControl::new(ContentControlType::Citation);

        let xml = writer.write(&control, "");

        assert!(xml.contains("<w:citation/>"));
    }

    #[test]
    fn test_write_bibliography_control() {
        let writer = ContentControlWriter::new();
        let control = ContentControl::new(ContentControlType::Bibliography);

        let xml = writer.write(&control, "");

        assert!(xml.contains("<w:bibliography/>"));
    }

    #[test]
    fn test_write_equation_control() {
        let writer = ContentControlWriter::new();
        let control = ContentControl::new(ContentControlType::Equation);

        let xml = writer.write(&control, "");

        assert!(xml.contains("<w:equation/>"));
    }

    #[test]
    fn test_escape_xml_special_chars() {
        assert_eq!(escape_xml("<test>"), "&lt;test&gt;");
        assert_eq!(escape_xml("a & b"), "a &amp; b");
        assert_eq!(escape_xml("\"quoted\""), "&quot;quoted&quot;");
        assert_eq!(escape_xml("it's"), "it&apos;s");
    }

    #[test]
    fn test_write_properties_only() {
        let writer = ContentControlWriter::new();
        let control = ContentControl::plain_text().with_tag("test");

        let props_xml = writer.write_properties_only(&control);

        assert!(props_xml.starts_with("<w:sdtPr>"));
        assert!(props_xml.ends_with("</w:sdtPr>"));
        assert!(props_xml.contains("<w:tag w:val=\"test\"/>"));
    }

    #[test]
    fn test_write_simple_text_control() {
        let writer = ContentControlWriter::new();
        let control = ContentControl::plain_text();

        let xml = writer.write_simple_text_control(&control, "Hello World");

        assert!(xml.contains("<w:t>Hello World</w:t>"));
    }

    #[test]
    fn test_write_checkbox_unchecked() {
        let writer = ContentControlWriter::new();
        let control = ContentControl::checkbox();

        let xml = writer.write(&control, "");

        assert!(xml.contains("<w14:checked w14:val=\"0\"/>"));
    }

    #[test]
    fn test_write_multiline_text() {
        let writer = ContentControlWriter::new();
        let mut control = ContentControl::plain_text();
        control.properties = ControlProperties::PlainText {
            multiline: true,
            max_chars: None,
        };

        let xml = writer.write(&control, "");

        assert!(xml.contains("<w:text w:multiLine=\"true\"/>"));
    }

    #[test]
    fn test_write_unknown_elements_preserved() {
        let writer = ContentControlWriter::new();
        let mut control = ContentControl::plain_text();
        control.unknown_elements.push("<w:customElement w:val=\"test\"/>".to_string());

        let xml = writer.write(&control, "");

        assert!(xml.contains("<w:customElement w:val=\"test\"/>"));
    }

    #[test]
    fn test_write_complete_control() {
        let writer = ContentControlWriter::new();
        let binding = DataBinding::with_store("/data/name", "{GUID}");
        let mut control = ContentControl::plain_text()
            .with_tag("name_field")
            .with_title("Name")
            .with_placeholder("Enter name")
            .with_locked(true)
            .with_data_binding(binding);
        control.sdt_id = Some(999);
        control.color = Some("0000FF".to_string());

        let xml = writer.write_simple_text_control(&control, "John Doe");

        assert!(xml.contains("<w:id w:val=\"999\"/>"));
        assert!(xml.contains("<w:tag w:val=\"name_field\"/>"));
        assert!(xml.contains("<w:alias w:val=\"Name\"/>"));
        assert!(xml.contains("<w:lock w:val=\"sdtLocked\"/>"));
        assert!(xml.contains("<w:placeholder>"));
        assert!(xml.contains("<w:dataBinding"));
        assert!(xml.contains("w:xpath=\"/data/name\""));
        assert!(xml.contains("John Doe"));
    }

    #[test]
    fn test_write_calendar_types() {
        let writer = ContentControlWriter::new();

        let calendars = vec![
            CalendarType::Gregorian,
            CalendarType::Hebrew,
            CalendarType::Hijri,
            CalendarType::Japan,
        ];

        for cal in calendars {
            let mut control = ContentControl::date_picker();
            control.properties = ControlProperties::DatePicker {
                date: None,
                format: "M/d/yyyy".to_string(),
                calendar_type: cal,
                storage_format: None,
                locale: None,
            };

            let xml = writer.write(&control, "");
            assert!(xml.contains(&format!(
                "<w:calendar w:val=\"{}\"/>",
                cal.ooxml_value()
            )));
        }
    }
}
