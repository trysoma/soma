-- name: insert_secret :exec
INSERT INTO secret (
    id,
    key,
    encrypted_secret,
    dek_alias,
    created_at,
    updated_at
) VALUES (
    :id,
    :key,
    :encrypted_secret,
    :dek_alias,
    :created_at,
    :updated_at
);

-- name: update_secret :exec
UPDATE secret SET
    encrypted_secret = :encrypted_secret,
    dek_alias = :dek_alias,
    updated_at = :updated_at
WHERE id = :id;

-- name: delete_secret :exec
DELETE FROM secret WHERE id = :id;

-- name: get_secret_by_id :one
SELECT * FROM secret WHERE id = :id;

-- name: get_secret_by_key :one
SELECT * FROM secret WHERE key = :key;

-- name: get_secrets :many
SELECT * FROM secret WHERE (created_at < sqlc.narg(cursor) OR sqlc.narg(cursor) IS NULL)
ORDER BY created_at DESC
LIMIT CAST(sqlc.arg(page_size) AS INTEGER) + 1;
