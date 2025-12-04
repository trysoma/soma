-- ============================================================================
-- User auth flow configuration table queries
-- ============================================================================

-- name: create_user_auth_flow_config :exec
INSERT INTO user_auth_flow_configuration (id, type, config, created_at, updated_at)
VALUES (sqlc.arg(id), sqlc.arg(config_type), sqlc.arg(config), sqlc.arg(created_at), sqlc.arg(updated_at));

-- name: get_user_auth_flow_config_by_id :one
SELECT id, type as config_type, config, created_at, updated_at
FROM user_auth_flow_configuration
WHERE id = ?;

-- name: delete_user_auth_flow_config :exec
DELETE FROM user_auth_flow_configuration WHERE id = ?;

-- name: get_user_auth_flow_configs :many
SELECT id, type as config_type, config, created_at, updated_at
FROM user_auth_flow_configuration
WHERE (created_at < sqlc.narg(cursor) OR sqlc.narg(cursor) IS NULL)
  AND (type = sqlc.narg(config_type) OR sqlc.narg(config_type) IS NULL)
ORDER BY created_at DESC
LIMIT CAST(sqlc.arg(page_size) AS INTEGER) + 1;
