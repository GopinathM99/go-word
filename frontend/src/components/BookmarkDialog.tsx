import { useState, useCallback, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import './BookmarkDialog.css';

/**
 * Bookmark data from the backend
 */
interface BookmarkData {
  id: string;
  name: string;
  isPoint: boolean;
  preview: string | null;
  paragraphId: string;
  offset: number;
}

interface BookmarkDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onInsert: (name: string) => void;
  onGoTo: (name: string) => void;
  onDelete: (name: string) => void;
  docId?: string;
}

/**
 * Validate bookmark name
 * - Must start with a letter
 * - Can only contain letters, numbers, and underscores
 * - Max 40 characters
 */
function validateBookmarkName(name: string): { valid: boolean; error?: string } {
  if (!name.trim()) {
    return { valid: false, error: 'Bookmark name cannot be empty' };
  }

  if (name.length > 40) {
    return { valid: false, error: 'Bookmark name cannot exceed 40 characters' };
  }

  const firstChar = name.charAt(0);
  if (!/^[a-zA-Z]$/.test(firstChar)) {
    return { valid: false, error: 'Bookmark name must start with a letter' };
  }

  if (!/^[a-zA-Z][a-zA-Z0-9_]*$/.test(name)) {
    return { valid: false, error: 'Bookmark name can only contain letters, numbers, and underscores' };
  }

  return { valid: true };
}

export function BookmarkDialog({
  isOpen,
  onClose,
  onInsert,
  onGoTo,
  onDelete,
  docId = 'default',
}: BookmarkDialogProps) {
  const [bookmarks, setBookmarks] = useState<BookmarkData[]>([]);
  const [selectedBookmark, setSelectedBookmark] = useState<string | null>(null);
  const [newBookmarkName, setNewBookmarkName] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [sortBy, setSortBy] = useState<'name' | 'position'>('name');

  // Load bookmarks when dialog opens
  useEffect(() => {
    if (isOpen) {
      loadBookmarks();
      setNewBookmarkName('');
      setSelectedBookmark(null);
      setError(null);
    }
  }, [isOpen, docId]);

  const loadBookmarks = async () => {
    setIsLoading(true);
    try {
      const result = await invoke<BookmarkData[]>('list_bookmarks', { docId });
      setBookmarks(result);
    } catch (e) {
      console.error('Failed to load bookmarks:', e);
      setBookmarks([]);
    } finally {
      setIsLoading(false);
    }
  };

  // Sort bookmarks
  const sortedBookmarks = [...bookmarks].sort((a, b) => {
    if (sortBy === 'name') {
      return a.name.localeCompare(b.name);
    } else {
      // Sort by position (paragraph id then offset)
      if (a.paragraphId !== b.paragraphId) {
        return a.paragraphId.localeCompare(b.paragraphId);
      }
      return a.offset - b.offset;
    }
  });

  const handleAdd = useCallback(async () => {
    setError(null);

    const validation = validateBookmarkName(newBookmarkName);
    if (!validation.valid) {
      setError(validation.error || 'Invalid bookmark name');
      return;
    }

    // Check for duplicate name
    const exists = bookmarks.some(
      (b) => b.name.toLowerCase() === newBookmarkName.toLowerCase()
    );
    if (exists) {
      setError('A bookmark with this name already exists');
      return;
    }

    try {
      onInsert(newBookmarkName);
      setNewBookmarkName('');
      // Reload bookmarks after insertion
      await loadBookmarks();
    } catch (e) {
      setError(String(e));
    }
  }, [newBookmarkName, bookmarks, onInsert]);

  const handleDelete = useCallback(async () => {
    if (!selectedBookmark) return;

    try {
      onDelete(selectedBookmark);
      setSelectedBookmark(null);
      // Reload bookmarks after deletion
      await loadBookmarks();
    } catch (e) {
      setError(String(e));
    }
  }, [selectedBookmark, onDelete]);

  const handleGoTo = useCallback(() => {
    if (!selectedBookmark) return;
    onGoTo(selectedBookmark);
    onClose();
  }, [selectedBookmark, onGoTo, onClose]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === 'Escape') {
        onClose();
      } else if (e.key === 'Enter' && newBookmarkName.trim()) {
        handleAdd();
      }
    },
    [onClose, handleAdd, newBookmarkName]
  );

  const handleBookmarkClick = useCallback((name: string) => {
    setSelectedBookmark(name);
  }, []);

  const handleBookmarkDoubleClick = useCallback(
    (name: string) => {
      onGoTo(name);
      onClose();
    },
    [onGoTo, onClose]
  );

  if (!isOpen) {
    return null;
  }

  return (
    <div className="bookmark-dialog-overlay" onClick={onClose} onKeyDown={handleKeyDown}>
      <div
        className="bookmark-dialog"
        onClick={(e) => e.stopPropagation()}
        role="dialog"
        aria-labelledby="bookmark-dialog-title"
        aria-modal="true"
      >
        <header className="bookmark-dialog-header">
          <h2 id="bookmark-dialog-title">Bookmarks</h2>
          <button className="close-button" onClick={onClose} aria-label="Close dialog">
            X
          </button>
        </header>

        <div className="bookmark-dialog-content">
          {/* Add new bookmark section */}
          <div className="bookmark-add-section">
            <label htmlFor="bookmark-name">Bookmark name:</label>
            <div className="bookmark-add-row">
              <input
                id="bookmark-name"
                type="text"
                value={newBookmarkName}
                onChange={(e) => setNewBookmarkName(e.target.value)}
                placeholder="Enter bookmark name"
                maxLength={40}
                autoFocus
              />
              <button
                onClick={handleAdd}
                disabled={!newBookmarkName.trim()}
                className="add-button"
              >
                Add
              </button>
            </div>
            {error && <div className="error-message">{error}</div>}
          </div>

          {/* Bookmarks list section */}
          <div className="bookmark-list-section">
            <div className="bookmark-list-header">
              <span>Bookmarks in document:</span>
              <div className="sort-controls">
                <label htmlFor="sort-by">Sort by:</label>
                <select
                  id="sort-by"
                  value={sortBy}
                  onChange={(e) => setSortBy(e.target.value as 'name' | 'position')}
                >
                  <option value="name">Name</option>
                  <option value="position">Position</option>
                </select>
              </div>
            </div>

            {isLoading ? (
              <div className="bookmark-loading">Loading bookmarks...</div>
            ) : sortedBookmarks.length === 0 ? (
              <div className="bookmark-empty">No bookmarks in this document</div>
            ) : (
              <ul className="bookmark-list">
                {sortedBookmarks.map((bookmark) => (
                  <li
                    key={bookmark.id}
                    className={`bookmark-item ${selectedBookmark === bookmark.name ? 'selected' : ''}`}
                    onClick={() => handleBookmarkClick(bookmark.name)}
                    onDoubleClick={() => handleBookmarkDoubleClick(bookmark.name)}
                    tabIndex={0}
                    onKeyDown={(e) => {
                      if (e.key === 'Enter') {
                        handleBookmarkDoubleClick(bookmark.name);
                      }
                    }}
                  >
                    <span className="bookmark-name">{bookmark.name}</span>
                    {bookmark.preview && (
                      <span className="bookmark-preview">{bookmark.preview}</span>
                    )}
                    <span className="bookmark-type">
                      {bookmark.isPoint ? 'Point' : 'Range'}
                    </span>
                  </li>
                ))}
              </ul>
            )}
          </div>
        </div>

        <footer className="bookmark-dialog-footer">
          <div className="button-group-left">
            <button
              onClick={handleDelete}
              disabled={!selectedBookmark}
              className="delete-button"
            >
              Delete
            </button>
          </div>
          <div className="button-group-right">
            <button onClick={onClose} className="cancel-button">
              Close
            </button>
            <button
              onClick={handleGoTo}
              disabled={!selectedBookmark}
              className="goto-button"
            >
              Go To
            </button>
          </div>
        </footer>
      </div>
    </div>
  );
}

export default BookmarkDialog;
