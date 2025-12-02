-- ============================================================================
-- User table queries
-- ============================================================================

-- name: create_user :exec
INSERT INTO user (id, type, email, role, created_at, updated_at)
VALUES (sqlc.arg(id), sqlc.arg(user_type), sqlc.arg(email), sqlc.arg(role), sqlc.arg(created_at), sqlc.arg(updated_at));

-- name: get_user_by_id :one
SELECT id, type as user_type, email, role, created_at, updated_at
FROM user
WHERE id = ?;

-- name: update_user :exec
UPDATE user
SET email = ?,
    role = ?,
    updated_at = CURRENT_TIMESTAMP
WHERE id = ?;

-- name: delete_user :exec
DELETE FROM user WHERE id = ?;

-- name: get_users :many
SELECT id, type as user_type, email, role, created_at, updated_at
FROM user
WHERE (created_at < sqlc.narg(cursor) OR sqlc.narg(cursor) IS NULL)
  AND (type = sqlc.narg(user_type) OR sqlc.narg(user_type) IS NULL)
  AND (role = sqlc.narg(role) OR sqlc.narg(role) IS NULL)
ORDER BY created_at DESC
LIMIT CAST(sqlc.arg(page_size) AS INTEGER) + 1;

-- ============================================================================
-- API key table queries
-- ============================================================================

-- name: create_api_key :exec
INSERT INTO api_key (hashed_value, user_id, created_at, updated_at)
VALUES (?, ?, ?, ?);

-- name: get_api_key_by_hashed_value :one
SELECT ak.hashed_value, ak.user_id, ak.created_at, ak.updated_at,
       u.id as user_id_fk, u.type as user_type, u.email as user_email, u.role as user_role,
       u.created_at as user_created_at, u.updated_at as user_updated_at
FROM api_key ak
JOIN user u ON ak.user_id = u.id
WHERE ak.hashed_value = ?;

-- name: delete_api_key :exec
DELETE FROM api_key WHERE hashed_value = ?;

-- name: get_api_keys :many
SELECT hashed_value, user_id, created_at, updated_at
FROM api_key
WHERE (created_at < sqlc.narg(cursor) OR sqlc.narg(cursor) IS NULL)
  AND (user_id = sqlc.narg(user_id) OR sqlc.narg(user_id) IS NULL)
ORDER BY created_at DESC
LIMIT CAST(sqlc.arg(page_size) AS INTEGER) + 1;

-- name: delete_api_keys_by_user_id :exec
DELETE FROM api_key WHERE user_id = ?;

-- ============================================================================
-- Group table queries
-- ============================================================================

-- name: create_group :exec
INSERT INTO `group` (id, name, created_at, updated_at)
VALUES (sqlc.arg(id), sqlc.arg(name), sqlc.arg(created_at), sqlc.arg(updated_at));

-- name: get_group_by_id :one
SELECT id, name, created_at, updated_at
FROM `group`
WHERE id = ?;

-- name: update_group :exec
UPDATE `group`
SET name = ?,
    updated_at = CURRENT_TIMESTAMP
WHERE id = ?;

-- name: delete_group :exec
DELETE FROM `group` WHERE id = ?;

-- name: get_groups :many
SELECT id, name, created_at, updated_at
FROM `group`
WHERE (created_at < sqlc.narg(cursor) OR sqlc.narg(cursor) IS NULL)
ORDER BY created_at DESC
LIMIT CAST(sqlc.arg(page_size) AS INTEGER) + 1;

-- ============================================================================
-- Group membership table queries
-- ============================================================================

-- name: create_group_membership :exec
INSERT INTO group_membership (group_id, user_id, created_at, updated_at)
VALUES (sqlc.arg(group_id), sqlc.arg(user_id), sqlc.arg(created_at), sqlc.arg(updated_at));

-- name: delete_group_membership :exec
DELETE FROM group_membership WHERE group_id = ? AND user_id = ?;

-- name: get_group_membership :one
SELECT group_id, user_id, created_at, updated_at
FROM group_membership
WHERE group_id = ? AND user_id = ?;

-- name: get_group_members :many
SELECT gm.group_id, gm.user_id, gm.created_at as membership_created_at, gm.updated_at as membership_updated_at,
       u.id as user_id_fk, u.type as user_type, u.email as user_email, u.role as user_role,
       u.created_at as user_created_at, u.updated_at as user_updated_at
FROM group_membership gm
JOIN user u ON gm.user_id = u.id
WHERE gm.group_id = sqlc.arg(group_id)
  AND (gm.created_at < sqlc.narg(cursor) OR sqlc.narg(cursor) IS NULL)
ORDER BY gm.created_at DESC
LIMIT CAST(sqlc.arg(page_size) AS INTEGER) + 1;

-- name: get_user_groups :many
SELECT gm.group_id, gm.user_id, gm.created_at as membership_created_at, gm.updated_at as membership_updated_at,
       g.id as group_id_fk, g.name as group_name, g.created_at as group_created_at, g.updated_at as group_updated_at
FROM group_membership gm
JOIN `group` g ON gm.group_id = g.id
WHERE gm.user_id = sqlc.arg(user_id)
  AND (gm.created_at < sqlc.narg(cursor) OR sqlc.narg(cursor) IS NULL)
ORDER BY gm.created_at DESC
LIMIT CAST(sqlc.arg(page_size) AS INTEGER) + 1;

-- name: delete_group_memberships_by_group_id :exec
DELETE FROM group_membership WHERE group_id = ?;

-- name: delete_group_memberships_by_user_id :exec
DELETE FROM group_membership WHERE user_id = ?;
