import { useState, useCallback, useRef, useEffect } from 'react';
import {
  BULLET_STYLES,
  NUMBERED_STYLES,
  ListStyleDefinition,
  LIST_STYLE_IDS,
} from '../lib/types';

interface ListGalleryProps {
  isOpen: boolean;
  onClose: () => void;
  onSelectStyle: (numId: number) => void;
  onRemoveList: () => void;
  anchorRect?: DOMRect;
  isBulletGallery: boolean;
}

export function ListGallery({
  isOpen,
  onClose,
  onSelectStyle,
  onRemoveList,
  anchorRect,
  isBulletGallery,
}: ListGalleryProps) {
  const menuRef = useRef<HTMLDivElement>(null);

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

  const styles = isBulletGallery ? BULLET_STYLES : NUMBERED_STYLES;

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
    <div ref={menuRef} className="list-gallery" style={style}>
      <div className="list-gallery-title">
        {isBulletGallery ? 'Bullet Library' : 'Numbering Library'}
      </div>
      <div className="list-gallery-grid">
        {styles.map((listStyle) => (
          <button
            key={listStyle.id}
            className="list-gallery-item"
            onClick={() => {
              onSelectStyle(listStyle.id);
              onClose();
            }}
            title={listStyle.name}
          >
            <div className="list-preview">
              {listStyle.preview.map((item, idx) => (
                <div
                  key={idx}
                  className="list-preview-item"
                  style={{ paddingLeft: `${idx * 12}px` }}
                >
                  <span className="list-marker">{item}</span>
                  <span className="list-text">Text</span>
                </div>
              ))}
            </div>
          </button>
        ))}
      </div>
      <div className="list-gallery-divider" />
      <button
        className="list-gallery-action"
        onClick={() => {
          onRemoveList();
          onClose();
        }}
      >
        Remove List
      </button>
    </div>
  );
}

interface ListButtonProps {
  isBullet: boolean;
  onToggle: () => void;
  onSelectStyle: (numId: number) => void;
  onRemoveList: () => void;
}

export function ListButton({
  isBullet,
  onToggle,
  onSelectStyle,
  onRemoveList,
}: ListButtonProps) {
  const [showGallery, setShowGallery] = useState(false);
  const buttonRef = useRef<HTMLButtonElement>(null);
  const [anchorRect, setAnchorRect] = useState<DOMRect | undefined>();

  const handleClick = useCallback(() => {
    onToggle();
  }, [onToggle]);

  const handleDropdownClick = useCallback(
    (e: React.MouseEvent) => {
      e.stopPropagation();
      if (buttonRef.current) {
        setAnchorRect(buttonRef.current.getBoundingClientRect());
      }
      setShowGallery(!showGallery);
    },
    [showGallery]
  );

  return (
    <div className="list-button-container">
      <button
        ref={buttonRef}
        onClick={handleClick}
        className="toolbar-icon-button"
        title={isBullet ? 'Bullets' : 'Numbering'}
      >
        {isBullet ? (
          <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
            <circle cx="3" cy="3" r="1.5" />
            <circle cx="3" cy="8" r="1.5" />
            <circle cx="3" cy="13" r="1.5" />
            <rect x="6" y="2" width="9" height="2" />
            <rect x="6" y="7" width="9" height="2" />
            <rect x="6" y="12" width="9" height="2" />
          </svg>
        ) : (
          <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
            <text x="2" y="4.5" fontSize="5" fontFamily="sans-serif">
              1.
            </text>
            <text x="2" y="9.5" fontSize="5" fontFamily="sans-serif">
              2.
            </text>
            <text x="2" y="14.5" fontSize="5" fontFamily="sans-serif">
              3.
            </text>
            <rect x="8" y="2" width="7" height="2" />
            <rect x="8" y="7" width="7" height="2" />
            <rect x="8" y="12" width="7" height="2" />
          </svg>
        )}
      </button>
      <button
        className="list-dropdown-arrow"
        onClick={handleDropdownClick}
        title={`${isBullet ? 'Bullet' : 'Numbering'} options`}
      >
        <svg width="8" height="8" viewBox="0 0 8 8" fill="currentColor">
          <path d="M0 2l4 4 4-4z" />
        </svg>
      </button>
      <ListGallery
        isOpen={showGallery}
        onClose={() => setShowGallery(false)}
        onSelectStyle={onSelectStyle}
        onRemoveList={onRemoveList}
        anchorRect={anchorRect}
        isBulletGallery={isBullet}
      />
    </div>
  );
}
