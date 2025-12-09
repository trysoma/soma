-- name: create_mcp_server_instance :exec
INSERT INTO mcp_server_instance (id, name, created_at, updated_at)
VALUES (?, ?, ?, ?);

-- name: get_mcp_server_instance_by_id :one
SELECT
    msi.id,
    msi.name,
    msi.created_at,
    msi.updated_at,
    CAST(COALESCE(
        (SELECT JSON_GROUP_ARRAY(
            JSON_OBJECT(
                'mcp_server_instance_id', msif.mcp_server_instance_id,
                'function_controller_type_id', msif.function_controller_type_id,
                'provider_controller_type_id', msif.provider_controller_type_id,
                'provider_instance_id', msif.provider_instance_id,
                'function_name', msif.function_name,
                'function_description', msif.function_description,
                'created_at', strftime('%Y-%m-%dT%H:%M:%fZ', msif.created_at),
                'updated_at', strftime('%Y-%m-%dT%H:%M:%fZ', msif.updated_at)
            )
        )
        FROM mcp_server_instance_function msif
        WHERE msif.mcp_server_instance_id = msi.id
        ), JSON('[]')) AS TEXT
    ) AS functions
FROM mcp_server_instance msi
WHERE msi.id = ?;

-- name: update_mcp_server_instance :exec
UPDATE mcp_server_instance
SET name = ?, updated_at = CURRENT_TIMESTAMP
WHERE id = ?;

-- name: delete_mcp_server_instance :exec
DELETE FROM mcp_server_instance WHERE id = ?;

-- name: list_mcp_server_instances :many
SELECT
    msi.id,
    msi.name,
    msi.created_at,
    msi.updated_at,
    CAST(COALESCE(
        (SELECT JSON_GROUP_ARRAY(
            JSON_OBJECT(
                'mcp_server_instance_id', msif.mcp_server_instance_id,
                'function_controller_type_id', msif.function_controller_type_id,
                'provider_controller_type_id', msif.provider_controller_type_id,
                'provider_instance_id', msif.provider_instance_id,
                'function_name', msif.function_name,
                'function_description', msif.function_description,
                'created_at', strftime('%Y-%m-%dT%H:%M:%fZ', msif.created_at),
                'updated_at', strftime('%Y-%m-%dT%H:%M:%fZ', msif.updated_at)
            )
        )
        FROM mcp_server_instance_function msif
        WHERE msif.mcp_server_instance_id = msi.id
        ), JSON('[]')) AS TEXT
    ) AS functions
FROM mcp_server_instance msi
WHERE (msi.created_at < sqlc.narg(cursor) OR sqlc.narg(cursor) IS NULL)
ORDER BY msi.created_at DESC
LIMIT CAST(sqlc.arg(page_size) AS INTEGER) + 1;

-- name: create_mcp_server_instance_function :exec
INSERT INTO mcp_server_instance_function (mcp_server_instance_id, function_controller_type_id, provider_controller_type_id, provider_instance_id, function_name, function_description, created_at, updated_at)
VALUES (?, ?, ?, ?, ?, ?, ?, ?);

-- name: update_mcp_server_instance_function :exec
UPDATE mcp_server_instance_function
SET function_name = ?, function_description = ?, updated_at = CURRENT_TIMESTAMP
WHERE mcp_server_instance_id = ?
  AND function_controller_type_id = ?
  AND provider_controller_type_id = ?
  AND provider_instance_id = ?;

-- name: get_mcp_server_instance_function_by_name :one
SELECT
    mcp_server_instance_id,
    function_controller_type_id,
    provider_controller_type_id,
    provider_instance_id,
    function_name,
    function_description,
    created_at,
    updated_at
FROM mcp_server_instance_function
WHERE mcp_server_instance_id = ?
  AND function_name = ?;

-- name: delete_mcp_server_instance_function :exec
DELETE FROM mcp_server_instance_function
WHERE mcp_server_instance_id = ?
  AND function_controller_type_id = ?
  AND provider_controller_type_id = ?
  AND provider_instance_id = ?;

-- name: delete_all_mcp_server_instance_functions :exec
DELETE FROM mcp_server_instance_function WHERE mcp_server_instance_id = ?;

-- name: list_mcp_server_instance_functions :many
SELECT
    mcp_server_instance_id,
    function_controller_type_id,
    provider_controller_type_id,
    provider_instance_id,
    function_name,
    function_description,
    created_at,
    updated_at
FROM mcp_server_instance_function
WHERE mcp_server_instance_id = ?
  AND (created_at < sqlc.narg(cursor) OR sqlc.narg(cursor) IS NULL)
ORDER BY created_at DESC
LIMIT CAST(sqlc.arg(page_size) AS INTEGER) + 1;
