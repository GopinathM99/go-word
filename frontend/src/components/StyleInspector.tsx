/**
 * StyleInspector - Shows computed style for current selection
 *
 * Displays which properties come from style vs direct formatting,
 * and allows clicking to edit style or clear direct formatting.
 */

import { useCallback } from 'react';
import {
  StyleInspectorData,
  ComputedProperty,
  PropertySource,
  getPropertySourceName,
  hasDirectFormatting,
  formatLineSpacing,
  LineSpacing,
  TextAlignment,
} from '../lib/types';
import './StyleInspector.css';

interface StyleInspectorProps {
  /** The computed style data for the current selection */
  data: StyleInspectorData;
  /** Callback when user clicks to edit a style */
  onEditStyle?: (styleId: string) => void;
  /** Callback when user clicks to clear direct formatting */
  onClearDirectFormatting?: (clearParagraph: boolean, clearCharacter: boolean) => void;
  /** Callback when user clicks a property to modify it */
  onEditProperty?: (property: string, section: 'paragraph' | 'character') => void;
}

interface PropertyRowProps<T> {
  label: string;
  property: ComputedProperty<T>;
  formatValue?: (value: T) => string;
  onEditStyle?: (styleId: string) => void;
  onClearDirect?: () => void;
  onEdit?: () => void;
}

function PropertyRow<T>({
  label,
  property,
  formatValue,
  onEditStyle,
  onClearDirect,
  onEdit,
}: PropertyRowProps<T>) {
  const displayValue = formatValue
    ? formatValue(property.value)
    : String(property.value);

  const sourceName = getPropertySourceName(property.source);
  const isDirect = hasDirectFormatting(property.source);

  const handleSourceClick = useCallback(() => {
    if (property.source.type === 'Style' && onEditStyle) {
      onEditStyle(property.source.styleId);
    } else if (isDirect && onClearDirect) {
      onClearDirect();
    }
  }, [property.source, onEditStyle, onClearDirect, isDirect]);

  return (
    <div className="property-row">
      <span className="property-label">{label}</span>
      <span
        className="property-value"
        onClick={onEdit}
        title="Click to edit"
      >
        {displayValue}
      </span>
      <span
        className={`property-source ${isDirect ? 'direct' : ''}`}
        onClick={handleSourceClick}
        title={isDirect ? 'Click to clear direct formatting' : 'Click to edit style'}
      >
        {sourceName}
      </span>
    </div>
  );
}

function formatAlignment(alignment: TextAlignment): string {
  return alignment.charAt(0).toUpperCase() + alignment.slice(1);
}

function formatBoolean(value: boolean): string {
  return value ? 'Yes' : 'No';
}

function formatPoints(value: number): string {
  return `${value}pt`;
}

export function StyleInspector({
  data,
  onEditStyle,
  onClearDirectFormatting,
  onEditProperty,
}: StyleInspectorProps) {
  const handleClearParagraphFormatting = useCallback(() => {
    onClearDirectFormatting?.(true, false);
  }, [onClearDirectFormatting]);

  const handleClearCharacterFormatting = useCallback(() => {
    onClearDirectFormatting?.(false, true);
  }, [onClearDirectFormatting]);

  const handleClearAllFormatting = useCallback(() => {
    onClearDirectFormatting?.(true, true);
  }, [onClearDirectFormatting]);

  return (
    <div className="style-inspector">
      <div className="inspector-header">
        <h3>Style Inspector</h3>
        {(data.hasDirectParagraphFormatting || data.hasDirectCharacterFormatting) && (
          <button
            className="clear-all-btn"
            onClick={handleClearAllFormatting}
            title="Clear all direct formatting"
          >
            Clear All
          </button>
        )}
      </div>

      {/* Applied Styles Section */}
      <section className="inspector-section">
        <h4>Applied Styles</h4>
        <div className="applied-styles">
          {data.paragraphStyleId && (
            <div
              className="applied-style paragraph-style"
              onClick={() => onEditStyle?.(data.paragraphStyleId!)}
              title="Click to edit paragraph style"
            >
              <span className="style-type">Paragraph:</span>
              <span className="style-name">{data.paragraphStyleId}</span>
            </div>
          )}
          {data.characterStyleId && (
            <div
              className="applied-style character-style"
              onClick={() => onEditStyle?.(data.characterStyleId!)}
              title="Click to edit character style"
            >
              <span className="style-type">Character:</span>
              <span className="style-name">{data.characterStyleId}</span>
            </div>
          )}
        </div>
      </section>

      {/* Paragraph Properties Section */}
      <section className="inspector-section">
        <div className="section-header">
          <h4>Paragraph Properties</h4>
          {data.hasDirectParagraphFormatting && (
            <button
              className="clear-section-btn"
              onClick={handleClearParagraphFormatting}
              title="Clear paragraph direct formatting"
            >
              Clear
            </button>
          )}
        </div>
        <div className="properties-list">
          <PropertyRow
            label="Alignment"
            property={data.paragraphProps.alignment}
            formatValue={formatAlignment}
            onEditStyle={onEditStyle}
            onClearDirect={handleClearParagraphFormatting}
            onEdit={() => onEditProperty?.('alignment', 'paragraph')}
          />
          <PropertyRow
            label="Left Indent"
            property={data.paragraphProps.indentLeft}
            formatValue={formatPoints}
            onEditStyle={onEditStyle}
            onClearDirect={handleClearParagraphFormatting}
            onEdit={() => onEditProperty?.('indentLeft', 'paragraph')}
          />
          <PropertyRow
            label="Right Indent"
            property={data.paragraphProps.indentRight}
            formatValue={formatPoints}
            onEditStyle={onEditStyle}
            onClearDirect={handleClearParagraphFormatting}
            onEdit={() => onEditProperty?.('indentRight', 'paragraph')}
          />
          <PropertyRow
            label="First Line Indent"
            property={data.paragraphProps.indentFirstLine}
            formatValue={formatPoints}
            onEditStyle={onEditStyle}
            onClearDirect={handleClearParagraphFormatting}
            onEdit={() => onEditProperty?.('indentFirstLine', 'paragraph')}
          />
          <PropertyRow
            label="Space Before"
            property={data.paragraphProps.spaceBefore}
            formatValue={formatPoints}
            onEditStyle={onEditStyle}
            onClearDirect={handleClearParagraphFormatting}
            onEdit={() => onEditProperty?.('spaceBefore', 'paragraph')}
          />
          <PropertyRow
            label="Space After"
            property={data.paragraphProps.spaceAfter}
            formatValue={formatPoints}
            onEditStyle={onEditStyle}
            onClearDirect={handleClearParagraphFormatting}
            onEdit={() => onEditProperty?.('spaceAfter', 'paragraph')}
          />
          <PropertyRow
            label="Line Spacing"
            property={data.paragraphProps.lineSpacing}
            formatValue={formatLineSpacing}
            onEditStyle={onEditStyle}
            onClearDirect={handleClearParagraphFormatting}
            onEdit={() => onEditProperty?.('lineSpacing', 'paragraph')}
          />
        </div>
      </section>

      {/* Character Properties Section */}
      <section className="inspector-section">
        <div className="section-header">
          <h4>Character Properties</h4>
          {data.hasDirectCharacterFormatting && (
            <button
              className="clear-section-btn"
              onClick={handleClearCharacterFormatting}
              title="Clear character direct formatting"
            >
              Clear
            </button>
          )}
        </div>
        <div className="properties-list">
          <PropertyRow
            label="Font Family"
            property={data.characterProps.fontFamily}
            onEditStyle={onEditStyle}
            onClearDirect={handleClearCharacterFormatting}
            onEdit={() => onEditProperty?.('fontFamily', 'character')}
          />
          <PropertyRow
            label="Font Size"
            property={data.characterProps.fontSize}
            formatValue={formatPoints}
            onEditStyle={onEditStyle}
            onClearDirect={handleClearCharacterFormatting}
            onEdit={() => onEditProperty?.('fontSize', 'character')}
          />
          <PropertyRow
            label="Bold"
            property={data.characterProps.bold}
            formatValue={formatBoolean}
            onEditStyle={onEditStyle}
            onClearDirect={handleClearCharacterFormatting}
            onEdit={() => onEditProperty?.('bold', 'character')}
          />
          <PropertyRow
            label="Italic"
            property={data.characterProps.italic}
            formatValue={formatBoolean}
            onEditStyle={onEditStyle}
            onClearDirect={handleClearCharacterFormatting}
            onEdit={() => onEditProperty?.('italic', 'character')}
          />
          <PropertyRow
            label="Underline"
            property={data.characterProps.underline}
            formatValue={formatBoolean}
            onEditStyle={onEditStyle}
            onClearDirect={handleClearCharacterFormatting}
            onEdit={() => onEditProperty?.('underline', 'character')}
          />
          <PropertyRow
            label="Color"
            property={data.characterProps.color}
            formatValue={(color: string) => (
              <span style={{ display: 'inline-flex', alignItems: 'center', gap: '4px' }}>
                <span
                  className="color-swatch"
                  style={{ backgroundColor: color }}
                />
                {color}
              </span>
            ) as unknown as string}
            onEditStyle={onEditStyle}
            onClearDirect={handleClearCharacterFormatting}
            onEdit={() => onEditProperty?.('color', 'character')}
          />
        </div>
      </section>
    </div>
  );
}

export default StyleInspector;
