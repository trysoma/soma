-- Tool CRUD operations

-- name: create_tool_group_deployment :exec
INSERT INTO tool_group_deployment (type_id, deployment_id, name, documentation, categories, endpoint_type, endpoint_configuration, metadata, created_at, updated_at)
VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?);

-- name: get_tool_group_deployment_by_id :one
SELECT type_id, deployment_id, name, documentation, categories, endpoint_type, endpoint_configuration, metadata, created_at, updated_at
FROM tool_group_deployment
WHERE type_id = ? AND deployment_id = ?;

-- name: delete_tool_group_deployment :exec
DELETE FROM tool_group_deployment WHERE type_id = ? AND deployment_id = ?;

-- name: list_tool_group_deployments :many
SELECT type_id, deployment_id, name, documentation, categories, endpoint_type, endpoint_configuration, metadata, created_at, updated_at
FROM tool_group_deployment
WHERE (CAST(endpoint_type = sqlc.narg(endpoint_type) AS TEXT) OR sqlc.narg(endpoint_type) IS NULL)
  AND (created_at < sqlc.narg(cursor) OR sqlc.narg(cursor) IS NULL)
ORDER BY created_at DESC
LIMIT CAST(sqlc.arg(page_size) AS INTEGER) + 1;

-- name: list_tool_group_deployments_by_category :many
SELECT type_id, deployment_id, name, documentation, categories, endpoint_type, endpoint_configuration, metadata, created_at, updated_at
FROM tool_group_deployment
WHERE JSON_EXTRACT(categories, '$') LIKE '%' || sqlc.arg(category) || '%'
  AND (CAST(endpoint_type = sqlc.narg(endpoint_type) AS TEXT) OR sqlc.narg(endpoint_type) IS NULL)
  AND (created_at < sqlc.narg(cursor) OR sqlc.narg(cursor) IS NULL)
ORDER BY created_at DESC
LIMIT CAST(sqlc.arg(page_size) AS INTEGER) + 1;

-- Tool alias operations

-- name: create_tool_group_deployment_alias :exec
INSERT INTO tool_group_deployment_alias (tool_group_deployment_type_id, tool_group_deployment_deployment_id, alias, created_at, updated_at)
VALUES (?, ?, ?, ?, ?);

-- name: get_tool_group_deployment_by_alias :one
SELECT t.type_id, t.deployment_id, t.name, t.documentation, t.categories, t.endpoint_type, t.endpoint_configuration, t.metadata, t.created_at, t.updated_at
FROM tool_group_deployment t
INNER JOIN tool_group_deployment_alias ta ON t.type_id = ta.tool_group_deployment_type_id AND t.deployment_id = ta.tool_group_deployment_deployment_id
WHERE ta.alias = ?;

-- name: delete_tool_group_deployment_alias :exec
DELETE FROM tool_group_deployment_alias WHERE alias = ?;

-- name: list_tool_group_deployment_aliases :many
SELECT tool_group_deployment_type_id, tool_group_deployment_deployment_id, alias, created_at, updated_at
FROM tool_group_deployment_alias
WHERE (CAST(tool_group_deployment_type_id = sqlc.narg(tool_group_deployment_type_id) AS TEXT) OR sqlc.narg(tool_group_deployment_type_id) IS NULL)
  AND (CAST(tool_group_deployment_deployment_id = sqlc.narg(tool_group_deployment_deployment_id) AS TEXT) OR sqlc.narg(tool_group_deployment_deployment_id) IS NULL)
  AND (created_at < sqlc.narg(cursor) OR sqlc.narg(cursor) IS NULL)
ORDER BY created_at DESC
LIMIT CAST(sqlc.arg(page_size) AS INTEGER) + 1;

-- name: get_aliases_for_tool_group_deployment :many
SELECT alias
FROM tool_group_deployment_alias
WHERE tool_group_deployment_type_id = ? AND tool_group_deployment_deployment_id = ?
ORDER BY created_at DESC;

-- name: update_tool_group_deployment_alias :exec
UPDATE tool_group_deployment_alias
SET tool_group_deployment_deployment_id = ?, updated_at = CURRENT_TIMESTAMP
WHERE tool_group_deployment_type_id = ? AND alias = ?;
