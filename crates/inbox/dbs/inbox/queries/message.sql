-- name: insert_message :exec
INSERT INTO message (
    id,
    thread_id,
    kind,
    role,
    body,
    metadata,
    inbox_settings,
    created_at,
    updated_at
) VALUES (
    :id,
    :thread_id,
    :kind,
    :role,
    :body,
    :metadata,
    :inbox_settings,
    :created_at,
    :updated_at
);

-- name: update_message :exec
UPDATE message SET
    body = :body,
    metadata = :metadata,
    inbox_settings = :inbox_settings,
    updated_at = :updated_at
WHERE id = :id;

-- name: delete_message :exec
DELETE FROM message WHERE id = :id;

-- name: get_message_by_id :one
SELECT id, thread_id, kind, role, body, metadata, inbox_settings, created_at, updated_at
FROM message WHERE id = :id;

-- name: get_messages :many
SELECT id, thread_id, kind, role, body, metadata, inbox_settings, created_at, updated_at
FROM message
WHERE (created_at < sqlc.narg(cursor) OR sqlc.narg(cursor) IS NULL)
ORDER BY created_at DESC
LIMIT CAST(sqlc.arg(page_size) AS INTEGER) + 1;

-- name: get_messages_by_thread :many
SELECT id, thread_id, kind, role, body, metadata, inbox_settings, created_at, updated_at
FROM message
WHERE thread_id = :thread_id
  AND (created_at < sqlc.narg(cursor) OR sqlc.narg(cursor) IS NULL)
ORDER BY created_at ASC
LIMIT CAST(sqlc.arg(page_size) AS INTEGER) + 1;

-- name: delete_messages_by_thread :exec
DELETE FROM message WHERE thread_id = :thread_id;
