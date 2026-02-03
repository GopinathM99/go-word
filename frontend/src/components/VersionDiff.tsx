import React, { useState, useCallback } from 'react';
import './VersionDiff.css';

interface VersionDiffProps {
  isOpen: boolean;
  onClose: () => void;
  fromVersion: VersionInfo;
  toVersion: VersionInfo;
  diffContent: DiffContent[];
  onNavigateToDiff: (index: number) => void;
}

interface VersionInfo {
  id: string;
  name?: string;
  timestamp: Date;
  author: string;
}

interface DiffContent {
  type: 'unchanged' | 'added' | 'removed' | 'modified';
  content: string;
  location?: string; // e.g., "Paragraph 5"
}

export const VersionDiff: React.FC<VersionDiffProps> = ({
  isOpen,
  onClose,
  fromVersion,
  toVersion,
  diffContent,
  onNavigateToDiff,
}) => {
  const [currentChangeIndex, setCurrentChangeIndex] = useState(0);
  const [viewMode, setViewMode] = useState<'unified' | 'split'>('unified');
  const [showUnchanged, setShowUnchanged] = useState(false);

  const changes = diffContent.filter((d) => d.type !== 'unchanged');
  const addedCount = diffContent.filter((d) => d.type === 'added').length;
  const removedCount = diffContent.filter((d) => d.type === 'removed').length;
  const modifiedCount = diffContent.filter((d) => d.type === 'modified').length;

  const displayedContent = showUnchanged
    ? diffContent
    : diffContent.filter((d) => d.type !== 'unchanged');

  const navigatePrevious = useCallback(() => {
    if (changes.length === 0) return;
    const newIndex = currentChangeIndex > 0 ? currentChangeIndex - 1 : changes.length - 1;
    setCurrentChangeIndex(newIndex);
    onNavigateToDiff(newIndex);
  }, [changes.length, currentChangeIndex, onNavigateToDiff]);

  const navigateNext = useCallback(() => {
    if (changes.length === 0) return;
    const newIndex = currentChangeIndex < changes.length - 1 ? currentChangeIndex + 1 : 0;
    setCurrentChangeIndex(newIndex);
    onNavigateToDiff(newIndex);
  }, [changes.length, currentChangeIndex, onNavigateToDiff]);

  const handleItemClick = useCallback((index: number) => {
    const changeIndex = changes.findIndex((_, i) => {
      const originalIndex = diffContent.findIndex((d, j) => d === changes[i] && j >= index);
      return originalIndex === index;
    });
    if (changeIndex !== -1) {
      setCurrentChangeIndex(changeIndex);
    }
    onNavigateToDiff(index);
  }, [changes, diffContent, onNavigateToDiff]);

  const formatVersionLabel = (version: VersionInfo) => {
    if (version.name) {
      return version.name;
    }
    return version.timestamp.toLocaleString();
  };

  if (!isOpen) return null;

  return (
    <div className="version-diff-overlay" onClick={onClose}>
      <div className="version-diff" onClick={(e) => e.stopPropagation()}>
        <div className="version-diff__header">
          <h2>Compare Versions</h2>
          <div className="version-diff__header-actions">
            <div className="version-diff__view-toggle">
              <button
                className={viewMode === 'unified' ? 'active' : ''}
                onClick={() => setViewMode('unified')}
              >
                Unified
              </button>
              <button
                className={viewMode === 'split' ? 'active' : ''}
                onClick={() => setViewMode('split')}
              >
                Split
              </button>
            </div>
            <button className="version-diff__close" onClick={onClose}>
              ×
            </button>
          </div>
        </div>

        <div className="version-diff__versions">
          <div className="version-diff__version from">
            <span className="version-diff__label">From:</span>
            <span className="version-diff__name">
              {formatVersionLabel(fromVersion)}
            </span>
            <span className="version-diff__author">by {fromVersion.author}</span>
          </div>
          <span className="version-diff__arrow">→</span>
          <div className="version-diff__version to">
            <span className="version-diff__label">To:</span>
            <span className="version-diff__name">
              {formatVersionLabel(toVersion)}
            </span>
            <span className="version-diff__author">by {toVersion.author}</span>
          </div>
        </div>

        <div className="version-diff__summary">
          <span className="version-diff__stat added">+{addedCount} added</span>
          <span className="version-diff__stat removed">-{removedCount} removed</span>
          <span className="version-diff__stat modified">~{modifiedCount} modified</span>
          <label className="version-diff__toggle">
            <input
              type="checkbox"
              checked={showUnchanged}
              onChange={(e) => setShowUnchanged(e.target.checked)}
            />
            Show unchanged
          </label>
        </div>

        <div className={`version-diff__content ${viewMode}`}>
          {displayedContent.length === 0 ? (
            <div className="version-diff__no-changes">
              No changes between these versions
            </div>
          ) : viewMode === 'unified' ? (
            displayedContent.map((diff, index) => (
              <div
                key={index}
                className={`version-diff__item ${diff.type}`}
                onClick={() => handleItemClick(index)}
              >
                <div className="version-diff__item-indicator">
                  {diff.type === 'added' && '+'}
                  {diff.type === 'removed' && '-'}
                  {diff.type === 'modified' && '~'}
                  {diff.type === 'unchanged' && ' '}
                </div>
                <div className="version-diff__item-content">
                  {diff.location && (
                    <div className="version-diff__location">{diff.location}</div>
                  )}
                  <div className="version-diff__text">{diff.content}</div>
                </div>
              </div>
            ))
          ) : (
            <div className="version-diff__split-view">
              <div className="version-diff__split-pane left">
                <div className="version-diff__split-header">
                  {formatVersionLabel(fromVersion)}
                </div>
                {displayedContent.map((diff, index) => (
                  <div
                    key={index}
                    className={`version-diff__split-item ${
                      diff.type === 'removed' || diff.type === 'modified' ? 'highlight-old' : ''
                    } ${diff.type === 'added' ? 'empty' : ''}`}
                    onClick={() => handleItemClick(index)}
                  >
                    {diff.type !== 'added' && (
                      <>
                        {diff.location && (
                          <div className="version-diff__location">{diff.location}</div>
                        )}
                        <div className="version-diff__text">
                          {diff.type === 'modified' ? diff.content.split(' → ')[0] : diff.content}
                        </div>
                      </>
                    )}
                  </div>
                ))}
              </div>
              <div className="version-diff__split-pane right">
                <div className="version-diff__split-header">
                  {formatVersionLabel(toVersion)}
                </div>
                {displayedContent.map((diff, index) => (
                  <div
                    key={index}
                    className={`version-diff__split-item ${
                      diff.type === 'added' || diff.type === 'modified' ? 'highlight-new' : ''
                    } ${diff.type === 'removed' ? 'empty' : ''}`}
                    onClick={() => handleItemClick(index)}
                  >
                    {diff.type !== 'removed' && (
                      <>
                        {diff.location && (
                          <div className="version-diff__location">{diff.location}</div>
                        )}
                        <div className="version-diff__text">
                          {diff.type === 'modified' ? diff.content.split(' → ')[1] || diff.content : diff.content}
                        </div>
                      </>
                    )}
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>

        <div className="version-diff__navigation">
          <span className="version-diff__nav-info">
            {changes.length > 0 ? (
              <>Change {currentChangeIndex + 1} of {changes.length}</>
            ) : (
              <>No changes</>
            )}
          </span>
          <div className="version-diff__nav-buttons">
            <button onClick={navigatePrevious} disabled={changes.length === 0}>
              ← Previous
            </button>
            <button onClick={navigateNext} disabled={changes.length === 0}>
              Next →
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};
