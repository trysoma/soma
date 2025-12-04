-- ============================================================================
-- User table queries
-- ============================================================================

-- name: create_user :exec
INSERT INTO user (id, type, email, role, description, created_at, updated_at)
VALUES (sqlc.arg(id), sqlc.arg(user_type), sqlc.arg(email), sqlc.arg(role), sqlc.arg(description), sqlc.arg(created_at), sqlc.arg(updated_at));

-- name: get_user_by_id :one
SELECT id, type as user_type, email, role, description, created_at, updated_at
FROM user
WHERE id = ?;

-- name: update_user :exec
UPDATE user
SET email = ?,
    role = ?,
    description = ?,
    updated_at = CURRENT_TIMESTAMP
WHERE id = ?;

-- name: delete_user :exec
DELETE FROM user WHERE id = ?;

-- name: get_users :many
SELECT id, type as user_type, email, role, description, created_at, updated_at
FROM user
WHERE (created_at < sqlc.narg(cursor) OR sqlc.narg(cursor) IS NULL)
  AND (type = sqlc.narg(user_type) OR sqlc.narg(user_type) IS NULL)
  AND (role = sqlc.narg(role) OR sqlc.narg(role) IS NULL)
ORDER BY created_at DESC
LIMIT CAST(sqlc.arg(page_size) AS INTEGER) + 1;
