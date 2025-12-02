-- ============================================================================
-- JWT signing key table queries
-- ============================================================================

-- name: create_jwt_signing_key :exec
INSERT INTO jwt_signing_key (kid, encrypted_private_key, expires_at, public_key, dek_alias, invalidated, created_at, updated_at)
VALUES (?, ?, ?, ?, ?, ?, ?, ?);

-- name: get_jwt_signing_key_by_kid :one
SELECT kid, encrypted_private_key, expires_at, public_key, dek_alias, invalidated, created_at, updated_at
FROM jwt_signing_key
WHERE kid = ?;

-- name: invalidate_jwt_signing_key :exec
UPDATE jwt_signing_key
SET invalidated = 1,
    updated_at = CURRENT_TIMESTAMP
WHERE kid = ?;

-- name: get_jwt_signing_keys :many
SELECT kid, encrypted_private_key, expires_at, public_key, dek_alias, invalidated, created_at, updated_at
FROM jwt_signing_key
WHERE invalidated = 0
  AND (created_at < sqlc.narg(cursor) OR sqlc.narg(cursor) IS NULL)
ORDER BY created_at DESC
LIMIT CAST(sqlc.arg(page_size) AS INTEGER) + 1;

