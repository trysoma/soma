-- name: insert_environment_variable :exec
INSERT INTO environment_variable (
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

-- name: update_environment_variable :exec
UPDATE environment_variable SET
    value = :value,
    updated_at = :updated_at
WHERE id = :id;

-- name: delete_environment_variable :exec
DELETE FROM environment_variable WHERE id = :id;

-- name: get_environment_variable_by_id :one
SELECT * FROM environment_variable WHERE id = :id;

-- name: get_environment_variable_by_key :one
SELECT * FROM environment_variable WHERE key = :key;

-- name: get_environment_variables :many
SELECT * FROM environment_variable WHERE (created_at < sqlc.narg(cursor) OR sqlc.narg(cursor) IS NULL)
ORDER BY created_at DESC
LIMIT CAST(sqlc.arg(page_size) AS INTEGER) + 1;
