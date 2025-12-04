-- ============================================================================
-- STS configuration table queries
-- ============================================================================

-- name: create_sts_configuration :exec
INSERT INTO sts_configuration (id, type, value, created_at, updated_at)
VALUES (sqlc.arg(id), sqlc.arg(config_type), sqlc.arg(value), sqlc.arg(created_at), sqlc.arg(updated_at));

-- name: get_sts_configuration_by_id :one
SELECT id, type as config_type, value, created_at, updated_at
FROM sts_configuration
WHERE id = ?;

-- name: delete_sts_configuration :exec
DELETE FROM sts_configuration WHERE id = ?;

-- name: get_sts_configurations :many
SELECT id, type as config_type, value, created_at, updated_at
FROM sts_configuration
WHERE (created_at < sqlc.narg(cursor) OR sqlc.narg(cursor) IS NULL)
  AND (type = sqlc.narg(config_type) OR sqlc.narg(config_type) IS NULL)
ORDER BY created_at DESC
LIMIT CAST(sqlc.arg(page_size) AS INTEGER) + 1;
