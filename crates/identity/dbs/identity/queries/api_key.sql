-- ============================================================================
-- API key table queries
-- ============================================================================

-- name: create_api_key :exec
INSERT INTO api_key (id, hashed_value, description, user_id, created_at, updated_at)
VALUES (?, ?, ?, ?, ?, ?);

-- name: get_api_key_by_hashed_value :one
SELECT ak.id, ak.hashed_value, ak.description, ak.user_id, ak.created_at, ak.updated_at,
       u.id as user_id_fk, u.type as user_type, u.email as user_email, u.role as user_role,
       u.description as user_description, u.created_at as user_created_at, u.updated_at as user_updated_at
FROM api_key ak
JOIN user u ON ak.user_id = u.id
WHERE ak.hashed_value = ?;

-- name: get_api_key_by_id :one
SELECT ak.id, ak.hashed_value, ak.description, ak.user_id, ak.created_at, ak.updated_at,
       u.id as user_id_fk, u.type as user_type, u.email as user_email, u.role as user_role,
       u.description as user_description, u.created_at as user_created_at, u.updated_at as user_updated_at
FROM api_key ak
JOIN user u ON ak.user_id = u.id
WHERE ak.id = ?;

-- name: delete_api_key :exec
DELETE FROM api_key WHERE id = ?;

-- name: get_api_keys :many
SELECT id, hashed_value, description, user_id, created_at, updated_at
FROM api_key
WHERE (created_at < sqlc.narg(cursor) OR sqlc.narg(cursor) IS NULL)
  AND (user_id = sqlc.narg(user_id) OR sqlc.narg(user_id) IS NULL)
ORDER BY created_at DESC
LIMIT CAST(sqlc.arg(page_size) AS INTEGER) + 1;

-- name: delete_api_keys_by_user_id :exec
DELETE FROM api_key WHERE user_id = ?;
