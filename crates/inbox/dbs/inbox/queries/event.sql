-- name: insert_event :exec
INSERT INTO event (
    id,
    kind,
    payload,
    inbox_id,
    inbox_settings,
    created_at
) VALUES (
    :id,
    :kind,
    :payload,
    :inbox_id,
    :inbox_settings,
    :created_at
);

-- name: get_event_by_id :one
SELECT id, kind, payload, inbox_id, inbox_settings, created_at
FROM event WHERE id = :id;

-- name: get_events :many
SELECT id, kind, payload, inbox_id, inbox_settings, created_at
FROM event
WHERE (created_at < sqlc.narg(cursor) OR sqlc.narg(cursor) IS NULL)
ORDER BY created_at DESC
LIMIT CAST(sqlc.arg(page_size) AS INTEGER) + 1;

-- name: get_events_by_inbox :many
SELECT id, kind, payload, inbox_id, inbox_settings, created_at
FROM event
WHERE inbox_id = :inbox_id
  AND (created_at < sqlc.narg(cursor) OR sqlc.narg(cursor) IS NULL)
ORDER BY created_at DESC
LIMIT CAST(sqlc.arg(page_size) AS INTEGER) + 1;

-- name: get_events_by_kind :many
SELECT id, kind, payload, inbox_id, inbox_settings, created_at
FROM event
WHERE kind = :kind
  AND (created_at < sqlc.narg(cursor) OR sqlc.narg(cursor) IS NULL)
ORDER BY created_at DESC
LIMIT CAST(sqlc.arg(page_size) AS INTEGER) + 1;

-- name: delete_events_before :exec
DELETE FROM event WHERE created_at < :before_date;
