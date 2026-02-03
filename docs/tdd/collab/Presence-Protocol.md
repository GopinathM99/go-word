# Presence and Collaboration Protocol

## Presence State
- userId
- cursor position (nodeId, offset)
- selection range
- user color

## Transport
- WebSocket-based real-time channel.

## Events
- user_joined
- user_left
- cursor_moved
- selection_changed
- document_op (CRDT ops)

## Throttling
- Cursor updates at 30-60Hz max.
- Coalesce intermediate movements.

## Security
- Auth token per session.
- Enforce read/write permissions server-side.
