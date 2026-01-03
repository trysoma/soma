-- name: insert_variable :exec
INSERT INTO variable (
    id,
    key,
    value,
    created_at,
    updated_at
) VALUES (
    :id,
    :key,
    :value,
    :created_at,
    :updated_at
);

-- name: update_variable :exec
UPDATE variable SET
    value = :value,
    updated_at = :updated_at
WHERE id = :id;

-- name: delete_variable :exec
DELETE FROM variable WHERE id = :id;

-- name: get_variable_by_id :one
SELECT * FROM variable WHERE id = :id;

-- name: get_variable_by_key :one
SELECT * FROM variable WHERE key = :key;

-- name: get_variables :many
SELECT * FROM variable WHERE (created_at < sqlc.narg(cursor) OR sqlc.narg(cursor) IS NULL)
ORDER BY created_at DESC
LIMIT CAST(sqlc.arg(page_size) AS INTEGER) + 1;
