import React, { useMemo } from 'react';
import './CollaboratorList.css';

interface CollaboratorListProps {
  users: CollaboratorInfo[];
  currentUserId: string;
  compact?: boolean;
  maxVisible?: number;
  onUserClick?: (userId: string) => void;
}

interface CollaboratorInfo {
  userId: string;
  displayName: string;
  avatar?: string;
  color: string;
  isOnline: boolean;
  isTyping?: boolean;
  lastActiveAt?: Date;
}

export const CollaboratorList: React.FC<CollaboratorListProps> = ({
  users,
  currentUserId,
  compact = false,
  maxVisible = 5,
  onUserClick,
}) => {
  const otherUsers = useMemo(() =>
    users.filter((u) => u.userId !== currentUserId),
    [users, currentUserId]
  );

  const visibleUsers = useMemo(() =>
    otherUsers.slice(0, maxVisible),
    [otherUsers, maxVisible]
  );

  const hiddenCount = otherUsers.length - visibleUsers.length;

  const handleKeyDown = (userId: string) => (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      onUserClick?.(userId);
    }
  };

  if (otherUsers.length === 0) {
    return compact ? null : (
      <div className="collaborator-list collaborator-list--empty">
        <p>No other collaborators</p>
      </div>
    );
  }

  if (compact) {
    return (
      <div
        className="collaborator-list collaborator-list--compact"
        role="list"
        aria-label="Active collaborators"
      >
        {visibleUsers.map((user, index) => (
          <div
            key={user.userId}
            className={`collaborator-avatar ${onUserClick ? 'collaborator-avatar--clickable' : ''}`}
            style={{
              borderColor: user.color,
              zIndex: visibleUsers.length - index,
            }}
            title={`${user.displayName}${user.isTyping ? ' (typing...)' : ''}${!user.isOnline ? ' (offline)' : ''}`}
            onClick={() => onUserClick?.(user.userId)}
            onKeyDown={handleKeyDown(user.userId)}
            role="listitem"
            tabIndex={onUserClick ? 0 : undefined}
            aria-label={`${user.displayName}${user.isOnline ? ', online' : ', offline'}${user.isTyping ? ', typing' : ''}`}
          >
            {user.avatar ? (
              <img
                src={user.avatar}
                alt=""
                aria-hidden="true"
                loading="lazy"
              />
            ) : (
              <span
                className="collaborator-avatar__initials"
                style={{ backgroundColor: user.color }}
                aria-hidden="true"
              >
                {user.displayName.charAt(0).toUpperCase()}
              </span>
            )}
            {user.isOnline && (
              <span
                className="collaborator-status collaborator-status--online"
                aria-hidden="true"
              />
            )}
            {user.isTyping && (
              <span className="collaborator-typing" aria-hidden="true">
                <span className="collaborator-typing__dot" />
                <span className="collaborator-typing__dot" />
                <span className="collaborator-typing__dot" />
              </span>
            )}
          </div>
        ))}
        {hiddenCount > 0 && (
          <div
            className="collaborator-avatar collaborator-avatar--more"
            title={`${hiddenCount} more collaborator${hiddenCount > 1 ? 's' : ''}`}
            role="listitem"
            aria-label={`${hiddenCount} more collaborators`}
          >
            +{hiddenCount}
          </div>
        )}
      </div>
    );
  }

  return (
    <div className="collaborator-list" role="region" aria-label="Collaborators">
      <h4 className="collaborator-list__title">Collaborators</h4>
      <div className="collaborator-list__items" role="list">
        {visibleUsers.map((user) => (
          <div
            key={user.userId}
            className={`collaborator-item ${onUserClick ? 'collaborator-item--clickable' : ''}`}
            onClick={() => onUserClick?.(user.userId)}
            onKeyDown={handleKeyDown(user.userId)}
            role="listitem"
            tabIndex={onUserClick ? 0 : undefined}
            aria-label={`${user.displayName}${user.isOnline ? ', online' : ', offline'}${user.isTyping ? ', typing' : ''}`}
          >
            <div
              className="collaborator-item__avatar"
              style={{ borderColor: user.color }}
            >
              {user.avatar ? (
                <img
                  src={user.avatar}
                  alt=""
                  aria-hidden="true"
                  loading="lazy"
                />
              ) : (
                <span
                  className="collaborator-item__initials"
                  style={{ backgroundColor: user.color }}
                  aria-hidden="true"
                >
                  {user.displayName.charAt(0).toUpperCase()}
                </span>
              )}
              {user.isOnline && (
                <span
                  className="collaborator-status collaborator-status--online"
                  aria-hidden="true"
                />
              )}
            </div>
            <div className="collaborator-item__info">
              <span className="collaborator-item__name">{user.displayName}</span>
              {user.isTyping ? (
                <span className="collaborator-item__typing">typing...</span>
              ) : !user.isOnline && user.lastActiveAt ? (
                <span className="collaborator-item__last-active">
                  Last active {formatLastActive(user.lastActiveAt)}
                </span>
              ) : user.isOnline ? (
                <span className="collaborator-item__online">Online</span>
              ) : null}
            </div>
          </div>
        ))}
        {hiddenCount > 0 && (
          <div className="collaborator-item collaborator-item--more" role="listitem">
            <div className="collaborator-item__avatar collaborator-item__avatar--more">
              <span>+{hiddenCount}</span>
            </div>
            <div className="collaborator-item__info">
              <span className="collaborator-item__name">
                and {hiddenCount} more...
              </span>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

function formatLastActive(date: Date): string {
  const now = new Date();
  const diff = now.getTime() - date.getTime();
  const minutes = Math.floor(diff / 60000);
  const hours = Math.floor(diff / 3600000);
  const days = Math.floor(diff / 86400000);

  if (minutes < 1) return 'just now';
  if (minutes < 60) return `${minutes}m ago`;
  if (hours < 24) return `${hours}h ago`;
  if (days < 7) return `${days}d ago`;
  return date.toLocaleDateString();
}

export type { CollaboratorListProps, CollaboratorInfo };
