/**
 * StyleGallery - Grid/list of available styles with preview
 *
 * Shows all paragraph styles in a gallery format, with visual previews.
 * Click to apply style to current selection.
 */

import { useState, useCallback, useMemo } from 'react';
import { Style, StyleType } from '../lib/types';
import './StyleGallery.css';

interface StyleGalleryProps {
  /** Available styles to display */
  styles: Style[];
  /** Currently applied style ID (for highlighting) */
  activeStyleId?: string;
  /** Callback when a style is selected */
  onSelectStyle: (styleId: string) => void;
  /** Callback to open style editor */
  onEditStyle?: (styleId: string) => void;
  /** Callback to create new style */
  onCreateStyle?: () => void;
  /** Show only specific style type */
  filterType?: StyleType;
  /** Display mode */
  viewMode?: 'gallery' | 'list';
}

interface StylePreviewProps {
  style: Style;
  isActive: boolean;
  onClick: () => void;
  onEdit?: () => void;
  viewMode: 'gallery' | 'list';
}

function StylePreview({
  style,
  isActive,
  onClick,
  onEdit,
  viewMode,
}: StylePreviewProps) {
  // Generate preview styles based on the style's properties
  const previewStyle = useMemo(() => {
    const props = style.characterProps;
    const paraProps = style.paragraphProps;

    return {
      fontFamily: props.fontFamily || 'Calibri',
      fontSize: Math.min(props.fontSize || 11, viewMode === 'gallery' ? 14 : 12),
      fontWeight: props.bold ? 'bold' : 'normal',
      fontStyle: props.italic ? 'italic' : 'normal',
      textDecoration: props.underline ? 'underline' : 'none',
      color: props.color || '#000000',
      textAlign: (paraProps.alignment || 'left') as React.CSSProperties['textAlign'],
    };
  }, [style, viewMode]);

  const handleContextMenu = useCallback(
    (e: React.MouseEvent) => {
      if (onEdit) {
        e.preventDefault();
        onEdit();
      }
    },
    [onEdit]
  );

  if (viewMode === 'list') {
    return (
      <div
        className={`style-list-item ${isActive ? 'active' : ''}`}
        onClick={onClick}
        onContextMenu={handleContextMenu}
        title={`${style.name}${style.basedOn ? ` (based on ${style.basedOn})` : ''}`}
      >
        <span className="style-preview-text" style={previewStyle}>
          {style.name}
        </span>
        <span className="style-info">
          {style.styleType === 'character' && (
            <span className="style-type-badge">Char</span>
          )}
          {!style.builtIn && <span className="custom-badge">Custom</span>}
        </span>
      </div>
    );
  }

  return (
    <div
      className={`style-gallery-item ${isActive ? 'active' : ''}`}
      onClick={onClick}
      onContextMenu={handleContextMenu}
      title={`${style.name}${style.basedOn ? ` (based on ${style.basedOn})` : ''}`}
    >
      <div className="style-preview-container">
        <span className="style-preview-text" style={previewStyle}>
          {style.name}
        </span>
      </div>
      <div className="style-label">{style.name}</div>
    </div>
  );
}

export function StyleGallery({
  styles,
  activeStyleId,
  onSelectStyle,
  onEditStyle,
  onCreateStyle,
  filterType = 'paragraph',
  viewMode = 'gallery',
}: StyleGalleryProps) {
  const [searchQuery, setSearchQuery] = useState('');
  const [expandedView, setExpandedView] = useState(false);

  // Filter and sort styles
  const filteredStyles = useMemo(() => {
    let filtered = styles.filter((s) => s.styleType === filterType && !s.hidden);

    if (searchQuery) {
      const query = searchQuery.toLowerCase();
      filtered = filtered.filter(
        (s) =>
          s.name.toLowerCase().includes(query) ||
          s.id.toLowerCase().includes(query)
      );
    }

    // Sort by priority, then alphabetically
    return filtered.sort((a, b) => {
      if (a.priority !== b.priority) {
        return a.priority - b.priority;
      }
      return a.name.localeCompare(b.name);
    });
  }, [styles, filterType, searchQuery]);

  // Group styles by category
  const groupedStyles = useMemo(() => {
    const groups: Record<string, Style[]> = {
      'Heading Styles': [],
      'Body Styles': [],
      'Other Styles': [],
    };

    for (const style of filteredStyles) {
      if (style.name.toLowerCase().includes('heading') || style.name === 'Title' || style.name === 'Subtitle') {
        groups['Heading Styles'].push(style);
      } else if (style.name === 'Normal' || style.name === 'No Spacing' || style.name.includes('Quote')) {
        groups['Body Styles'].push(style);
      } else {
        groups['Other Styles'].push(style);
      }
    }

    // Remove empty groups
    return Object.fromEntries(
      Object.entries(groups).filter(([_, styles]) => styles.length > 0)
    );
  }, [filteredStyles]);

  const handleStyleClick = useCallback(
    (styleId: string) => {
      onSelectStyle(styleId);
    },
    [onSelectStyle]
  );

  const handleStyleEdit = useCallback(
    (styleId: string) => {
      onEditStyle?.(styleId);
    },
    [onEditStyle]
  );

  // Display styles either as flat list or grouped
  const displayedStyles = expandedView ? filteredStyles : filteredStyles.slice(0, 8);

  return (
    <div className={`style-gallery ${viewMode}`}>
      <div className="gallery-header">
        <div className="search-container">
          <input
            type="text"
            placeholder="Search styles..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="style-search"
          />
        </div>
        <div className="gallery-actions">
          {onCreateStyle && (
            <button
              className="create-style-btn"
              onClick={onCreateStyle}
              title="Create new style"
            >
              + New
            </button>
          )}
          <button
            className="view-toggle-btn"
            onClick={() => setExpandedView(!expandedView)}
            title={expandedView ? 'Show less' : 'Show all styles'}
          >
            {expandedView ? 'Show Less' : `Show All (${filteredStyles.length})`}
          </button>
        </div>
      </div>

      {expandedView ? (
        // Expanded view with groups
        <div className="gallery-groups">
          {Object.entries(groupedStyles).map(([groupName, groupStyles]) => (
            <div key={groupName} className="style-group">
              <h4 className="group-title">{groupName}</h4>
              <div className={`style-items ${viewMode}`}>
                {groupStyles.map((style) => (
                  <StylePreview
                    key={style.id}
                    style={style}
                    isActive={style.id === activeStyleId}
                    onClick={() => handleStyleClick(style.id)}
                    onEdit={() => handleStyleEdit(style.id)}
                    viewMode={viewMode}
                  />
                ))}
              </div>
            </div>
          ))}
        </div>
      ) : (
        // Compact view
        <div className={`style-items ${viewMode}`}>
          {displayedStyles.map((style) => (
            <StylePreview
              key={style.id}
              style={style}
              isActive={style.id === activeStyleId}
              onClick={() => handleStyleClick(style.id)}
              onEdit={() => handleStyleEdit(style.id)}
              viewMode={viewMode}
            />
          ))}
        </div>
      )}

      {filteredStyles.length === 0 && (
        <div className="no-styles-message">
          {searchQuery
            ? `No styles matching "${searchQuery}"`
            : 'No styles available'}
        </div>
      )}
    </div>
  );
}

export default StyleGallery;
