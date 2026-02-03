import React, { useState, useMemo } from 'react';
import './VersionHistory.css';

interface VersionHistoryProps {
  isOpen: boolean;
  onClose: () => void;
  versions: VersionInfo[];
  currentVersionId?: string;
  isLoading: boolean;
  onPreview: (versionId: string) => void;
  onRestore: (versionId: string) => Promise<void>;
  onCompare: (fromId: string, toId: string) => void;
  onRename: (versionId: string, name: string) => Promise<void>;
  onCreateSnapshot: (name: string) => Promise<void>;
}

interface VersionInfo {
  id: string;
  timestamp: Date;
  author: string;
  name?: string;
  summary: string;
  isNamed: boolean;
  isCurrent: boolean;
}

export const VersionHistory: React.FC<VersionHistoryProps> = ({
  isOpen,
  onClose,
  versions,
  currentVersionId,
  isLoading,
  onPreview,
  onRestore,
  onCompare,
  onRename,
  onCreateSnapshot,
}) => {
  const [selectedVersions, setSelectedVersions] = useState<string[]>([]);
  const [compareMode, setCompareMode] = useState(false);
  const [previewingId, setPreviewingId] = useState<string | null>(null);
  const [showCreateSnapshot, setShowCreateSnapshot] = useState(false);
  const [snapshotName, setSnapshotName] = useState('');
  const [filter, setFilter] = useState<'all' | 'named'>('all');
  const [editingNameId, setEditingNameId] = useState<string | null>(null);
  const [newName, setNewName] = useState('');

  const filteredVersions = useMemo(() => {
    if (filter === 'named') {
      return versions.filter((v) => v.isNamed);
    }
    return versions;
  }, [versions, filter]);

  const handleVersionClick = (versionId: string) => {
    if (compareMode) {
      setSelectedVersions((prev) => {
        if (prev.includes(versionId)) {
          return prev.filter((id) => id !== versionId);
        }
        if (prev.length < 2) {
          return [...prev, versionId];
        }
        return [prev[1], versionId];
      });
    } else {
      setPreviewingId(versionId);
      onPreview(versionId);
    }
  };

  const handleCompare = () => {
    if (selectedVersions.length === 2) {
      onCompare(selectedVersions[0], selectedVersions[1]);
    }
  };

  const handleRestore = async (versionId: string) => {
    if (confirm('Restore to this version? This will create a new version with the restored content.')) {
      await onRestore(versionId);
    }
  };

  const handleCreateSnapshot = async () => {
    if (snapshotName.trim()) {
      await onCreateSnapshot(snapshotName.trim());
      setSnapshotName('');
      setShowCreateSnapshot(false);
    }
  };

  const handleRename = async (versionId: string) => {
    if (newName.trim()) {
      await onRename(versionId, newName.trim());
      setEditingNameId(null);
      setNewName('');
    }
  };

  const formatDate = (date: Date) => {
    const now = new Date();
    const diff = now.getTime() - date.getTime();
    const minutes = Math.floor(diff / 60000);
    const hours = Math.floor(diff / 3600000);
    const days = Math.floor(diff / 86400000);

    if (minutes < 1) return 'Just now';
    if (minutes < 60) return `${minutes}m ago`;
    if (hours < 24) return `${hours}h ago`;
    if (days < 7) return `${days}d ago`;
    return date.toLocaleDateString();
  };

  if (!isOpen) return null;

  return (
    <div className="version-history-panel">
      <div className="version-history__header">
        <h2>Version History</h2>
        <button className="version-history__close" onClick={onClose}>
          ×
        </button>
      </div>

      <div className="version-history__toolbar">
        <div className="version-history__filters">
          <button
            className={filter === 'all' ? 'active' : ''}
            onClick={() => setFilter('all')}
          >
            All Versions
          </button>
          <button
            className={filter === 'named' ? 'active' : ''}
            onClick={() => setFilter('named')}
          >
            Named Only
          </button>
        </div>

        <div className="version-history__actions">
          <button
            className={compareMode ? 'active' : ''}
            onClick={() => {
              setCompareMode(!compareMode);
              setSelectedVersions([]);
            }}
          >
            {compareMode ? 'Exit Compare' : 'Compare'}
          </button>
          <button onClick={() => setShowCreateSnapshot(true)}>
            Create Snapshot
          </button>
        </div>
      </div>

      {compareMode && selectedVersions.length === 2 && (
        <div className="version-history__compare-bar">
          <span>Comparing 2 versions</span>
          <button onClick={handleCompare}>View Comparison</button>
        </div>
      )}

      {showCreateSnapshot && (
        <div className="version-history__snapshot-form">
          <input
            type="text"
            placeholder="Snapshot name (e.g., 'Final Draft')"
            value={snapshotName}
            onChange={(e) => setSnapshotName(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && handleCreateSnapshot()}
            autoFocus
          />
          <button onClick={handleCreateSnapshot}>Create</button>
          <button onClick={() => setShowCreateSnapshot(false)}>Cancel</button>
        </div>
      )}

      <div className="version-history__list">
        {isLoading ? (
          <div className="version-history__loading">Loading versions...</div>
        ) : filteredVersions.length === 0 ? (
          <div className="version-history__empty">No versions found</div>
        ) : (
          filteredVersions.map((version) => (
            <div
              key={version.id}
              className={`version-history__item ${
                version.isCurrent ? 'current' : ''
              } ${previewingId === version.id ? 'previewing' : ''} ${
                selectedVersions.includes(version.id) ? 'selected' : ''
              }`}
              onClick={() => handleVersionClick(version.id)}
            >
              <div className="version-history__item-header">
                {editingNameId === version.id ? (
                  <input
                    type="text"
                    value={newName}
                    onChange={(e) => setNewName(e.target.value)}
                    onKeyDown={(e) => {
                      if (e.key === 'Enter') handleRename(version.id);
                      if (e.key === 'Escape') setEditingNameId(null);
                    }}
                    onBlur={() => setEditingNameId(null)}
                    autoFocus
                    onClick={(e) => e.stopPropagation()}
                  />
                ) : (
                  <span className="version-history__item-name">
                    {version.name || formatDate(version.timestamp)}
                    {version.isNamed && (
                      <button
                        className="version-history__edit-name"
                        onClick={(e) => {
                          e.stopPropagation();
                          setEditingNameId(version.id);
                          setNewName(version.name || '');
                        }}
                      >
                        ✎
                      </button>
                    )}
                  </span>
                )}
                {version.isCurrent && (
                  <span className="version-history__current-badge">Current</span>
                )}
                {version.isNamed && !version.isCurrent && (
                  <span className="version-history__named-badge">Named</span>
                )}
              </div>

              <div className="version-history__item-meta">
                <span className="version-history__author">{version.author}</span>
                {!version.name && (
                  <span className="version-history__time">
                    {version.timestamp.toLocaleString()}
                  </span>
                )}
              </div>

              <div className="version-history__item-summary">{version.summary}</div>

              <div className="version-history__item-actions">
                {!version.isCurrent && (
                  <>
                    <button onClick={(e) => { e.stopPropagation(); handleRestore(version.id); }}>
                      Restore
                    </button>
                    {!version.isNamed && (
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          setEditingNameId(version.id);
                          setNewName('');
                        }}
                      >
                        Name this version
                      </button>
                    )}
                  </>
                )}
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
};
