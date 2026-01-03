-- name: insert_thread :exec
INSERT INTO thread (
    id,
    title,
    metadata,
    inbox_settings,
    created_at,
    updated_at
) VALUES (
    :id,
    :title,
    :metadata,
    :inbox_settings,
    :created_at,
    :updated_at
);

-- name: update_thread :exec
UPDATE thread SET
    title = :title,
    metadata = :metadata,
    inbox_settings = :inbox_settings,
    updated_at = :updated_at
WHERE id = :id;

-- name: delete_thread :exec
DELETE FROM thread WHERE id = :id;

-- name: get_thread_by_id :one
SELECT id, title, metadata, inbox_settings, created_at, updated_at
FROM thread WHERE id = :id;

-- name: get_threads :many
SELECT id, title, metadata, inbox_settings, created_at, updated_at
FROM thread
WHERE (created_at < sqlc.narg(cursor) OR sqlc.narg(cursor) IS NULL)
ORDER BY created_at DESC
LIMIT CAST(sqlc.arg(page_size) AS INTEGER) + 1;
