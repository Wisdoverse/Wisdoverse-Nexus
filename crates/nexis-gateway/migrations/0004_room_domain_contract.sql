-- Align persisted room data with the room bounded-context contract.

ALTER TABLE rooms ADD COLUMN IF NOT EXISTS creator_id TEXT;
ALTER TABLE messages ADD COLUMN IF NOT EXISTS reply_to TEXT;

CREATE TABLE IF NOT EXISTS room_members (
    room_id TEXT NOT NULL REFERENCES rooms(id) ON DELETE CASCADE,
    member_id TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (room_id, member_id)
);

CREATE INDEX IF NOT EXISTS idx_rooms_creator_id ON rooms (creator_id);
CREATE INDEX IF NOT EXISTS idx_messages_sender_id ON messages (sender_id);
CREATE INDEX IF NOT EXISTS idx_room_members_member_id ON room_members (member_id);
