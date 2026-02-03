import React, { useState, useCallback } from 'react';
import './ShareDialog.css';

interface ShareDialogProps {
  isOpen: boolean;
  onClose: () => void;
  documentId: string;
  documentTitle: string;
  currentUserEmail: string;
  collaborators: Collaborator[];
  onAddCollaborator: (email: string, permission: PermissionLevel) => Promise<void>;
  onRemoveCollaborator: (userId: string) => Promise<void>;
  onChangePermission: (userId: string, permission: PermissionLevel) => Promise<void>;
  onCreateLink: (permission: PermissionLevel, options: LinkOptions) => Promise<ShareLink>;
  onRevokeLink: (linkId: string) => Promise<void>;
  shareLinks: ShareLink[];
}

type PermissionLevel = 'viewer' | 'commenter' | 'editor' | 'owner';

interface Collaborator {
  userId: string;
  email: string;
  displayName: string;
  avatar?: string;
  permission: PermissionLevel;
  status: 'accepted' | 'pending';
}

interface ShareLink {
  id: string;
  url: string;
  permission: PermissionLevel;
  createdAt: Date;
  expiresAt?: Date;
  requiresPassword: boolean;
}

interface LinkOptions {
  expiresInDays?: number;
  password?: string;
}

export const ShareDialog: React.FC<ShareDialogProps> = ({
  isOpen,
  onClose,
  documentId,
  documentTitle,
  currentUserEmail,
  collaborators,
  onAddCollaborator,
  onRemoveCollaborator,
  onChangePermission,
  onCreateLink,
  onRevokeLink,
  shareLinks,
}) => {
  const [email, setEmail] = useState('');
  const [permission, setPermission] = useState<PermissionLevel>('editor');
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<'people' | 'link'>('people');
  const [linkPermission, setLinkPermission] = useState<PermissionLevel>('viewer');
  const [linkExpires, setLinkExpires] = useState(false);
  const [linkExpiryDays, setLinkExpiryDays] = useState(7);
  const [linkPassword, setLinkPassword] = useState('');
  const [usePassword, setUsePassword] = useState(false);
  const [copiedLinkId, setCopiedLinkId] = useState<string | null>(null);

  const handleAddCollaborator = useCallback(async () => {
    if (!email.trim()) return;

    setIsLoading(true);
    setError(null);

    try {
      await onAddCollaborator(email.trim(), permission);
      setEmail('');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to add collaborator');
    } finally {
      setIsLoading(false);
    }
  }, [email, permission, onAddCollaborator]);

  const handleCreateLink = useCallback(async () => {
    setIsLoading(true);
    setError(null);

    try {
      const options: LinkOptions = {};
      if (linkExpires) options.expiresInDays = linkExpiryDays;
      if (usePassword && linkPassword) options.password = linkPassword;

      await onCreateLink(linkPermission, options);
      setLinkPassword('');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create link');
    } finally {
      setIsLoading(false);
    }
  }, [linkExpires, linkExpiryDays, usePassword, linkPassword, linkPermission, onCreateLink]);

  const handleCopyLink = useCallback((link: ShareLink) => {
    navigator.clipboard.writeText(link.url);
    setCopiedLinkId(link.id);
    setTimeout(() => setCopiedLinkId(null), 2000);
  }, []);

  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (e.key === 'Escape') {
      onClose();
    }
  }, [onClose]);

  const handleRemoveCollaborator = useCallback(async (userId: string) => {
    setIsLoading(true);
    setError(null);
    try {
      await onRemoveCollaborator(userId);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to remove collaborator');
    } finally {
      setIsLoading(false);
    }
  }, [onRemoveCollaborator]);

  const handleChangePermission = useCallback(async (userId: string, newPermission: PermissionLevel) => {
    setIsLoading(true);
    setError(null);
    try {
      await onChangePermission(userId, newPermission);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to change permission');
    } finally {
      setIsLoading(false);
    }
  }, [onChangePermission]);

  const handleRevokeLink = useCallback(async (linkId: string) => {
    setIsLoading(true);
    setError(null);
    try {
      await onRevokeLink(linkId);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to revoke link');
    } finally {
      setIsLoading(false);
    }
  }, [onRevokeLink]);

  if (!isOpen) return null;

  return (
    <div
      className="share-dialog-overlay"
      onClick={onClose}
      onKeyDown={handleKeyDown}
      role="dialog"
      aria-modal="true"
      aria-labelledby="share-dialog-title"
    >
      <div className="share-dialog" onClick={(e) => e.stopPropagation()}>
        <div className="share-dialog__header">
          <h2 id="share-dialog-title">Share "{documentTitle}"</h2>
          <button
            className="share-dialog__close"
            onClick={onClose}
            aria-label="Close dialog"
          >
            <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
              <path d="M4.646 4.646a.5.5 0 0 1 .708 0L8 7.293l2.646-2.647a.5.5 0 0 1 .708.708L8.707 8l2.647 2.646a.5.5 0 0 1-.708.708L8 8.707l-2.646 2.647a.5.5 0 0 1-.708-.708L7.293 8 4.646 5.354a.5.5 0 0 1 0-.708z"/>
            </svg>
          </button>
        </div>

        <div className="share-dialog__tabs" role="tablist">
          <button
            className={`share-dialog__tab ${activeTab === 'people' ? 'active' : ''}`}
            onClick={() => setActiveTab('people')}
            role="tab"
            aria-selected={activeTab === 'people'}
            aria-controls="people-panel"
          >
            <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
              <path d="M7 14s-1 0-1-1 1-4 5-4 5 3 5 4-1 1-1 1H7zm4-6a3 3 0 1 0 0-6 3 3 0 0 0 0 6z"/>
              <path fillRule="evenodd" d="M5.216 14A2.238 2.238 0 0 1 5 13c0-1.355.68-2.75 1.936-3.72A6.325 6.325 0 0 0 5 9c-4 0-5 3-5 4s1 1 1 1h4.216z"/>
              <path d="M4.5 8a2.5 2.5 0 1 0 0-5 2.5 2.5 0 0 0 0 5z"/>
            </svg>
            People
          </button>
          <button
            className={`share-dialog__tab ${activeTab === 'link' ? 'active' : ''}`}
            onClick={() => setActiveTab('link')}
            role="tab"
            aria-selected={activeTab === 'link'}
            aria-controls="link-panel"
          >
            <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
              <path d="M4.715 6.542L3.343 7.914a3 3 0 1 0 4.243 4.243l1.828-1.829A3 3 0 0 0 8.586 5.5L8 6.086a1.001 1.001 0 0 0-.154.199 2 2 0 0 1 .861 3.337L6.88 11.45a2 2 0 1 1-2.83-2.83l.793-.792a4.018 4.018 0 0 1-.128-1.287z"/>
              <path d="M6.586 4.672A3 3 0 0 0 7.414 9.5l.775-.776a2 2 0 0 1-.896-3.346L9.12 3.55a2 2 0 0 1 2.83 2.83l-.793.792c.112.42.155.855.128 1.287l1.372-1.372a3 3 0 0 0-4.243-4.243L6.586 4.672z"/>
            </svg>
            Get Link
          </button>
        </div>

        {error && (
          <div className="share-dialog__error" role="alert">
            <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
              <path d="M8.982 1.566a1.13 1.13 0 0 0-1.96 0L.165 13.233c-.457.778.091 1.767.98 1.767h13.713c.889 0 1.438-.99.98-1.767L8.982 1.566zM8 5c.535 0 .954.462.9.995l-.35 3.507a.552.552 0 0 1-1.1 0L7.1 5.995A.905.905 0 0 1 8 5zm.002 6a1 1 0 1 1 0 2 1 1 0 0 1 0-2z"/>
            </svg>
            {error}
          </div>
        )}

        {activeTab === 'people' && (
          <div className="share-dialog__people" id="people-panel" role="tabpanel">
            {/* Add person form */}
            <div className="share-dialog__add-form">
              <div className="share-dialog__input-wrapper">
                <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor" className="share-dialog__input-icon">
                  <path d="M0 4a2 2 0 0 1 2-2h12a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H2a2 2 0 0 1-2-2V4zm2-1a1 1 0 0 0-1 1v.217l7 4.2 7-4.2V4a1 1 0 0 0-1-1H2zm13 2.383l-4.758 2.855L15 11.114v-5.73zm-.034 6.878L9.271 8.82 8 9.583 6.728 8.82l-5.694 3.44A1 1 0 0 0 2 13h12a1 1 0 0 0 .966-.739zM1 11.114l4.758-2.876L1 5.383v5.73z"/>
                </svg>
                <input
                  type="email"
                  placeholder="Add people by email"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  onKeyDown={(e) => e.key === 'Enter' && handleAddCollaborator()}
                  aria-label="Email address"
                  disabled={isLoading}
                />
              </div>
              <select
                value={permission}
                onChange={(e) => setPermission(e.target.value as PermissionLevel)}
                aria-label="Permission level"
                disabled={isLoading}
              >
                <option value="viewer">Viewer</option>
                <option value="commenter">Commenter</option>
                <option value="editor">Editor</option>
              </select>
              <button
                onClick={handleAddCollaborator}
                disabled={isLoading || !email.trim()}
                className="share-dialog__share-btn"
              >
                {isLoading ? (
                  <span className="share-dialog__spinner" />
                ) : (
                  'Share'
                )}
              </button>
            </div>

            {/* Collaborator list */}
            <div className="share-dialog__collaborators">
              <h3>People with access</h3>
              <div className="share-dialog__collaborator-list">
                {collaborators.map((collab) => (
                  <div key={collab.userId} className="share-dialog__collaborator">
                    <div className="share-dialog__collaborator-avatar">
                      {collab.avatar ? (
                        <img src={collab.avatar} alt={collab.displayName} />
                      ) : (
                        <span className="share-dialog__collaborator-initials">
                          {collab.displayName.charAt(0).toUpperCase()}
                        </span>
                      )}
                      {collab.status === 'pending' && (
                        <span className="share-dialog__pending-badge" title="Invitation pending">
                          <svg width="10" height="10" viewBox="0 0 16 16" fill="currentColor">
                            <path d="M8 15A7 7 0 1 1 8 1a7 7 0 0 1 0 14zm0 1A8 8 0 1 0 8 0a8 8 0 0 0 0 16z"/>
                            <path d="M8 4a.5.5 0 0 1 .5.5v3h3a.5.5 0 0 1 0 1h-3v3a.5.5 0 0 1-1 0v-3h-3a.5.5 0 0 1 0-1h3v-3A.5.5 0 0 1 8 4z"/>
                          </svg>
                        </span>
                      )}
                    </div>
                    <div className="share-dialog__collaborator-info">
                      <div className="share-dialog__collaborator-name">
                        {collab.displayName}
                        {collab.email === currentUserEmail && <span className="share-dialog__you-badge">(you)</span>}
                        {collab.status === 'pending' && <span className="share-dialog__status-badge">Pending</span>}
                      </div>
                      <div className="share-dialog__collaborator-email">{collab.email}</div>
                    </div>
                    <div className="share-dialog__collaborator-actions">
                      {collab.permission === 'owner' ? (
                        <span className="share-dialog__collaborator-owner">Owner</span>
                      ) : collab.email !== currentUserEmail ? (
                        <>
                          <select
                            value={collab.permission}
                            onChange={(e) =>
                              handleChangePermission(collab.userId, e.target.value as PermissionLevel)
                            }
                            aria-label={`Permission for ${collab.displayName}`}
                            disabled={isLoading}
                          >
                            <option value="viewer">Viewer</option>
                            <option value="commenter">Commenter</option>
                            <option value="editor">Editor</option>
                          </select>
                          <button
                            className="share-dialog__remove"
                            onClick={() => handleRemoveCollaborator(collab.userId)}
                            title="Remove access"
                            aria-label={`Remove ${collab.displayName}`}
                            disabled={isLoading}
                          >
                            <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
                              <path d="M4.646 4.646a.5.5 0 0 1 .708 0L8 7.293l2.646-2.647a.5.5 0 0 1 .708.708L8.707 8l2.647 2.646a.5.5 0 0 1-.708.708L8 8.707l-2.646 2.647a.5.5 0 0 1-.708-.708L7.293 8 4.646 5.354a.5.5 0 0 1 0-.708z"/>
                            </svg>
                          </button>
                        </>
                      ) : (
                        <span className="share-dialog__collaborator-permission">{collab.permission}</span>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            </div>
          </div>
        )}

        {activeTab === 'link' && (
          <div className="share-dialog__link" id="link-panel" role="tabpanel">
            {/* Create link form */}
            <div className="share-dialog__link-form">
              <div className="share-dialog__link-row">
                <label htmlFor="link-permission">Anyone with the link can:</label>
                <select
                  id="link-permission"
                  value={linkPermission}
                  onChange={(e) => setLinkPermission(e.target.value as PermissionLevel)}
                  disabled={isLoading}
                >
                  <option value="viewer">View</option>
                  <option value="commenter">Comment</option>
                  <option value="editor">Edit</option>
                </select>
              </div>

              <div className="share-dialog__link-row">
                <label className="share-dialog__checkbox-label">
                  <input
                    type="checkbox"
                    checked={linkExpires}
                    onChange={(e) => setLinkExpires(e.target.checked)}
                    disabled={isLoading}
                  />
                  <span className="share-dialog__checkbox-text">Link expires</span>
                </label>
                {linkExpires && (
                  <select
                    value={linkExpiryDays}
                    onChange={(e) => setLinkExpiryDays(Number(e.target.value))}
                    aria-label="Expiry duration"
                    disabled={isLoading}
                  >
                    <option value={1}>1 day</option>
                    <option value={7}>7 days</option>
                    <option value={30}>30 days</option>
                    <option value={90}>90 days</option>
                  </select>
                )}
              </div>

              <div className="share-dialog__link-row">
                <label className="share-dialog__checkbox-label">
                  <input
                    type="checkbox"
                    checked={usePassword}
                    onChange={(e) => setUsePassword(e.target.checked)}
                    disabled={isLoading}
                  />
                  <span className="share-dialog__checkbox-text">Require password</span>
                </label>
                {usePassword && (
                  <input
                    type="password"
                    placeholder="Enter password"
                    value={linkPassword}
                    onChange={(e) => setLinkPassword(e.target.value)}
                    className="share-dialog__password-input"
                    disabled={isLoading}
                  />
                )}
              </div>

              <button
                className="share-dialog__create-link"
                onClick={handleCreateLink}
                disabled={isLoading || (usePassword && !linkPassword)}
              >
                {isLoading ? (
                  <span className="share-dialog__spinner" />
                ) : (
                  <>
                    <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
                      <path d="M4.715 6.542L3.343 7.914a3 3 0 1 0 4.243 4.243l1.828-1.829A3 3 0 0 0 8.586 5.5L8 6.086a1.001 1.001 0 0 0-.154.199 2 2 0 0 1 .861 3.337L6.88 11.45a2 2 0 1 1-2.83-2.83l.793-.792a4.018 4.018 0 0 1-.128-1.287z"/>
                      <path d="M6.586 4.672A3 3 0 0 0 7.414 9.5l.775-.776a2 2 0 0 1-.896-3.346L9.12 3.55a2 2 0 0 1 2.83 2.83l-.793.792c.112.42.155.855.128 1.287l1.372-1.372a3 3 0 0 0-4.243-4.243L6.586 4.672z"/>
                    </svg>
                    Create Link
                  </>
                )}
              </button>
            </div>

            {/* Existing links */}
            {shareLinks.length > 0 && (
              <div className="share-dialog__links">
                <h3>Active links</h3>
                <div className="share-dialog__links-list">
                  {shareLinks.map((link) => (
                    <div key={link.id} className="share-dialog__link-item">
                      <div className="share-dialog__link-icon">
                        <svg width="20" height="20" viewBox="0 0 16 16" fill="currentColor">
                          <path d="M4.715 6.542L3.343 7.914a3 3 0 1 0 4.243 4.243l1.828-1.829A3 3 0 0 0 8.586 5.5L8 6.086a1.001 1.001 0 0 0-.154.199 2 2 0 0 1 .861 3.337L6.88 11.45a2 2 0 1 1-2.83-2.83l.793-.792a4.018 4.018 0 0 1-.128-1.287z"/>
                          <path d="M6.586 4.672A3 3 0 0 0 7.414 9.5l.775-.776a2 2 0 0 1-.896-3.346L9.12 3.55a2 2 0 0 1 2.83 2.83l-.793.792c.112.42.155.855.128 1.287l1.372-1.372a3 3 0 0 0-4.243-4.243L6.586 4.672z"/>
                        </svg>
                      </div>
                      <div className="share-dialog__link-info">
                        <span className="share-dialog__link-permission">
                          Can {link.permission === 'viewer' ? 'view' : link.permission === 'commenter' ? 'comment' : 'edit'}
                        </span>
                        <div className="share-dialog__link-meta">
                          {link.expiresAt && (
                            <span className="share-dialog__link-expires">
                              <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor">
                                <path d="M8 3.5a.5.5 0 0 0-1 0V9a.5.5 0 0 0 .252.434l3.5 2a.5.5 0 0 0 .496-.868L8 8.71V3.5z"/>
                                <path d="M8 16A8 8 0 1 0 8 0a8 8 0 0 0 0 16zm7-8A7 7 0 1 1 1 8a7 7 0 0 1 14 0z"/>
                              </svg>
                              Expires {new Date(link.expiresAt).toLocaleDateString()}
                            </span>
                          )}
                          {link.requiresPassword && (
                            <span className="share-dialog__link-password">
                              <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor">
                                <path d="M8 1a2 2 0 0 1 2 2v4H6V3a2 2 0 0 1 2-2zm3 6V3a3 3 0 0 0-6 0v4a2 2 0 0 0-2 2v5a2 2 0 0 0 2 2h6a2 2 0 0 0 2-2V9a2 2 0 0 0-2-2z"/>
                              </svg>
                              Password protected
                            </span>
                          )}
                        </div>
                      </div>
                      <div className="share-dialog__link-actions">
                        <button
                          onClick={() => handleCopyLink(link)}
                          className="share-dialog__copy-btn"
                          disabled={isLoading}
                        >
                          {copiedLinkId === link.id ? (
                            <>
                              <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
                                <path d="M12.736 3.97a.733.733 0 0 1 1.047 0c.286.289.29.756.01 1.05L7.88 12.01a.733.733 0 0 1-1.065.02L3.217 8.384a.757.757 0 0 1 0-1.06.733.733 0 0 1 1.047 0l3.052 3.093 5.4-6.425a.247.247 0 0 1 .02-.022z"/>
                              </svg>
                              Copied!
                            </>
                          ) : (
                            <>
                              <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
                                <path d="M4 1.5H3a2 2 0 0 0-2 2V14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V3.5a2 2 0 0 0-2-2h-1v1h1a1 1 0 0 1 1 1V14a1 1 0 0 1-1 1H3a1 1 0 0 1-1-1V3.5a1 1 0 0 1 1-1h1v-1z"/>
                                <path d="M9.5 1a.5.5 0 0 1 .5.5v1a.5.5 0 0 1-.5.5h-3a.5.5 0 0 1-.5-.5v-1a.5.5 0 0 1 .5-.5h3zm-3-1A1.5 1.5 0 0 0 5 1.5v1A1.5 1.5 0 0 0 6.5 4h3A1.5 1.5 0 0 0 11 2.5v-1A1.5 1.5 0 0 0 9.5 0h-3z"/>
                              </svg>
                              Copy
                            </>
                          )}
                        </button>
                        <button
                          className="share-dialog__revoke"
                          onClick={() => handleRevokeLink(link.id)}
                          title="Revoke link"
                          disabled={isLoading}
                        >
                          <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
                            <path d="M5.5 5.5A.5.5 0 0 1 6 6v6a.5.5 0 0 1-1 0V6a.5.5 0 0 1 .5-.5zm2.5 0a.5.5 0 0 1 .5.5v6a.5.5 0 0 1-1 0V6a.5.5 0 0 1 .5-.5zm3 .5a.5.5 0 0 0-1 0v6a.5.5 0 0 0 1 0V6z"/>
                            <path fillRule="evenodd" d="M14.5 3a1 1 0 0 1-1 1H13v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V4h-.5a1 1 0 0 1-1-1V2a1 1 0 0 1 1-1H6a1 1 0 0 1 1-1h2a1 1 0 0 1 1 1h3.5a1 1 0 0 1 1 1v1zM4.118 4L4 4.059V13a1 1 0 0 0 1 1h6a1 1 0 0 0 1-1V4.059L11.882 4H4.118zM2.5 3V2h11v1h-11z"/>
                          </svg>
                          Revoke
                        </button>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {shareLinks.length === 0 && (
              <div className="share-dialog__no-links">
                <svg width="48" height="48" viewBox="0 0 16 16" fill="currentColor">
                  <path d="M4.715 6.542L3.343 7.914a3 3 0 1 0 4.243 4.243l1.828-1.829A3 3 0 0 0 8.586 5.5L8 6.086a1.001 1.001 0 0 0-.154.199 2 2 0 0 1 .861 3.337L6.88 11.45a2 2 0 1 1-2.83-2.83l.793-.792a4.018 4.018 0 0 1-.128-1.287z"/>
                  <path d="M6.586 4.672A3 3 0 0 0 7.414 9.5l.775-.776a2 2 0 0 1-.896-3.346L9.12 3.55a2 2 0 0 1 2.83 2.83l-.793.792c.112.42.155.855.128 1.287l1.372-1.372a3 3 0 0 0-4.243-4.243L6.586 4.672z"/>
                </svg>
                <p>No shareable links yet</p>
                <span>Create a link to share this document with anyone</span>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
};

export type { ShareDialogProps, Collaborator, ShareLink, LinkOptions, PermissionLevel };
