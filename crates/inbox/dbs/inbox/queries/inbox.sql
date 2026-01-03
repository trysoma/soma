-- name: insert_inbox :exec
INSERT INTO inbox (
    id,
    provider_id,
    status,
    configuration,
    settings,
    created_at,
    updated_at
) VALUES (
    :id,
    :provider_id,
    :status,
    :configuration,
    :settings,
    :created_at,
    :updated_at
);

-- name: update_inbox :exec
UPDATE inbox SET
    configuration = :configuration,
    settings = :settings,
    updated_at = :updated_at
WHERE id = :id;

-- name: update_inbox_status :exec
UPDATE inbox SET
    status = :status,
    updated_at = :updated_at
WHERE id = :id;

-- name: delete_inbox :exec
DELETE FROM inbox WHERE id = :id;

-- name: get_inbox_by_id :one
SELECT id, provider_id, status, configuration, settings, created_at, updated_at
FROM inbox WHERE id = :id;

-- name: get_inboxes :many
SELECT id, provider_id, status, configuration, settings, created_at, updated_at
FROM inbox
WHERE (created_at < sqlc.narg(cursor) OR sqlc.narg(cursor) IS NULL)
ORDER BY created_at DESC
LIMIT CAST(sqlc.arg(page_size) AS INTEGER) + 1;

-- name: get_inboxes_by_provider :many
SELECT id, provider_id, status, configuration, settings, created_at, updated_at
FROM inbox
WHERE provider_id = :provider_id
  AND (created_at < sqlc.narg(cursor) OR sqlc.narg(cursor) IS NULL)
ORDER BY created_at DESC
LIMIT CAST(sqlc.arg(page_size) AS INTEGER) + 1;

-- name: get_enabled_inboxes :many
SELECT id, provider_id, status, configuration, settings, created_at, updated_at
FROM inbox
WHERE status = 'enabled'
  AND (created_at < sqlc.narg(cursor) OR sqlc.narg(cursor) IS NULL)
ORDER BY created_at DESC
LIMIT CAST(sqlc.arg(page_size) AS INTEGER) + 1;
