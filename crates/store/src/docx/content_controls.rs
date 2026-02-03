//! Content Controls (SDT) Import for DOCX
//!
//! Parses w:sdt elements from OOXML documents into ContentControl structures.
//! Supports all standard content control types including:
//! - Rich text, Plain text
//! - Checkbox, Dropdown list, Combo box
//! - Date picker, Picture
//! - Repeating sections
//!
//! Preserves unknown elements for round-trip fidelity.

use crate::docx::error::{DocxError, DocxResult};
use crate::docx::reader::XmlParser;
use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;
use std::collections::HashMap;

// =============================================================================
// Parsed Structures
// =============================================================================

/// Parsed content control from DOCX
#[derive(Debug, Clone)]
pub struct ParsedContentControl {
    /// SDT ID (w:id)
    pub sdt_id: Option<i64>,
    /// Control type determined from properties
    pub control_type: ParsedControlType,
    /// Tag (w:tag)
    pub tag: String,
    /// Title (w:alias or w15:appearance title)
    pub title: String,
    /// Placeholder text
    pub placeholder: String,
    /// Lock settings (w:lock)
    pub lock: LockSettings,
    /// Data binding (w:dataBinding)
    pub data_binding: Option<ParsedDataBinding>,
    /// Type-specific properties
    pub type_properties: ParsedTypeProperties,
    /// Appearance setting
    pub appearance: String,
    /// Color (w:color or w15:color)
    pub color: Option<String>,
    /// Temporary flag (w:temporary)
    pub temporary: bool,
    /// Raw SDT content XML for preserving children
    pub content_xml: String,
    /// Unknown/preserved property elements
    pub unknown_properties: Vec<String>,
}

impl Default for ParsedContentControl {
    fn default() -> Self {
        Self {
            sdt_id: None,
            control_type: ParsedControlType::RichText,
            tag: String::new(),
            title: String::new(),
            placeholder: String::new(),
            lock: LockSettings::default(),
            data_binding: None,
            type_properties: ParsedTypeProperties::None,
            appearance: String::new(),
            color: None,
            temporary: false,
            content_xml: String::new(),
            unknown_properties: Vec::new(),
        }
    }
}

/// Parsed control type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParsedControlType {
    RichText,
    PlainText,
    Checkbox,
    DropdownList,
    ComboBox,
    DatePicker,
    Picture,
    RepeatingSection,
    RepeatingSectionItem,
    BuildingBlockGallery,
    Citation,
    Bibliography,
    Equation,
    Group,
}

/// Lock settings for content control
#[derive(Debug, Clone, Default)]
pub struct LockSettings {
    /// Cannot delete the control
    pub sdt_locked: bool,
    /// Cannot edit contents
    pub content_locked: bool,
}

/// Parsed data binding configuration
#[derive(Debug, Clone)]
pub struct ParsedDataBinding {
    /// XPath expression
    pub xpath: String,
    /// Prefix mappings as OOXML string
    pub prefix_mappings: String,
    /// Store item ID
    pub store_item_id: String,
}

/// Type-specific properties parsed from DOCX
#[derive(Debug, Clone)]
pub enum ParsedTypeProperties {
    /// No type-specific properties
    None,
    /// Plain text properties
    PlainText {
        multiline: bool,
    },
    /// Checkbox properties
    Checkbox {
        checked: bool,
        checked_state: Option<CheckboxState>,
        unchecked_state: Option<CheckboxState>,
    },
    /// Dropdown list properties
    DropdownList {
        items: Vec<ParsedListItem>,
        last_value: Option<String>,
    },
    /// Combo box properties
    ComboBox {
        items: Vec<ParsedListItem>,
        last_value: Option<String>,
    },
    /// Date picker properties
    DatePicker {
        date_format: String,
        lid: Option<String>,
        storage_mapping_type: Option<String>,
        calendar: String,
        full_date: Option<String>,
    },
    /// Picture properties
    Picture,
    /// Repeating section properties
    RepeatingSection {
        section_title: String,
        do_not_allow_insert_delete_section: bool,
    },
    /// Building block gallery properties
    BuildingBlockGallery {
        gallery: String,
        category: String,
    },
}

/// Checkbox state with font and character
#[derive(Debug, Clone)]
pub struct CheckboxState {
    pub font: String,
    pub value: String,
}

/// Parsed list item for dropdown/combo box
#[derive(Debug, Clone)]
pub struct ParsedListItem {
    pub display_text: String,
    pub value: String,
}

// =============================================================================
// Content Control Parser
// =============================================================================

/// Parser for content controls in DOCX
pub struct ContentControlParser;

impl ContentControlParser {
    /// Create a new content control parser
    pub fn new() -> Self {
        Self
    }

    /// Parse a w:sdt element from XML content
    pub fn parse_sdt(&self, xml: &str) -> DocxResult<ParsedContentControl> {
        let mut reader = XmlParser::from_string(xml);
        let mut buf = Vec::new();
        let mut control = ParsedContentControl::default();
        let mut in_sdt_pr = false;
        let mut in_sdt_content = false;
        let mut content_xml = String::new();

        // Variables to track complex property parsing
        let mut pending_complex_property: Option<(String, String)> = None; // (element_name, accumulated_xml)

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name_binding = e.name();
                    let name_ref = name_binding.as_ref();
                    let name_str = std::str::from_utf8(name_ref).unwrap_or("");

                    if XmlParser::matches_element(name_ref, "sdtPr") {
                        in_sdt_pr = true;
                    } else if XmlParser::matches_element(name_ref, "sdtContent") {
                        in_sdt_content = true;
                    } else if in_sdt_pr {
                        // Start tracking a complex property
                        let mut prop_xml = format!("<{}", name_str);
                        for attr in e.attributes().filter_map(|a| a.ok()) {
                            let key = std::str::from_utf8(attr.key.as_ref()).unwrap_or("");
                            let val = String::from_utf8_lossy(&attr.value);
                            prop_xml.push_str(&format!(" {}=\"{}\"", key, val));
                        }
                        prop_xml.push('>');
                        pending_complex_property = Some((name_str.to_string(), prop_xml));
                    } else if in_sdt_content {
                        content_xml.push_str(&format!("<{}", name_str));
                        for attr in e.attributes().filter_map(|a| a.ok()) {
                            let key = std::str::from_utf8(attr.key.as_ref()).unwrap_or("");
                            let val = String::from_utf8_lossy(&attr.value);
                            content_xml.push_str(&format!(" {}=\"{}\"", key, val));
                        }
                        content_xml.push('>');
                    }
                }
                Ok(Event::Empty(ref e)) => {
                    let name_binding = e.name();
                    let name_ref = name_binding.as_ref();
                    let name_str = std::str::from_utf8(name_ref).unwrap_or("");

                    if let Some((_, ref mut prop_xml)) = pending_complex_property {
                        // Add to complex property
                        prop_xml.push_str(&format!("<{}", name_str));
                        for attr in e.attributes().filter_map(|a| a.ok()) {
                            let key = std::str::from_utf8(attr.key.as_ref()).unwrap_or("");
                            let val = String::from_utf8_lossy(&attr.value);
                            prop_xml.push_str(&format!(" {}=\"{}\"", key, val));
                        }
                        prop_xml.push_str("/>");
                    } else if in_sdt_pr {
                        self.parse_sdt_property_empty(e, &mut control)?;
                    } else if in_sdt_content {
                        content_xml.push_str(&format!("<{}", name_str));
                        for attr in e.attributes().filter_map(|a| a.ok()) {
                            let key = std::str::from_utf8(attr.key.as_ref()).unwrap_or("");
                            let val = String::from_utf8_lossy(&attr.value);
                            content_xml.push_str(&format!(" {}=\"{}\"", key, val));
                        }
                        content_xml.push_str("/>");
                    }
                }
                Ok(Event::End(ref e)) => {
                    let name_binding = e.name();
                    let name_ref = name_binding.as_ref();
                    let name_str = std::str::from_utf8(name_ref).unwrap_or("");

                    if XmlParser::matches_element(name_ref, "sdtPr") {
                        in_sdt_pr = false;
                    } else if XmlParser::matches_element(name_ref, "sdtContent") {
                        in_sdt_content = false;
                    } else if XmlParser::matches_element(name_ref, "sdt") {
                        break;
                    } else if let Some((ref prop_name, ref mut prop_xml)) = pending_complex_property {
                        prop_xml.push_str(&format!("</{}>", name_str));
                        // Check if this is the end of the complex property
                        if name_str.ends_with(prop_name) || name_str == prop_name {
                            let full_xml = prop_xml.clone();
                            self.parse_complex_property(&full_xml, &mut control)?;
                            pending_complex_property = None;
                        }
                    } else if in_sdt_content {
                        content_xml.push_str(&format!("</{}>", name_str));
                    }
                }
                Ok(Event::Text(ref e)) => {
                    if let Some((_, ref mut prop_xml)) = pending_complex_property {
                        let text = e.unescape().unwrap_or_default();
                        prop_xml.push_str(&escape_xml(&text));
                    } else if in_sdt_content {
                        let text = e.unescape().unwrap_or_default();
                        content_xml.push_str(&escape_xml(&text));
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(DocxError::XmlParse(format!("Error parsing SDT: {}", e))),
                _ => {}
            }
            buf.clear();
        }

        control.content_xml = content_xml;
        Ok(control)
    }

    /// Parse a complex property from its XML representation
    fn parse_complex_property(&self, xml: &str, control: &mut ParsedContentControl) -> DocxResult<()> {
        // Determine property type from opening tag
        if xml.starts_with("<w:placeholder") || xml.starts_with("<placeholder") {
            // Parse placeholder
            if let Some(start) = xml.find("w:val=\"") {
                let rest = &xml[start + 7..];
                if let Some(end) = rest.find('"') {
                    control.placeholder = rest[..end].to_string();
                }
            }
        } else if xml.starts_with("<w14:checkbox") || xml.starts_with("<checkbox") {
            control.control_type = ParsedControlType::Checkbox;
            let checked = xml.contains("checked") && (xml.contains("val=\"1\"") || xml.contains("val=\"true\""));
            let checked_state = self.parse_checkbox_state_from_xml(xml, "checkedState");
            let unchecked_state = self.parse_checkbox_state_from_xml(xml, "uncheckedState");
            control.type_properties = ParsedTypeProperties::Checkbox {
                checked,
                checked_state,
                unchecked_state,
            };
        } else if xml.starts_with("<w:dropDownList") || xml.starts_with("<dropDownList") {
            control.control_type = ParsedControlType::DropdownList;
            let items = self.parse_list_items_from_xml(xml);
            let last_value = self.extract_attribute_from_xml(xml, "lastValue");
            control.type_properties = ParsedTypeProperties::DropdownList { items, last_value };
        } else if xml.starts_with("<w:comboBox") || xml.starts_with("<comboBox") {
            control.control_type = ParsedControlType::ComboBox;
            let items = self.parse_list_items_from_xml(xml);
            let last_value = self.extract_attribute_from_xml(xml, "lastValue");
            control.type_properties = ParsedTypeProperties::ComboBox { items, last_value };
        } else if xml.starts_with("<w:date") || xml.starts_with("<date") {
            control.control_type = ParsedControlType::DatePicker;
            let full_date = self.extract_attribute_from_xml(xml, "fullDate");
            let date_format = self.extract_nested_val(xml, "dateFormat").unwrap_or_default();
            let lid = self.extract_nested_val(xml, "lid");
            let storage_mapping_type = self.extract_nested_val(xml, "storeMappedDataAs");
            let calendar = self.extract_nested_val(xml, "calendar").unwrap_or("gregorian".to_string());
            control.type_properties = ParsedTypeProperties::DatePicker {
                date_format,
                lid,
                storage_mapping_type,
                calendar,
                full_date,
            };
        } else if xml.starts_with("<w15:repeatingSection") || xml.starts_with("<repeatingSection") {
            control.control_type = ParsedControlType::RepeatingSection;
            let section_title = self.extract_nested_val(xml, "sectionTitle").unwrap_or_default();
            let do_not_allow = xml.contains("doNotAllowInsertDeleteSection");
            control.type_properties = ParsedTypeProperties::RepeatingSection {
                section_title,
                do_not_allow_insert_delete_section: do_not_allow,
            };
        } else if xml.starts_with("<w:docPartList") || xml.starts_with("<docPartList") || xml.starts_with("<w:docPartObj") {
            control.control_type = ParsedControlType::BuildingBlockGallery;
            let gallery = self.extract_nested_val(xml, "docPartGallery").unwrap_or_default();
            let category = self.extract_nested_val(xml, "docPartCategory").unwrap_or_default();
            control.type_properties = ParsedTypeProperties::BuildingBlockGallery { gallery, category };
        } else {
            // Unknown complex property - preserve it
            control.unknown_properties.push(xml.to_string());
        }
        Ok(())
    }

    /// Helper to extract checkbox state from XML
    fn parse_checkbox_state_from_xml(&self, xml: &str, state_name: &str) -> Option<CheckboxState> {
        let search = format!("<w14:{}", state_name);
        let search_alt = format!("<{}", state_name);

        let start_pos = xml.find(&search).or_else(|| xml.find(&search_alt))?;
        let rest = &xml[start_pos..];
        let end_pos = rest.find("/>")?;
        let element = &rest[..end_pos + 2];

        let val = self.extract_attribute_from_str(element, "val")?;
        let font = self.extract_attribute_from_str(element, "font").unwrap_or_default();

        Some(CheckboxState { font, value: val })
    }

    /// Helper to parse list items from XML
    fn parse_list_items_from_xml(&self, xml: &str) -> Vec<ParsedListItem> {
        let mut items = Vec::new();
        let mut pos = 0;

        while let Some(start) = xml[pos..].find("<w:listItem") {
            let item_start = pos + start;
            if let Some(end) = xml[item_start..].find("/>") {
                let element = &xml[item_start..item_start + end + 2];
                let display_text = self.extract_attribute_from_str(element, "displayText").unwrap_or_default();
                let value = self.extract_attribute_from_str(element, "value").unwrap_or_default();
                items.push(ParsedListItem {
                    display_text: if display_text.is_empty() { value.clone() } else { display_text },
                    value,
                });
                pos = item_start + end + 2;
            } else {
                break;
            }
        }

        items
    }

    /// Helper to extract attribute value from XML string
    fn extract_attribute_from_xml(&self, xml: &str, attr_name: &str) -> Option<String> {
        self.extract_attribute_from_str(xml, attr_name)
    }

    /// Helper to extract attribute from element string
    fn extract_attribute_from_str(&self, element: &str, attr_name: &str) -> Option<String> {
        // Try w: prefix first
        let search = format!("w:{}=\"", attr_name);
        if let Some(start) = element.find(&search) {
            let rest = &element[start + search.len()..];
            if let Some(end) = rest.find('"') {
                return Some(rest[..end].to_string());
            }
        }

        // Try w14: prefix
        let search = format!("w14:{}=\"", attr_name);
        if let Some(start) = element.find(&search) {
            let rest = &element[start + search.len()..];
            if let Some(end) = rest.find('"') {
                return Some(rest[..end].to_string());
            }
        }

        // Try w15: prefix
        let search = format!("w15:{}=\"", attr_name);
        if let Some(start) = element.find(&search) {
            let rest = &element[start + search.len()..];
            if let Some(end) = rest.find('"') {
                return Some(rest[..end].to_string());
            }
        }

        // Try without prefix
        let search = format!("{}=\"", attr_name);
        if let Some(start) = element.find(&search) {
            let rest = &element[start + search.len()..];
            if let Some(end) = rest.find('"') {
                return Some(rest[..end].to_string());
            }
        }

        None
    }

    /// Helper to extract nested element val attribute
    fn extract_nested_val(&self, xml: &str, element_name: &str) -> Option<String> {
        // Try w: prefix
        let search = format!("<w:{}", element_name);
        if let Some(start) = xml.find(&search) {
            let rest = &xml[start..];
            if let Some(end) = rest.find("/>").or_else(|| rest.find('>')) {
                let element = &rest[..end + if rest[end..].starts_with("/>") { 2 } else { 1 }];
                return self.extract_attribute_from_str(element, "val");
            }
        }

        // Try w15: prefix
        let search = format!("<w15:{}", element_name);
        if let Some(start) = xml.find(&search) {
            let rest = &xml[start..];
            if let Some(end) = rest.find("/>").or_else(|| rest.find('>')) {
                let element = &rest[..end + if rest[end..].starts_with("/>") { 2 } else { 1 }];
                return self.extract_attribute_from_str(element, "val");
            }
        }

        // Try without prefix
        let search = format!("<{}", element_name);
        if let Some(start) = xml.find(&search) {
            let rest = &xml[start..];
            if let Some(end) = rest.find("/>").or_else(|| rest.find('>')) {
                let element = &rest[..end + if rest[end..].starts_with("/>") { 2 } else { 1 }];
                return self.extract_attribute_from_str(element, "val");
            }
        }

        None
    }

    /// Parse an empty property element within w:sdtPr
    fn parse_sdt_property_empty(
        &self,
        e: &BytesStart,
        control: &mut ParsedContentControl,
    ) -> DocxResult<()> {
        let name = e.name();

        if XmlParser::matches_element(name.as_ref(), "id") {
            if let Some(val) = XmlParser::get_w_attribute(e, "val") {
                control.sdt_id = val.parse().ok();
            }
        } else if XmlParser::matches_element(name.as_ref(), "tag") {
            if let Some(val) = XmlParser::get_w_attribute(e, "val") {
                control.tag = val;
            }
        } else if XmlParser::matches_element(name.as_ref(), "alias") {
            if let Some(val) = XmlParser::get_w_attribute(e, "val") {
                control.title = val;
            }
        } else if XmlParser::matches_element(name.as_ref(), "lock") {
            if let Some(val) = XmlParser::get_w_attribute(e, "val") {
                match val.as_str() {
                    "sdtLocked" => control.lock.sdt_locked = true,
                    "contentLocked" => control.lock.content_locked = true,
                    "sdtContentLocked" => {
                        control.lock.sdt_locked = true;
                        control.lock.content_locked = true;
                    }
                    _ => {}
                }
            }
        } else if XmlParser::matches_element(name.as_ref(), "dataBinding") {
            control.data_binding = Some(self.parse_data_binding(e)?);
        } else if XmlParser::matches_element(name.as_ref(), "text") {
            control.control_type = ParsedControlType::PlainText;
            let multiline = XmlParser::get_w_attribute(e, "multiLine")
                .map(|v| XmlParser::parse_bool(&v))
                .unwrap_or(false);
            control.type_properties = ParsedTypeProperties::PlainText { multiline };
        } else if XmlParser::matches_element(name.as_ref(), "picture") {
            control.control_type = ParsedControlType::Picture;
            control.type_properties = ParsedTypeProperties::Picture;
        } else if XmlParser::matches_element(name.as_ref(), "repeatingSectionItem") {
            control.control_type = ParsedControlType::RepeatingSectionItem;
        } else if XmlParser::matches_element(name.as_ref(), "citation") {
            control.control_type = ParsedControlType::Citation;
        } else if XmlParser::matches_element(name.as_ref(), "bibliography") {
            control.control_type = ParsedControlType::Bibliography;
        } else if XmlParser::matches_element(name.as_ref(), "equation") {
            control.control_type = ParsedControlType::Equation;
        } else if XmlParser::matches_element(name.as_ref(), "group") {
            control.control_type = ParsedControlType::Group;
        } else if XmlParser::matches_element(name.as_ref(), "temporary") {
            control.temporary = true;
        } else if XmlParser::matches_element(name.as_ref(), "appearance") {
            if let Some(val) = XmlParser::get_w_attribute(e, "val")
                .or_else(|| XmlParser::get_prefixed_attribute(e, "w15", "val"))
            {
                control.appearance = val;
            }
        } else if XmlParser::matches_element(name.as_ref(), "color") {
            if let Some(val) = XmlParser::get_w_attribute(e, "val") {
                control.color = Some(val);
            }
        } else {
            // Preserve unknown empty elements
            let name_str = std::str::from_utf8(name.as_ref()).unwrap_or("");
            let mut unknown_xml = format!("<{}", name_str);
            for attr in e.attributes().filter_map(|a| a.ok()) {
                let key = std::str::from_utf8(attr.key.as_ref()).unwrap_or("");
                let val = String::from_utf8_lossy(&attr.value);
                unknown_xml.push_str(&format!(" {}=\"{}\"", key, val));
            }
            unknown_xml.push_str("/>");
            control.unknown_properties.push(unknown_xml);
        }

        Ok(())
    }

    /// Parse data binding element
    fn parse_data_binding(&self, e: &BytesStart) -> DocxResult<ParsedDataBinding> {
        let xpath = XmlParser::get_w_attribute(e, "xpath")
            .or_else(|| XmlParser::get_prefixed_attribute(e, "w", "xpath"))
            .unwrap_or_default();

        let prefix_mappings = XmlParser::get_w_attribute(e, "prefixMappings")
            .or_else(|| XmlParser::get_prefixed_attribute(e, "w", "prefixMappings"))
            .unwrap_or_default();

        let store_item_id = XmlParser::get_w_attribute(e, "storeItemID")
            .or_else(|| XmlParser::get_prefixed_attribute(e, "w", "storeItemID"))
            .unwrap_or_default();

        Ok(ParsedDataBinding {
            xpath,
            prefix_mappings,
            store_item_id,
        })
    }

    /// Parse all SDT elements from document XML
    pub fn parse_all(&self, document_xml: &str) -> DocxResult<Vec<ParsedContentControl>> {
        let mut controls = Vec::new();
        let mut reader = XmlParser::from_string(document_xml);
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    if XmlParser::matches_element(e.name().as_ref(), "sdt") {
                        // Capture the entire sdt element using a separate buffer
                        let mut inner_buf = Vec::new();
                        let mut sdt_xml = String::new();
                        self.capture_sdt_element_from_start(&mut reader, e, &mut sdt_xml, &mut inner_buf)?;

                        // Parse the captured SDT
                        let control = self.parse_sdt(&sdt_xml)?;
                        controls.push(control);
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(DocxError::XmlParse(format!("Error parsing document: {}", e))),
                _ => {}
            }
            buf.clear();
        }

        Ok(controls)
    }

    /// Capture an entire SDT element as XML string from a start element
    fn capture_sdt_element_from_start(
        &self,
        reader: &mut Reader<&[u8]>,
        start: &BytesStart,
        xml: &mut String,
        buf: &mut Vec<u8>,
    ) -> DocxResult<()> {
        let name_binding = start.name();
        let name_str = std::str::from_utf8(name_binding.as_ref()).unwrap_or("w:sdt");
        xml.push_str(&format!("<{}", name_str));
        for attr in start.attributes().filter_map(|a| a.ok()) {
            let key = std::str::from_utf8(attr.key.as_ref()).unwrap_or("");
            let val = String::from_utf8_lossy(&attr.value);
            xml.push_str(&format!(" {}=\"{}\"", key, val));
        }
        xml.push('>');

        let mut depth = 1;

        loop {
            match reader.read_event_into(buf) {
                Ok(Event::Start(ref e)) => {
                    depth += 1;
                    let name_binding = e.name();
                    let name = std::str::from_utf8(name_binding.as_ref()).unwrap_or("");
                    xml.push_str(&format!("<{}", name));
                    for attr in e.attributes().filter_map(|a| a.ok()) {
                        let key = std::str::from_utf8(attr.key.as_ref()).unwrap_or("");
                        let val = String::from_utf8_lossy(&attr.value);
                        xml.push_str(&format!(" {}=\"{}\"", key, val));
                    }
                    xml.push('>');
                }
                Ok(Event::Empty(ref e)) => {
                    let name_binding = e.name();
                    let name = std::str::from_utf8(name_binding.as_ref()).unwrap_or("");
                    xml.push_str(&format!("<{}", name));
                    for attr in e.attributes().filter_map(|a| a.ok()) {
                        let key = std::str::from_utf8(attr.key.as_ref()).unwrap_or("");
                        let val = String::from_utf8_lossy(&attr.value);
                        xml.push_str(&format!(" {}=\"{}\"", key, val));
                    }
                    xml.push_str("/>");
                }
                Ok(Event::End(ref e)) => {
                    depth -= 1;
                    let name_binding = e.name();
                    let name = std::str::from_utf8(name_binding.as_ref()).unwrap_or("");
                    xml.push_str(&format!("</{}>", name));
                    if depth == 0 {
                        break;
                    }
                }
                Ok(Event::Text(ref e)) => {
                    let text = e.unescape().unwrap_or_default();
                    xml.push_str(&escape_xml(&text));
                }
                Ok(Event::CData(ref e)) => {
                    let text = String::from_utf8_lossy(e.as_ref());
                    xml.push_str(&format!("<![CDATA[{}]]>", text));
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    return Err(DocxError::XmlParse(format!(
                        "Error capturing SDT element: {}",
                        e
                    )))
                }
                _ => {}
            }
            buf.clear();
        }

        Ok(())
    }
}

impl Default for ContentControlParser {
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

    #[test]
    fn test_parser_creation() {
        let parser = ContentControlParser::new();
        // Parser should be created without error
    }

    #[test]
    fn test_parse_plain_text_sdt() {
        let xml = r#"<w:sdt>
            <w:sdtPr>
                <w:id w:val="12345"/>
                <w:tag w:val="test_tag"/>
                <w:alias w:val="Test Title"/>
                <w:text/>
            </w:sdtPr>
            <w:sdtContent>
                <w:p><w:r><w:t>Content</w:t></w:r></w:p>
            </w:sdtContent>
        </w:sdt>"#;

        let parser = ContentControlParser::new();
        let control = parser.parse_sdt(xml).unwrap();

        assert_eq!(control.sdt_id, Some(12345));
        assert_eq!(control.tag, "test_tag");
        assert_eq!(control.title, "Test Title");
        assert_eq!(control.control_type, ParsedControlType::PlainText);
    }

    #[test]
    fn test_parse_checkbox_sdt() {
        let xml = r#"<w:sdt>
            <w:sdtPr>
                <w:id w:val="999"/>
                <w14:checkbox>
                    <w14:checked w14:val="1"/>
                    <w14:checkedState w14:val="2612" w14:font="MS Gothic"/>
                    <w14:uncheckedState w14:val="2610" w14:font="MS Gothic"/>
                </w14:checkbox>
            </w:sdtPr>
            <w:sdtContent/>
        </w:sdt>"#;

        let parser = ContentControlParser::new();
        let control = parser.parse_sdt(xml).unwrap();

        assert_eq!(control.control_type, ParsedControlType::Checkbox);
        if let ParsedTypeProperties::Checkbox {
            checked,
            checked_state,
            unchecked_state,
        } = control.type_properties
        {
            assert!(checked);
            assert!(checked_state.is_some());
            assert!(unchecked_state.is_some());
        } else {
            panic!("Expected checkbox properties");
        }
    }

    #[test]
    fn test_parse_dropdown_sdt() {
        let xml = r#"<w:sdt>
            <w:sdtPr>
                <w:dropDownList>
                    <w:listItem w:displayText="Option 1" w:value="opt1"/>
                    <w:listItem w:displayText="Option 2" w:value="opt2"/>
                    <w:listItem w:displayText="Option 3" w:value="opt3"/>
                </w:dropDownList>
            </w:sdtPr>
            <w:sdtContent/>
        </w:sdt>"#;

        let parser = ContentControlParser::new();
        let control = parser.parse_sdt(xml).unwrap();

        assert_eq!(control.control_type, ParsedControlType::DropdownList);
        if let ParsedTypeProperties::DropdownList { items, .. } = control.type_properties {
            assert_eq!(items.len(), 3);
            assert_eq!(items[0].display_text, "Option 1");
            assert_eq!(items[0].value, "opt1");
        } else {
            panic!("Expected dropdown properties");
        }
    }

    #[test]
    fn test_parse_date_picker_sdt() {
        let xml = r#"<w:sdt>
            <w:sdtPr>
                <w:date w:fullDate="2025-01-15T00:00:00Z">
                    <w:dateFormat w:val="M/d/yyyy"/>
                    <w:lid w:val="en-US"/>
                    <w:calendar w:val="gregorian"/>
                </w:date>
            </w:sdtPr>
            <w:sdtContent/>
        </w:sdt>"#;

        let parser = ContentControlParser::new();
        let control = parser.parse_sdt(xml).unwrap();

        assert_eq!(control.control_type, ParsedControlType::DatePicker);
        if let ParsedTypeProperties::DatePicker {
            date_format,
            calendar,
            full_date,
            ..
        } = control.type_properties
        {
            assert_eq!(date_format, "M/d/yyyy");
            assert_eq!(calendar, "gregorian");
            assert!(full_date.is_some());
        } else {
            panic!("Expected date picker properties");
        }
    }

    #[test]
    fn test_parse_lock_settings() {
        let xml = r#"<w:sdt>
            <w:sdtPr>
                <w:lock w:val="sdtContentLocked"/>
                <w:text/>
            </w:sdtPr>
            <w:sdtContent/>
        </w:sdt>"#;

        let parser = ContentControlParser::new();
        let control = parser.parse_sdt(xml).unwrap();

        assert!(control.lock.sdt_locked);
        assert!(control.lock.content_locked);
    }

    #[test]
    fn test_parse_data_binding() {
        let xml = r#"<w:sdt>
            <w:sdtPr>
                <w:dataBinding w:xpath="/root/element" w:storeItemID="{12345}" w:prefixMappings="xmlns:ns='http://example.com'"/>
                <w:text/>
            </w:sdtPr>
            <w:sdtContent/>
        </w:sdt>"#;

        let parser = ContentControlParser::new();
        let control = parser.parse_sdt(xml).unwrap();

        assert!(control.data_binding.is_some());
        let binding = control.data_binding.unwrap();
        assert_eq!(binding.xpath, "/root/element");
        assert_eq!(binding.store_item_id, "{12345}");
    }

    #[test]
    fn test_parse_picture_sdt() {
        let xml = r#"<w:sdt>
            <w:sdtPr>
                <w:picture/>
            </w:sdtPr>
            <w:sdtContent/>
        </w:sdt>"#;

        let parser = ContentControlParser::new();
        let control = parser.parse_sdt(xml).unwrap();

        assert_eq!(control.control_type, ParsedControlType::Picture);
    }

    #[test]
    fn test_parse_combo_box_sdt() {
        let xml = r#"<w:sdt>
            <w:sdtPr>
                <w:comboBox>
                    <w:listItem w:displayText="Item 1" w:value="val1"/>
                    <w:listItem w:displayText="Item 2" w:value="val2"/>
                </w:comboBox>
            </w:sdtPr>
            <w:sdtContent/>
        </w:sdt>"#;

        let parser = ContentControlParser::new();
        let control = parser.parse_sdt(xml).unwrap();

        assert_eq!(control.control_type, ParsedControlType::ComboBox);
        if let ParsedTypeProperties::ComboBox { items, .. } = control.type_properties {
            assert_eq!(items.len(), 2);
        } else {
            panic!("Expected combo box properties");
        }
    }

    #[test]
    fn test_parse_rich_text_default() {
        let xml = r#"<w:sdt>
            <w:sdtPr>
                <w:id w:val="1"/>
            </w:sdtPr>
            <w:sdtContent/>
        </w:sdt>"#;

        let parser = ContentControlParser::new();
        let control = parser.parse_sdt(xml).unwrap();

        // Without type-specific elements, should default to RichText
        assert_eq!(control.control_type, ParsedControlType::RichText);
    }

    #[test]
    fn test_parse_temporary_flag() {
        let xml = r#"<w:sdt>
            <w:sdtPr>
                <w:temporary/>
                <w:text/>
            </w:sdtPr>
            <w:sdtContent/>
        </w:sdt>"#;

        let parser = ContentControlParser::new();
        let control = parser.parse_sdt(xml).unwrap();

        assert!(control.temporary);
    }

    #[test]
    fn test_parse_appearance() {
        let xml = r#"<w:sdt>
            <w:sdtPr>
                <w15:appearance w15:val="hidden"/>
                <w:text/>
            </w:sdtPr>
            <w:sdtContent/>
        </w:sdt>"#;

        let parser = ContentControlParser::new();
        let control = parser.parse_sdt(xml).unwrap();

        assert_eq!(control.appearance, "hidden");
    }

    #[test]
    fn test_parse_unknown_properties_preserved() {
        let xml = r#"<w:sdt>
            <w:sdtPr>
                <w:text/>
                <w:customElement w:val="custom"/>
            </w:sdtPr>
            <w:sdtContent/>
        </w:sdt>"#;

        let parser = ContentControlParser::new();
        let control = parser.parse_sdt(xml).unwrap();

        // Unknown elements should be preserved
        assert!(!control.unknown_properties.is_empty());
    }

    #[test]
    fn test_parse_content_xml_preserved() {
        let xml = r#"<w:sdt>
            <w:sdtPr>
                <w:text/>
            </w:sdtPr>
            <w:sdtContent>
                <w:p><w:r><w:t>Hello World</w:t></w:r></w:p>
            </w:sdtContent>
        </w:sdt>"#;

        let parser = ContentControlParser::new();
        let control = parser.parse_sdt(xml).unwrap();

        assert!(!control.content_xml.is_empty());
        assert!(control.content_xml.contains("Hello World"));
    }

    #[test]
    fn test_parsed_control_type_values() {
        assert_eq!(ParsedControlType::RichText, ParsedControlType::RichText);
        assert_ne!(ParsedControlType::RichText, ParsedControlType::PlainText);
    }

    #[test]
    fn test_lock_settings_default() {
        let lock = LockSettings::default();
        assert!(!lock.sdt_locked);
        assert!(!lock.content_locked);
    }

    #[test]
    fn test_parsed_content_control_default() {
        let control = ParsedContentControl::default();
        assert!(control.sdt_id.is_none());
        assert!(control.tag.is_empty());
        assert!(control.title.is_empty());
        assert_eq!(control.control_type, ParsedControlType::RichText);
    }

    #[test]
    fn test_escape_xml() {
        assert_eq!(escape_xml("<test>"), "&lt;test&gt;");
        assert_eq!(escape_xml("a & b"), "a &amp; b");
        assert_eq!(escape_xml("\"quoted\""), "&quot;quoted&quot;");
    }

    #[test]
    fn test_parse_multiline_text() {
        let xml = r#"<w:sdt>
            <w:sdtPr>
                <w:text w:multiLine="true"/>
            </w:sdtPr>
            <w:sdtContent/>
        </w:sdt>"#;

        let parser = ContentControlParser::new();
        let control = parser.parse_sdt(xml).unwrap();

        if let ParsedTypeProperties::PlainText { multiline } = control.type_properties {
            assert!(multiline);
        } else {
            panic!("Expected plain text properties");
        }
    }

    #[test]
    fn test_parse_group_sdt() {
        let xml = r#"<w:sdt>
            <w:sdtPr>
                <w:group/>
            </w:sdtPr>
            <w:sdtContent/>
        </w:sdt>"#;

        let parser = ContentControlParser::new();
        let control = parser.parse_sdt(xml).unwrap();

        assert_eq!(control.control_type, ParsedControlType::Group);
    }

    #[test]
    fn test_parse_citation_sdt() {
        let xml = r#"<w:sdt>
            <w:sdtPr>
                <w:citation/>
            </w:sdtPr>
            <w:sdtContent/>
        </w:sdt>"#;

        let parser = ContentControlParser::new();
        let control = parser.parse_sdt(xml).unwrap();

        assert_eq!(control.control_type, ParsedControlType::Citation);
    }

    #[test]
    fn test_parse_bibliography_sdt() {
        let xml = r#"<w:sdt>
            <w:sdtPr>
                <w:bibliography/>
            </w:sdtPr>
            <w:sdtContent/>
        </w:sdt>"#;

        let parser = ContentControlParser::new();
        let control = parser.parse_sdt(xml).unwrap();

        assert_eq!(control.control_type, ParsedControlType::Bibliography);
    }

    #[test]
    fn test_parse_equation_sdt() {
        let xml = r#"<w:sdt>
            <w:sdtPr>
                <w:equation/>
            </w:sdtPr>
            <w:sdtContent/>
        </w:sdt>"#;

        let parser = ContentControlParser::new();
        let control = parser.parse_sdt(xml).unwrap();

        assert_eq!(control.control_type, ParsedControlType::Equation);
    }
}
