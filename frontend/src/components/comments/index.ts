/**
 * Comments UI Components
 *
 * This module exports all components related to the document comments system.
 */

// Main panel component
export { CommentsPanel } from '../CommentsPanel';
export type { default as CommentsPanelDefault } from '../CommentsPanel';

// Inline bubble marker
export {
  CommentBubble,
  CommentBubblesContainer,
  groupCommentsByPosition,
} from '../CommentBubble';

// Text highlighting
export {
  CommentHighlight,
  CommentHighlightRange,
  CommentHighlightsContainer,
  useCommentHighlights,
} from '../CommentHighlight';

// Add comment dialog
export {
  AddCommentDialog,
  AddCommentPopup,
  QuickCommentInput,
} from '../AddCommentDialog';

// Hook and types
export {
  useComments,
  formatCommentDate,
  getInitials,
  stringToColor,
  truncateText,
} from '../../hooks/useComments';

export type {
  Comment,
  CommentReply,
  CommentRange,
  CommentAuthor,
  CommentFilter,
  CommentSort,
  SortDirection,
  TextSelection,
  UseCommentsOptions,
  UseCommentsReturn,
} from '../../hooks/useComments';
