/**
 * StyleDialog - Form to create or modify styles
 *
 * Provides a comprehensive dialog for creating new styles or editing existing ones.
 * Includes base style selection, property configuration, and live preview.
 */

import { useState, useCallback, useMemo, useEffect } from 'react';
import {
  Style,
  StyleType,
  ParagraphProperties,
  CharacterProperties,
  TextAlignment,
  LineSpacing,
  FONT_FAMILIES,
} from '../lib/types';
import './StyleDialog.css';

interface StyleDialogProps {
  /** Whether the dialog is open */
  isOpen: boolean;
  /** Callback to close the dialog */
  onClose: () => void;
  /** Callback when style is saved */
  onSave: (style: Partial<Style>) => void;
  /** Existing style to edit (null for new style) */
  existingStyle?: Style | null;
  /** Available styles to use as base */
  availableStyles: Style[];
  /** Mode: 'create' or 'edit' */
  mode?: 'create' | 'edit';
}

interface TabProps {
  label: string;
  isActive: boolean;
  onClick: () => void;
}

function Tab({ label, isActive, onClick }: TabProps) {
  return (
    <button
      className={`dialog-tab ${isActive ? 'active' : ''}`}
      onClick={onClick}
    >
      {label}
    </button>
  );
}

const ALIGNMENT_OPTIONS: { value: TextAlignment; label: string }[] = [
  { value: 'left', label: 'Left' },
  { value: 'center', label: 'Center' },
  { value: 'right', label: 'Right' },
  { value: 'justify', label: 'Justify' },
];

const FONT_SIZE_OPTIONS = [8, 9, 10, 11, 12, 14, 16, 18, 20, 22, 24, 26, 28, 36, 48, 72];

export function StyleDialog({
  isOpen,
  onClose,
  onSave,
  existingStyle,
  availableStyles,
  mode = 'create',
}: StyleDialogProps) {
  const [activeTab, setActiveTab] = useState<'general' | 'paragraph' | 'character'>('general');

  // Style properties state
  const [styleName, setStyleName] = useState('');
  const [styleType, setStyleType] = useState<StyleType>('paragraph');
  const [basedOn, setBasedOn] = useState<string | undefined>();
  const [nextStyle, setNextStyle] = useState<string | undefined>();

  // Paragraph properties
  const [alignment, setAlignment] = useState<TextAlignment | undefined>();
  const [indentLeft, setIndentLeft] = useState<number | undefined>();
  const [indentRight, setIndentRight] = useState<number | undefined>();
  const [indentFirstLine, setIndentFirstLine] = useState<number | undefined>();
  const [spaceBefore, setSpaceBefore] = useState<number | undefined>();
  const [spaceAfter, setSpaceAfter] = useState<number | undefined>();
  const [lineSpacingType, setLineSpacingType] = useState<'Multiple' | 'Exact' | 'AtLeast'>('Multiple');
  const [lineSpacingValue, setLineSpacingValue] = useState<number>(1.0);

  // Character properties
  const [fontFamily, setFontFamily] = useState<string | undefined>();
  const [fontSize, setFontSize] = useState<number | undefined>();
  const [bold, setBold] = useState<boolean | undefined>();
  const [italic, setItalic] = useState<boolean | undefined>();
  const [underline, setUnderline] = useState<boolean | undefined>();
  const [color, setColor] = useState<string | undefined>();

  // Reset form when dialog opens or existing style changes
  useEffect(() => {
    if (isOpen && existingStyle) {
      setStyleName(existingStyle.name);
      setStyleType(existingStyle.styleType);
      setBasedOn(existingStyle.basedOn);
      setNextStyle(existingStyle.nextStyle);

      // Paragraph properties
      setAlignment(existingStyle.paragraphProps.alignment);
      setIndentLeft(existingStyle.paragraphProps.indentLeft);
      setIndentRight(existingStyle.paragraphProps.indentRight);
      setIndentFirstLine(existingStyle.paragraphProps.indentFirstLine);
      setSpaceBefore(existingStyle.paragraphProps.spaceBefore);
      setSpaceAfter(existingStyle.paragraphProps.spaceAfter);
      if (existingStyle.paragraphProps.lineSpacing) {
        setLineSpacingType(existingStyle.paragraphProps.lineSpacing.type);
        setLineSpacingValue(existingStyle.paragraphProps.lineSpacing.value);
      }

      // Character properties
      setFontFamily(existingStyle.characterProps.fontFamily);
      setFontSize(existingStyle.characterProps.fontSize);
      setBold(existingStyle.characterProps.bold);
      setItalic(existingStyle.characterProps.italic);
      setUnderline(existingStyle.characterProps.underline);
      setColor(existingStyle.characterProps.color);
    } else if (isOpen) {
      // Reset to defaults for new style
      setStyleName('');
      setStyleType('paragraph');
      setBasedOn('Normal');
      setNextStyle(undefined);
      setAlignment(undefined);
      setIndentLeft(undefined);
      setIndentRight(undefined);
      setIndentFirstLine(undefined);
      setSpaceBefore(undefined);
      setSpaceAfter(undefined);
      setLineSpacingType('Multiple');
      setLineSpacingValue(1.0);
      setFontFamily(undefined);
      setFontSize(undefined);
      setBold(undefined);
      setItalic(undefined);
      setUnderline(undefined);
      setColor(undefined);
    }
  }, [isOpen, existingStyle]);

  // Filter available base styles by type
  const baseStyleOptions = useMemo(() => {
    return availableStyles.filter(
      (s) => s.styleType === styleType && s.id !== existingStyle?.id
    );
  }, [availableStyles, styleType, existingStyle]);

  // Build line spacing object
  const lineSpacing: LineSpacing | undefined = useMemo(() => {
    if (lineSpacingValue === undefined) return undefined;
    return { type: lineSpacingType, value: lineSpacingValue };
  }, [lineSpacingType, lineSpacingValue]);

  // Build paragraph properties
  const paragraphProps: ParagraphProperties = useMemo(() => ({
    alignment,
    indentLeft,
    indentRight,
    indentFirstLine,
    spaceBefore,
    spaceAfter,
    lineSpacing,
  }), [alignment, indentLeft, indentRight, indentFirstLine, spaceBefore, spaceAfter, lineSpacing]);

  // Build character properties
  const characterProps: CharacterProperties = useMemo(() => ({
    fontFamily,
    fontSize,
    bold,
    italic,
    underline,
    color,
  }), [fontFamily, fontSize, bold, italic, underline, color]);

  // Preview style
  const previewStyle = useMemo(() => ({
    fontFamily: fontFamily || 'Calibri',
    fontSize: fontSize || 11,
    fontWeight: bold ? 'bold' : 'normal',
    fontStyle: italic ? 'italic' : 'normal',
    textDecoration: underline ? 'underline' : 'none',
    color: color || '#000000',
    textAlign: (alignment || 'left') as React.CSSProperties['textAlign'],
  }), [fontFamily, fontSize, bold, italic, underline, color, alignment]);

  const handleSave = useCallback(() => {
    if (!styleName.trim()) {
      alert('Please enter a style name');
      return;
    }

    const styleData: Partial<Style> = {
      id: existingStyle?.id || styleName.replace(/\s+/g, ''),
      name: styleName,
      styleType,
      basedOn,
      nextStyle,
      paragraphProps,
      characterProps,
    };

    onSave(styleData);
    onClose();
  }, [styleName, styleType, basedOn, nextStyle, paragraphProps, characterProps, existingStyle, onSave, onClose]);

  if (!isOpen) return null;

  return (
    <div className="dialog-overlay" onClick={onClose}>
      <div className="style-dialog" onClick={(e) => e.stopPropagation()}>
        <div className="dialog-header">
          <h2>{mode === 'edit' ? 'Modify Style' : 'Create New Style'}</h2>
          <button className="close-btn" onClick={onClose}>
            x
          </button>
        </div>

        <div className="dialog-tabs">
          <Tab
            label="General"
            isActive={activeTab === 'general'}
            onClick={() => setActiveTab('general')}
          />
          <Tab
            label="Paragraph"
            isActive={activeTab === 'paragraph'}
            onClick={() => setActiveTab('paragraph')}
          />
          <Tab
            label="Character"
            isActive={activeTab === 'character'}
            onClick={() => setActiveTab('character')}
          />
        </div>

        <div className="dialog-content">
          {activeTab === 'general' && (
            <div className="tab-content">
              <div className="form-group">
                <label htmlFor="style-name">Style Name</label>
                <input
                  id="style-name"
                  type="text"
                  value={styleName}
                  onChange={(e) => setStyleName(e.target.value)}
                  placeholder="Enter style name"
                  disabled={existingStyle?.builtIn}
                />
              </div>

              <div className="form-group">
                <label htmlFor="style-type">Style Type</label>
                <select
                  id="style-type"
                  value={styleType}
                  onChange={(e) => setStyleType(e.target.value as StyleType)}
                  disabled={mode === 'edit'}
                >
                  <option value="paragraph">Paragraph</option>
                  <option value="character">Character</option>
                </select>
              </div>

              <div className="form-group">
                <label htmlFor="based-on">Based On</label>
                <select
                  id="based-on"
                  value={basedOn || ''}
                  onChange={(e) => setBasedOn(e.target.value || undefined)}
                >
                  <option value="">None</option>
                  {baseStyleOptions.map((s) => (
                    <option key={s.id} value={s.id}>
                      {s.name}
                    </option>
                  ))}
                </select>
              </div>

              {styleType === 'paragraph' && (
                <div className="form-group">
                  <label htmlFor="next-style">Style for Following Paragraph</label>
                  <select
                    id="next-style"
                    value={nextStyle || ''}
                    onChange={(e) => setNextStyle(e.target.value || undefined)}
                  >
                    <option value="">Same as current</option>
                    {baseStyleOptions.map((s) => (
                      <option key={s.id} value={s.id}>
                        {s.name}
                      </option>
                    ))}
                  </select>
                </div>
              )}
            </div>
          )}

          {activeTab === 'paragraph' && (
            <div className="tab-content">
              <div className="form-group">
                <label htmlFor="alignment">Alignment</label>
                <select
                  id="alignment"
                  value={alignment || ''}
                  onChange={(e) => setAlignment(e.target.value as TextAlignment || undefined)}
                >
                  <option value="">Inherit</option>
                  {ALIGNMENT_OPTIONS.map((opt) => (
                    <option key={opt.value} value={opt.value}>
                      {opt.label}
                    </option>
                  ))}
                </select>
              </div>

              <div className="form-row">
                <div className="form-group half">
                  <label htmlFor="indent-left">Left Indent (pt)</label>
                  <input
                    id="indent-left"
                    type="number"
                    value={indentLeft ?? ''}
                    onChange={(e) => setIndentLeft(e.target.value ? Number(e.target.value) : undefined)}
                    min={0}
                    step={6}
                  />
                </div>
                <div className="form-group half">
                  <label htmlFor="indent-right">Right Indent (pt)</label>
                  <input
                    id="indent-right"
                    type="number"
                    value={indentRight ?? ''}
                    onChange={(e) => setIndentRight(e.target.value ? Number(e.target.value) : undefined)}
                    min={0}
                    step={6}
                  />
                </div>
              </div>

              <div className="form-group">
                <label htmlFor="indent-first">First Line Indent (pt)</label>
                <input
                  id="indent-first"
                  type="number"
                  value={indentFirstLine ?? ''}
                  onChange={(e) => setIndentFirstLine(e.target.value ? Number(e.target.value) : undefined)}
                  step={6}
                />
              </div>

              <div className="form-row">
                <div className="form-group half">
                  <label htmlFor="space-before">Space Before (pt)</label>
                  <input
                    id="space-before"
                    type="number"
                    value={spaceBefore ?? ''}
                    onChange={(e) => setSpaceBefore(e.target.value ? Number(e.target.value) : undefined)}
                    min={0}
                    step={2}
                  />
                </div>
                <div className="form-group half">
                  <label htmlFor="space-after">Space After (pt)</label>
                  <input
                    id="space-after"
                    type="number"
                    value={spaceAfter ?? ''}
                    onChange={(e) => setSpaceAfter(e.target.value ? Number(e.target.value) : undefined)}
                    min={0}
                    step={2}
                  />
                </div>
              </div>

              <div className="form-row">
                <div className="form-group half">
                  <label htmlFor="line-spacing-type">Line Spacing</label>
                  <select
                    id="line-spacing-type"
                    value={lineSpacingType}
                    onChange={(e) => setLineSpacingType(e.target.value as 'Multiple' | 'Exact' | 'AtLeast')}
                  >
                    <option value="Multiple">Multiple</option>
                    <option value="Exact">Exactly</option>
                    <option value="AtLeast">At Least</option>
                  </select>
                </div>
                <div className="form-group half">
                  <label htmlFor="line-spacing-value">Value</label>
                  <input
                    id="line-spacing-value"
                    type="number"
                    value={lineSpacingValue}
                    onChange={(e) => setLineSpacingValue(Number(e.target.value))}
                    min={lineSpacingType === 'Multiple' ? 0.5 : 1}
                    step={lineSpacingType === 'Multiple' ? 0.25 : 1}
                  />
                </div>
              </div>
            </div>
          )}

          {activeTab === 'character' && (
            <div className="tab-content">
              <div className="form-group">
                <label htmlFor="font-family">Font Family</label>
                <select
                  id="font-family"
                  value={fontFamily || ''}
                  onChange={(e) => setFontFamily(e.target.value || undefined)}
                >
                  <option value="">Inherit</option>
                  {FONT_FAMILIES.map((font) => (
                    <option key={font} value={font}>
                      {font}
                    </option>
                  ))}
                </select>
              </div>

              <div className="form-group">
                <label htmlFor="font-size">Font Size (pt)</label>
                <select
                  id="font-size"
                  value={fontSize ?? ''}
                  onChange={(e) => setFontSize(e.target.value ? Number(e.target.value) : undefined)}
                >
                  <option value="">Inherit</option>
                  {FONT_SIZE_OPTIONS.map((size) => (
                    <option key={size} value={size}>
                      {size}
                    </option>
                  ))}
                </select>
              </div>

              <div className="form-group checkbox-group">
                <label>
                  <input
                    type="checkbox"
                    checked={bold === true}
                    onChange={(e) => setBold(e.target.checked ? true : undefined)}
                  />
                  Bold
                </label>
                <label>
                  <input
                    type="checkbox"
                    checked={italic === true}
                    onChange={(e) => setItalic(e.target.checked ? true : undefined)}
                  />
                  Italic
                </label>
                <label>
                  <input
                    type="checkbox"
                    checked={underline === true}
                    onChange={(e) => setUnderline(e.target.checked ? true : undefined)}
                  />
                  Underline
                </label>
              </div>

              <div className="form-group">
                <label htmlFor="color">Text Color</label>
                <div className="color-input-group">
                  <input
                    id="color"
                    type="color"
                    value={color || '#000000'}
                    onChange={(e) => setColor(e.target.value)}
                  />
                  <input
                    type="text"
                    value={color || ''}
                    onChange={(e) => setColor(e.target.value || undefined)}
                    placeholder="e.g., #000000"
                  />
                </div>
              </div>
            </div>
          )}
        </div>

        {/* Preview Section */}
        <div className="dialog-preview">
          <h4>Preview</h4>
          <div className="preview-container">
            <p style={previewStyle}>
              {styleName || 'Sample Text'} - The quick brown fox jumps over the lazy dog.
            </p>
          </div>
        </div>

        <div className="dialog-footer">
          <button className="cancel-btn" onClick={onClose}>
            Cancel
          </button>
          <button className="save-btn" onClick={handleSave}>
            {mode === 'edit' ? 'Save Changes' : 'Create Style'}
          </button>
        </div>
      </div>
    </div>
  );
}

export default StyleDialog;
