import { useState, useCallback, useRef, useEffect } from 'react';
import type { ShapeTypeName } from '../lib/types';

// =============================================================================
// Shape Category Definitions
// =============================================================================

interface ShapeDefinition {
  type: ShapeTypeName;
  name: string;
  icon: JSX.Element;
  defaultWidth: number;
  defaultHeight: number;
}

interface ShapeCategory {
  name: string;
  shapes: ShapeDefinition[];
}

// Shape categories with their shapes
const SHAPE_CATEGORIES: ShapeCategory[] = [
  {
    name: 'Basic Shapes',
    shapes: [
      {
        type: 'Rectangle',
        name: 'Rectangle',
        icon: (
          <svg viewBox="0 0 24 24" width="24" height="24">
            <rect x="2" y="4" width="20" height="16" fill="#4472C4" stroke="#333" strokeWidth="1" />
          </svg>
        ),
        defaultWidth: 100,
        defaultHeight: 75,
      },
      {
        type: 'RoundedRectangle',
        name: 'Rounded Rectangle',
        icon: (
          <svg viewBox="0 0 24 24" width="24" height="24">
            <rect x="2" y="4" width="20" height="16" rx="4" ry="4" fill="#4472C4" stroke="#333" strokeWidth="1" />
          </svg>
        ),
        defaultWidth: 100,
        defaultHeight: 75,
      },
      {
        type: 'Oval',
        name: 'Oval',
        icon: (
          <svg viewBox="0 0 24 24" width="24" height="24">
            <ellipse cx="12" cy="12" rx="10" ry="7" fill="#4472C4" stroke="#333" strokeWidth="1" />
          </svg>
        ),
        defaultWidth: 100,
        defaultHeight: 75,
      },
      {
        type: 'Triangle',
        name: 'Triangle',
        icon: (
          <svg viewBox="0 0 24 24" width="24" height="24">
            <polygon points="12,2 22,22 2,22" fill="#4472C4" stroke="#333" strokeWidth="1" />
          </svg>
        ),
        defaultWidth: 100,
        defaultHeight: 100,
      },
      {
        type: 'Diamond',
        name: 'Diamond',
        icon: (
          <svg viewBox="0 0 24 24" width="24" height="24">
            <polygon points="12,2 22,12 12,22 2,12" fill="#4472C4" stroke="#333" strokeWidth="1" />
          </svg>
        ),
        defaultWidth: 100,
        defaultHeight: 100,
      },
      {
        type: 'Pentagon',
        name: 'Pentagon',
        icon: (
          <svg viewBox="0 0 24 24" width="24" height="24">
            <polygon points="12,2 22,9 19,21 5,21 2,9" fill="#4472C4" stroke="#333" strokeWidth="1" />
          </svg>
        ),
        defaultWidth: 100,
        defaultHeight: 100,
      },
      {
        type: 'Hexagon',
        name: 'Hexagon',
        icon: (
          <svg viewBox="0 0 24 24" width="24" height="24">
            <polygon points="12,2 21,7 21,17 12,22 3,17 3,7" fill="#4472C4" stroke="#333" strokeWidth="1" />
          </svg>
        ),
        defaultWidth: 100,
        defaultHeight: 100,
      },
    ],
  },
  {
    name: 'Lines & Arrows',
    shapes: [
      {
        type: 'Line',
        name: 'Line',
        icon: (
          <svg viewBox="0 0 24 24" width="24" height="24">
            <line x1="2" y1="22" x2="22" y2="2" stroke="#333" strokeWidth="2" />
          </svg>
        ),
        defaultWidth: 100,
        defaultHeight: 0,
      },
      {
        type: 'Arrow',
        name: 'Arrow',
        icon: (
          <svg viewBox="0 0 24 24" width="24" height="24">
            <line x1="2" y1="12" x2="18" y2="12" stroke="#333" strokeWidth="2" />
            <polygon points="22,12 16,8 16,16" fill="#333" />
          </svg>
        ),
        defaultWidth: 100,
        defaultHeight: 0,
      },
      {
        type: 'DoubleArrow',
        name: 'Double Arrow',
        icon: (
          <svg viewBox="0 0 24 24" width="24" height="24">
            <line x1="6" y1="12" x2="18" y2="12" stroke="#333" strokeWidth="2" />
            <polygon points="2,12 8,8 8,16" fill="#333" />
            <polygon points="22,12 16,8 16,16" fill="#333" />
          </svg>
        ),
        defaultWidth: 100,
        defaultHeight: 0,
      },
    ],
  },
  {
    name: 'Block Arrows',
    shapes: [
      {
        type: 'RightArrowBlock',
        name: 'Right Arrow',
        icon: (
          <svg viewBox="0 0 24 24" width="24" height="24">
            <polygon points="2,8 14,8 14,4 22,12 14,20 14,16 2,16" fill="#4472C4" stroke="#333" strokeWidth="1" />
          </svg>
        ),
        defaultWidth: 100,
        defaultHeight: 60,
      },
      {
        type: 'LeftArrowBlock',
        name: 'Left Arrow',
        icon: (
          <svg viewBox="0 0 24 24" width="24" height="24">
            <polygon points="22,8 10,8 10,4 2,12 10,20 10,16 22,16" fill="#4472C4" stroke="#333" strokeWidth="1" />
          </svg>
        ),
        defaultWidth: 100,
        defaultHeight: 60,
      },
      {
        type: 'UpArrowBlock',
        name: 'Up Arrow',
        icon: (
          <svg viewBox="0 0 24 24" width="24" height="24">
            <polygon points="8,22 8,10 4,10 12,2 20,10 16,10 16,22" fill="#4472C4" stroke="#333" strokeWidth="1" />
          </svg>
        ),
        defaultWidth: 60,
        defaultHeight: 100,
      },
      {
        type: 'DownArrowBlock',
        name: 'Down Arrow',
        icon: (
          <svg viewBox="0 0 24 24" width="24" height="24">
            <polygon points="8,2 8,14 4,14 12,22 20,14 16,14 16,2" fill="#4472C4" stroke="#333" strokeWidth="1" />
          </svg>
        ),
        defaultWidth: 60,
        defaultHeight: 100,
      },
    ],
  },
  {
    name: 'Stars & Banners',
    shapes: [
      {
        type: 'Star',
        name: '5-Point Star',
        icon: (
          <svg viewBox="0 0 24 24" width="24" height="24">
            <polygon
              points="12,2 14.5,9 22,9 16,14 18,22 12,17 6,22 8,14 2,9 9.5,9"
              fill="#4472C4"
              stroke="#333"
              strokeWidth="1"
            />
          </svg>
        ),
        defaultWidth: 100,
        defaultHeight: 100,
      },
    ],
  },
  {
    name: 'Callouts',
    shapes: [
      {
        type: 'Callout',
        name: 'Rectangular Callout',
        icon: (
          <svg viewBox="0 0 24 24" width="24" height="24">
            <path d="M2,2 L22,2 L22,16 L14,16 L10,22 L10,16 L2,16 Z" fill="#4472C4" stroke="#333" strokeWidth="1" />
          </svg>
        ),
        defaultWidth: 150,
        defaultHeight: 100,
      },
      {
        type: 'TextBox',
        name: 'Text Box',
        icon: (
          <svg viewBox="0 0 24 24" width="24" height="24">
            <rect x="2" y="4" width="20" height="16" fill="white" stroke="#333" strokeWidth="1" />
            <text x="6" y="14" fontSize="8" fill="#333">
              Aa
            </text>
          </svg>
        ),
        defaultWidth: 150,
        defaultHeight: 100,
      },
    ],
  },
];

// =============================================================================
// ShapeGallery Props
// =============================================================================

interface ShapeGalleryProps {
  isOpen: boolean;
  onClose: () => void;
  onSelectShape: (shapeType: ShapeTypeName, width: number, height: number) => void;
  anchorRect?: DOMRect;
}

// =============================================================================
// ShapeGallery Component
// =============================================================================

export function ShapeGallery({ isOpen, onClose, onSelectShape, anchorRect }: ShapeGalleryProps) {
  const menuRef = useRef<HTMLDivElement>(null);
  const [selectedCategory, setSelectedCategory] = useState<string>(SHAPE_CATEGORIES[0].name);

  useEffect(() => {
    if (!isOpen) return;

    const handleClickOutside = (event: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
        onClose();
      }
    };

    const handleEscape = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        onClose();
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    document.addEventListener('keydown', handleEscape);

    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
      document.removeEventListener('keydown', handleEscape);
    };
  }, [isOpen, onClose]);

  if (!isOpen) return null;

  const currentCategory = SHAPE_CATEGORIES.find((c) => c.name === selectedCategory) || SHAPE_CATEGORIES[0];

  // Calculate position
  const style: React.CSSProperties = anchorRect
    ? {
        position: 'fixed',
        top: anchorRect.bottom + 4,
        left: anchorRect.left,
        zIndex: 1000,
      }
    : {
        position: 'absolute',
        top: '100%',
        left: 0,
        zIndex: 1000,
      };

  return (
    <div ref={menuRef} className="shape-gallery" style={style}>
      <div className="shape-gallery-header">
        <span className="shape-gallery-title">Shapes</span>
      </div>

      <div className="shape-gallery-categories">
        {SHAPE_CATEGORIES.map((category) => (
          <button
            key={category.name}
            className={`shape-category-tab ${selectedCategory === category.name ? 'active' : ''}`}
            onClick={() => setSelectedCategory(category.name)}
          >
            {category.name}
          </button>
        ))}
      </div>

      <div className="shape-gallery-grid">
        {currentCategory.shapes.map((shape) => (
          <button
            key={shape.type}
            className="shape-gallery-item"
            onClick={() => {
              onSelectShape(shape.type, shape.defaultWidth, shape.defaultHeight);
              onClose();
            }}
            title={shape.name}
          >
            {shape.icon}
            <span className="shape-name">{shape.name}</span>
          </button>
        ))}
      </div>

      <style>{`
        .shape-gallery {
          background: white;
          border: 1px solid #ccc;
          border-radius: 4px;
          box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
          width: 320px;
          max-height: 400px;
          overflow: hidden;
          display: flex;
          flex-direction: column;
        }

        .shape-gallery-header {
          padding: 12px 16px;
          border-bottom: 1px solid #e0e0e0;
          background: #f5f5f5;
        }

        .shape-gallery-title {
          font-weight: 600;
          font-size: 14px;
          color: #333;
        }

        .shape-gallery-categories {
          display: flex;
          flex-wrap: wrap;
          gap: 4px;
          padding: 8px;
          border-bottom: 1px solid #e0e0e0;
          background: #fafafa;
        }

        .shape-category-tab {
          padding: 4px 8px;
          font-size: 11px;
          border: 1px solid transparent;
          border-radius: 4px;
          background: transparent;
          cursor: pointer;
          color: #666;
          transition: all 0.15s;
        }

        .shape-category-tab:hover {
          background: #e8e8e8;
          color: #333;
        }

        .shape-category-tab.active {
          background: #0066cc;
          color: white;
          border-color: #0052a3;
        }

        .shape-gallery-grid {
          display: grid;
          grid-template-columns: repeat(4, 1fr);
          gap: 8px;
          padding: 12px;
          overflow-y: auto;
        }

        .shape-gallery-item {
          display: flex;
          flex-direction: column;
          align-items: center;
          gap: 4px;
          padding: 8px;
          border: 1px solid #e0e0e0;
          border-radius: 4px;
          background: white;
          cursor: pointer;
          transition: all 0.15s;
        }

        .shape-gallery-item:hover {
          border-color: #0066cc;
          background: #f0f7ff;
        }

        .shape-gallery-item svg {
          width: 32px;
          height: 32px;
        }

        .shape-name {
          font-size: 9px;
          color: #666;
          text-align: center;
          max-width: 100%;
          overflow: hidden;
          text-overflow: ellipsis;
          white-space: nowrap;
        }
      `}</style>
    </div>
  );
}

// =============================================================================
// ShapeButton Component (for Toolbar)
// =============================================================================

interface ShapeButtonProps {
  onSelectShape: (shapeType: ShapeTypeName, width: number, height: number) => void;
}

export function ShapeButton({ onSelectShape }: ShapeButtonProps) {
  const [showGallery, setShowGallery] = useState(false);
  const buttonRef = useRef<HTMLButtonElement>(null);
  const [anchorRect, setAnchorRect] = useState<DOMRect | undefined>();

  const handleClick = useCallback(() => {
    if (buttonRef.current) {
      setAnchorRect(buttonRef.current.getBoundingClientRect());
    }
    setShowGallery(!showGallery);
  }, [showGallery]);

  return (
    <div className="shape-button-container" style={{ position: 'relative' }}>
      <button
        ref={buttonRef}
        onClick={handleClick}
        className="toolbar-icon-button"
        title="Insert Shape"
      >
        <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
          <circle cx="5" cy="5" r="4" fill="none" stroke="currentColor" strokeWidth="1.5" />
          <rect x="7" y="7" width="8" height="8" fill="none" stroke="currentColor" strokeWidth="1.5" />
        </svg>
      </button>
      <ShapeGallery
        isOpen={showGallery}
        onClose={() => setShowGallery(false)}
        onSelectShape={onSelectShape}
        anchorRect={anchorRect}
      />
    </div>
  );
}

// Export shape definitions for use elsewhere
export { SHAPE_CATEGORIES, type ShapeDefinition, type ShapeCategory };
