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
                'tool_deployment_type_id', msif.tool_deployment_type_id,
                'tool_group_deployment_type_id', msif.tool_group_deployment_type_id,
                'tool_group_id', msif.tool_group_id,
                'tool_name', msif.tool_name,
                'tool_description', msif.tool_description,
                'created_at', strftime('%Y-%m-%dT%H:%M:%fZ', msif.created_at),
                'updated_at', strftime('%Y-%m-%dT%H:%M:%fZ', msif.updated_at)
            )
        )
        FROM mcp_server_instance_tool msif
        WHERE msif.mcp_server_instance_id = msi.id
        ), JSON('[]')) AS TEXT
    ) AS tools
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
                'tool_deployment_type_id', msif.tool_deployment_type_id,
                'tool_group_deployment_type_id', msif.tool_group_deployment_type_id,
                'tool_group_id', msif.tool_group_id,
                'tool_name', msif.tool_name,
                'tool_description', msif.tool_description,
                'created_at', strftime('%Y-%m-%dT%H:%M:%fZ', msif.created_at),
                'updated_at', strftime('%Y-%m-%dT%H:%M:%fZ', msif.updated_at)
            )
        )
        FROM mcp_server_instance_tool msif
        WHERE msif.mcp_server_instance_id = msi.id
        ), JSON('[]')) AS TEXT
    ) AS tools
FROM mcp_server_instance msi
WHERE (msi.created_at < sqlc.narg(cursor) OR sqlc.narg(cursor) IS NULL)
ORDER BY msi.created_at DESC
LIMIT CAST(sqlc.arg(page_size) AS INTEGER) + 1;

-- name: create_mcp_server_instance_tool :exec
INSERT INTO mcp_server_instance_tool (mcp_server_instance_id, tool_deployment_type_id, tool_group_deployment_type_id, tool_group_id, tool_name, tool_description, created_at, updated_at)
VALUES (?, ?, ?, ?, ?, ?, ?, ?);

-- name: update_mcp_server_instance_tool :exec
UPDATE mcp_server_instance_tool
SET tool_name = ?, tool_description = ?, updated_at = CURRENT_TIMESTAMP
WHERE mcp_server_instance_id = ?
  AND tool_deployment_type_id = ?
  AND tool_group_deployment_type_id = ?
  AND tool_group_id = ?;

-- name: get_mcp_server_instance_tool_by_name :one
SELECT
    mcp_server_instance_id,
    tool_deployment_type_id,
    tool_group_deployment_type_id,
    tool_group_id,
    tool_name,
    tool_description,
    created_at,
    updated_at
FROM mcp_server_instance_tool
WHERE mcp_server_instance_id = ?
  AND tool_name = ?;

-- name: delete_mcp_server_instance_tool :exec
DELETE FROM mcp_server_instance_tool
WHERE mcp_server_instance_id = ?
  AND tool_deployment_type_id = ?
  AND tool_group_deployment_type_id = ?
  AND tool_group_id = ?;

-- name: delete_all_mcp_server_instance_tools :exec
DELETE FROM mcp_server_instance_tool WHERE mcp_server_instance_id = ?;

-- name: list_mcp_server_instance_tools :many
SELECT
    mcp_server_instance_id,
    tool_deployment_type_id,
    tool_group_deployment_type_id,
    tool_group_id,
    tool_name,
    tool_description,
    created_at,
    updated_at
FROM mcp_server_instance_tool
WHERE mcp_server_instance_id = ?
  AND (created_at < sqlc.narg(cursor) OR sqlc.narg(cursor) IS NULL)
ORDER BY created_at DESC
LIMIT CAST(sqlc.arg(page_size) AS INTEGER) + 1;
