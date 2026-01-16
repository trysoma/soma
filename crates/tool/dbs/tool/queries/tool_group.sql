-- name: create_resource_server_credential :exec
INSERT INTO resource_server_credential (id, type_id, metadata, value, created_at, updated_at, next_rotation_time, dek_alias)
VALUES (?, ?, ?, ?, ?, ?, ?, ?);

-- name: get_resource_server_credential_by_id :one
SELECT id, type_id, metadata, value, created_at, updated_at, next_rotation_time, dek_alias
FROM resource_server_credential
WHERE id = ?;

-- name: create_user_credential :exec
INSERT INTO user_credential (id, type_id, metadata, value, created_at, updated_at, next_rotation_time, dek_alias)
VALUES (?, ?, ?, ?, ?, ?, ?, ?);

-- name: get_user_credential_by_id :one
SELECT id, type_id, metadata, value, created_at, updated_at, next_rotation_time, dek_alias
FROM user_credential
WHERE id = ?;

-- name: delete_user_credential :exec
DELETE FROM user_credential WHERE id = ?;

-- name: delete_resource_server_credential :exec
DELETE FROM resource_server_credential WHERE id = ?;

-- name: get_user_credentials :many
SELECT id, type_id, metadata, value, created_at, updated_at, next_rotation_time, dek_alias
FROM user_credential WHERE (created_at < sqlc.narg(cursor) OR sqlc.narg(cursor) IS NULL)
ORDER BY created_at DESC
LIMIT CAST(sqlc.arg(page_size) AS INTEGER) + 1;

-- name: get_resource_server_credentials :many
SELECT id, type_id, metadata, value, created_at, updated_at, next_rotation_time, dek_alias
FROM resource_server_credential WHERE (created_at < sqlc.narg(cursor) OR sqlc.narg(cursor) IS NULL)
ORDER BY created_at DESC
LIMIT CAST(sqlc.arg(page_size) AS INTEGER) + 1;

-- name: create_tool_group :exec
INSERT INTO tool_group (id, display_name, resource_server_credential_id, user_credential_id, created_at, updated_at, tool_group_deployment_type_id, credential_deployment_type_id, status, return_on_successful_brokering)
VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?);

-- name: update_tool_group :exec
UPDATE tool_group SET display_name = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?;

-- name: update_tool_group_after_brokering :exec
UPDATE tool_group SET user_credential_id = ?, status = 'active', updated_at = CURRENT_TIMESTAMP WHERE id = ?;

-- name: get_tool_group_by_id :one
SELECT 
    pi.id,
    pi.display_name,
    pi.resource_server_credential_id,
    pi.user_credential_id,
    pi.created_at,
    pi.updated_at,
    pi.tool_group_deployment_type_id,
    pi.credential_deployment_type_id, pi.status, pi.return_on_successful_brokering,
    CAST(COALESCE(
        (SELECT JSON_GROUP_ARRAY(
            JSON_OBJECT(
                'tool_deployment_type_id', fi.tool_deployment_type_id,
                'tool_group_deployment_type_id', fi.tool_group_deployment_type_id,
                'tool_group_id', fi.tool_group_id,
                'created_at', strftime('%Y-%m-%dT%H:%M:%fZ', fi.created_at),
                'updated_at', strftime('%Y-%m-%dT%H:%M:%fZ', fi.updated_at)
            )
        )
        FROM tool fi
        WHERE fi.tool_group_id = pi.id
        ), JSON('[]')) AS TEXT
    ) AS functions,
    CAST(COALESCE(
        (SELECT JSON_OBJECT(
            'id', rsc.id,
            'type_id', rsc.type_id,
            'metadata', JSON(rsc.metadata),
            'value', JSON(rsc.value),
            'created_at', strftime('%Y-%m-%dT%H:%M:%fZ', rsc.created_at),
            'updated_at', strftime('%Y-%m-%dT%H:%M:%fZ', rsc.updated_at),
            'next_rotation_time', CASE WHEN rsc.next_rotation_time IS NOT NULL THEN strftime('%Y-%m-%dT%H:%M:%fZ', rsc.next_rotation_time) ELSE NULL END,
            'dek_alias', rsc.dek_alias
        )
        FROM resource_server_credential rsc
        WHERE rsc.id = pi.resource_server_credential_id
        ), JSON('null')) AS TEXT
    ) AS resource_server_credential,
    CAST(COALESCE(
        (SELECT JSON_OBJECT(
            'id', uc.id,
            'type_id', uc.type_id,
            'metadata', JSON(uc.metadata),
            'value', JSON(uc.value),
            'created_at', strftime('%Y-%m-%dT%H:%M:%fZ', uc.created_at),
            'updated_at', strftime('%Y-%m-%dT%H:%M:%fZ', uc.updated_at),
            'next_rotation_time', CASE WHEN uc.next_rotation_time IS NOT NULL THEN strftime('%Y-%m-%dT%H:%M:%fZ', uc.next_rotation_time) ELSE NULL END,
            'dek_alias', uc.dek_alias
        )
        FROM user_credential uc
        WHERE uc.id = pi.user_credential_id
        ), JSON('null')) AS TEXT
    ) AS user_credential
FROM tool_group pi
WHERE pi.id = ?;



-- name: delete_tool_group :exec
DELETE FROM tool_group WHERE id = ?;

-- name: create_tool :exec
INSERT INTO tool (tool_deployment_type_id, tool_group_deployment_type_id, tool_group_id, created_at, updated_at)
VALUES (?, ?, ?, ?, ?);

-- name: get_tool_by_id :one
SELECT tool_deployment_type_id, tool_group_deployment_type_id, tool_group_id, created_at, updated_at
FROM tool
WHERE tool_deployment_type_id = ? AND tool_group_deployment_type_id = ? AND tool_group_id = ?;

-- name: delete_tool :exec
DELETE FROM tool WHERE tool_deployment_type_id = ? AND tool_group_deployment_type_id = ? AND tool_group_id = ?;

-- name: create_broker_state :exec
INSERT INTO broker_state (id, created_at, updated_at, tool_group_id, tool_group_deployment_type_id, credential_deployment_type_id, metadata, action)
VALUES (?, ?, ?, ?, ?, ?, ?, ?);

-- name: get_broker_state_by_id :one
SELECT id, created_at, updated_at, tool_group_id, tool_group_deployment_type_id, credential_deployment_type_id, metadata, action
FROM broker_state
WHERE id = ?;

-- name: delete_broker_state :exec
DELETE FROM broker_state WHERE id = ?;

-- name: get_tool_with_credentials :one
SELECT
    fi.tool_deployment_type_id as tool_tool_deployment_type_id,
    fi.tool_group_deployment_type_id as tool_tool_group_deployment_type_id,
    fi.tool_group_id as tool_tool_group_id,
    fi.created_at as tool_created_at,
    fi.updated_at as tool_updated_at,
    pi.id as tool_group_id,
    pi.display_name as tool_group_display_name,
    pi.resource_server_credential_id as tool_group_resource_server_credential_id,
    pi.user_credential_id as tool_group_user_credential_id,
    pi.created_at as tool_group_created_at,
    pi.updated_at as tool_group_updated_at,
    pi.tool_group_deployment_type_id as tool_group_tool_group_deployment_type_id,
    pi.credential_deployment_type_id,
    pi.status as tool_group_status,
    pi.return_on_successful_brokering as tool_group_return_on_successful_brokering,
    rsc.id as resource_server_credential_id,
    rsc.type_id as resource_server_credential_type_id,
    rsc.metadata as resource_server_credential_metadata,
    rsc.value as resource_server_credential_value,
    rsc.created_at as resource_server_credential_created_at,
    rsc.updated_at as resource_server_credential_updated_at,
    rsc.next_rotation_time as resource_server_credential_next_rotation_time,
    rsc.dek_alias as resource_server_credential_dek_alias,
    uc.id as user_credential_id,
    uc.type_id as user_credential_type_id,
    uc.metadata as user_credential_metadata,
    uc.value as user_credential_value,
    uc.created_at as user_credential_created_at,
    uc.updated_at as user_credential_updated_at,
    uc.next_rotation_time as user_credential_next_rotation_time,
    uc.dek_alias as user_credential_dek_alias
FROM tool fi
JOIN tool_group pi ON fi.tool_group_id = pi.id
JOIN resource_server_credential rsc ON pi.resource_server_credential_id = rsc.id
LEFT JOIN user_credential uc ON pi.user_credential_id = uc.id
WHERE fi.tool_deployment_type_id = ? AND fi.tool_group_deployment_type_id = ? AND fi.tool_group_id = ?;

-- name: get_tool_groups :many
SELECT
    pi.id,
    pi.display_name,
    pi.resource_server_credential_id,
    pi.user_credential_id,
    pi.created_at,
    pi.updated_at,
    pi.tool_group_deployment_type_id,
    pi.credential_deployment_type_id,
    pi.status,
    pi.return_on_successful_brokering,
    CAST(COALESCE(
        (SELECT JSON_GROUP_ARRAY(
            JSON_OBJECT(
                'tool_deployment_type_id', fi.tool_deployment_type_id,
                'tool_group_deployment_type_id', fi.tool_group_deployment_type_id,
                'tool_group_id', fi.tool_group_id,
                'created_at', strftime('%Y-%m-%dT%H:%M:%fZ', fi.created_at),
                'updated_at', strftime('%Y-%m-%dT%H:%M:%fZ', fi.updated_at)
            )
        )
        FROM tool fi
        WHERE fi.tool_group_id = pi.id
        ), JSON('[]')) AS TEXT
    ) AS functions,
    CAST(COALESCE(
        (SELECT JSON_OBJECT(
            'id', rsc.id,
            'type_id', rsc.type_id,
            'metadata', JSON(rsc.metadata),
            'value', JSON(rsc.value),
            'created_at', strftime('%Y-%m-%dT%H:%M:%fZ', rsc.created_at),
            'updated_at', strftime('%Y-%m-%dT%H:%M:%fZ', rsc.updated_at),
            'next_rotation_time', CASE WHEN rsc.next_rotation_time IS NOT NULL THEN strftime('%Y-%m-%dT%H:%M:%fZ', rsc.next_rotation_time) ELSE NULL END,
            'dek_alias', rsc.dek_alias
        )
        FROM resource_server_credential rsc
        WHERE rsc.id = pi.resource_server_credential_id
        ), JSON('null')) AS TEXT
    ) AS resource_server_credential,
    CAST(COALESCE(
        (SELECT JSON_OBJECT(
            'id', uc.id,
            'type_id', uc.type_id,
            'metadata', JSON(uc.metadata),
            'value', JSON(uc.value),
            'created_at', strftime('%Y-%m-%dT%H:%M:%fZ', uc.created_at),
            'updated_at', strftime('%Y-%m-%dT%H:%M:%fZ', uc.updated_at),
            'next_rotation_time', CASE WHEN uc.next_rotation_time IS NOT NULL THEN strftime('%Y-%m-%dT%H:%M:%fZ', uc.next_rotation_time) ELSE NULL END,
            'dek_alias', uc.dek_alias
        )
        FROM user_credential uc
        WHERE uc.id = pi.user_credential_id
        ), JSON('null')) AS TEXT
    ) AS user_credential
FROM tool_group pi
WHERE (pi.created_at < sqlc.narg(cursor) OR sqlc.narg(cursor) IS NULL)
  AND (CAST(pi.status = sqlc.narg(status) AS TEXT) OR sqlc.narg(status) IS NULL)
  AND (CAST(pi.tool_group_deployment_type_id = sqlc.narg(tool_group_deployment_type_id) AS TEXT) OR sqlc.narg(tool_group_deployment_type_id) IS NULL)
ORDER BY pi.created_at DESC
LIMIT CAST(sqlc.arg(page_size) AS INTEGER) + 1;

-- name: get_tools :many
SELECT tool_deployment_type_id, tool_group_deployment_type_id, tool_group_id, created_at, updated_at
FROM tool
WHERE (created_at < sqlc.narg(cursor) OR sqlc.narg(cursor) IS NULL)
  AND (CAST(tool_group_id = sqlc.narg(tool_group_id) AS TEXT) OR sqlc.narg(tool_group_id) IS NULL)
ORDER BY created_at DESC
LIMIT CAST(sqlc.arg(page_size) AS INTEGER) + 1;

-- name: get_tool_groups_grouped_by_tool_deployment_type_id :many
SELECT
    fi.tool_deployment_type_id,
    CAST(
        JSON_GROUP_ARRAY(
            JSON_OBJECT(
                'id', pi.id,
                'display_name', pi.display_name,
                'tool_group_deployment_type_id', pi.tool_group_deployment_type_id,
                'credential_deployment_type_id', pi.credential_deployment_type_id,
                'status', pi.status,
                'return_on_successful_brokering', pi.return_on_successful_brokering,
                'created_at', strftime('%Y-%m-%dT%H:%M:%fZ', pi.created_at),
                'updated_at', strftime('%Y-%m-%dT%H:%M:%fZ', pi.updated_at),

                -- resource server credential
                'resource_server_credential', COALESCE((
                    SELECT JSON_OBJECT(
                        'id', rsc.id,
                        'type_id', rsc.type_id,
                        'metadata', JSON(rsc.metadata),
                        'value', JSON(rsc.value),
                        'created_at', strftime('%Y-%m-%dT%H:%M:%fZ', rsc.created_at),
                        'updated_at', strftime('%Y-%m-%dT%H:%M:%fZ', rsc.updated_at),
                        'next_rotation_time', CASE
                            WHEN rsc.next_rotation_time IS NOT NULL
                            THEN strftime('%Y-%m-%dT%H:%M:%fZ', rsc.next_rotation_time)
                            ELSE NULL END,
                        'dek_alias', rsc.dek_alias
                    )
                    FROM resource_server_credential rsc
                    WHERE rsc.id = pi.resource_server_credential_id
                ), JSON('null')),

                -- user credential
                'user_credential', COALESCE((
                    SELECT JSON_OBJECT(
                        'id', uc.id,
                        'type_id', uc.type_id,
                        'metadata', JSON(uc.metadata),
                        'value', JSON(uc.value),
                        'created_at', strftime('%Y-%m-%dT%H:%M:%fZ', uc.created_at),
                        'updated_at', strftime('%Y-%m-%dT%H:%M:%fZ', uc.updated_at),
                        'next_rotation_time', CASE
                            WHEN uc.next_rotation_time IS NOT NULL
                            THEN strftime('%Y-%m-%dT%H:%M:%fZ', uc.next_rotation_time)
                            ELSE NULL END,
                        'dek_alias', uc.dek_alias
                    )
                    FROM user_credential uc
                    WHERE uc.id = pi.user_credential_id
                ), JSON('null')),

                -- include tool metadata
                'tool', JSON_OBJECT(
                    'tool_group_deployment_type_id', fi.tool_group_deployment_type_id,
                    'tool_group_id', fi.tool_group_id,
                    'created_at', strftime('%Y-%m-%dT%H:%M:%fZ', fi.created_at),
                    'updated_at', strftime('%Y-%m-%dT%H:%M:%fZ', fi.updated_at)
                )
            )
        ) AS TEXT
    ) AS tool_groups
FROM tool fi
JOIN tool_group pi ON fi.tool_group_id = pi.id
WHERE (
    fi.tool_deployment_type_id IN (sqlc.narg('tool_deployment_type_ids'))
    OR sqlc.narg('tool_deployment_type_ids') IS NULL
)
GROUP BY fi.tool_deployment_type_id
ORDER BY fi.tool_deployment_type_id ASC;

-- name: update_resource_server_credential :exec
UPDATE resource_server_credential
SET value = CASE WHEN CAST(sqlc.narg(value) AS JSON) IS NOT NULL
    THEN sqlc.narg(value)
    ELSE value
    END,
    metadata = CASE WHEN CAST(sqlc.narg(metadata) AS JSON) IS NOT NULL
    THEN sqlc.narg(metadata)
    ELSE metadata
    END,
    next_rotation_time = CASE WHEN CAST(sqlc.narg(next_rotation_time) AS DATETIME) IS NOT NULL
    THEN sqlc.narg(next_rotation_time)
    ELSE next_rotation_time
    END,
    updated_at = CASE WHEN CAST(sqlc.narg(updated_at) AS DATETIME) IS NOT NULL
    THEN sqlc.narg(updated_at)
    ELSE CURRENT_TIMESTAMP
    END
WHERE id = sqlc.arg(id);

-- name: update_user_credential :exec
UPDATE user_credential
SET value = CASE WHEN CAST(sqlc.narg(value) AS JSON) IS NOT NULL
    THEN sqlc.narg(value)
    ELSE value
    END,
    metadata = CASE WHEN CAST(sqlc.narg(metadata) AS JSON) IS NOT NULL
    THEN sqlc.narg(metadata)
    ELSE metadata
    END,
    next_rotation_time = CASE WHEN CAST(sqlc.narg(next_rotation_time) AS DATETIME) IS NOT NULL
    THEN sqlc.narg(next_rotation_time)
    ELSE next_rotation_time
    END,
    updated_at = CASE WHEN CAST(sqlc.narg(updated_at) AS DATETIME) IS NOT NULL
    THEN sqlc.narg(updated_at)
    ELSE CURRENT_TIMESTAMP
    END
WHERE id = sqlc.arg(id);

-- name: get_tool_groups_with_credentials :many
SELECT
    pi.id,
    pi.display_name,
    pi.tool_group_deployment_type_id,
    pi.credential_deployment_type_id,
    pi.status,
    pi.return_on_successful_brokering,
    pi.created_at,
    pi.updated_at,
    CAST(JSON_OBJECT(
        'id', rsc.id,
        'type_id', rsc.type_id,
        'metadata', JSON(rsc.metadata),
        'value', JSON(rsc.value),
        'created_at', strftime('%Y-%m-%dT%H:%M:%fZ', rsc.created_at),
        'updated_at', strftime('%Y-%m-%dT%H:%M:%fZ', rsc.updated_at),
        'next_rotation_time', CASE
            WHEN rsc.next_rotation_time IS NOT NULL
            THEN strftime('%Y-%m-%dT%H:%M:%fZ', rsc.next_rotation_time)
            ELSE NULL END,
        'dek_alias', rsc.dek_alias
    ) AS TEXT) as resource_server_credential,
    CAST(COALESCE(
        CASE WHEN uc.id IS NOT NULL THEN
            JSON_OBJECT(
                'id', uc.id,
                'type_id', uc.type_id,
                'metadata', JSON(uc.metadata),
                'value', JSON(uc.value),
                'created_at', strftime('%Y-%m-%dT%H:%M:%fZ', uc.created_at),
                'updated_at', strftime('%Y-%m-%dT%H:%M:%fZ', uc.updated_at),
                'next_rotation_time', CASE
                    WHEN uc.next_rotation_time IS NOT NULL
                    THEN strftime('%Y-%m-%dT%H:%M:%fZ', uc.next_rotation_time)
                    ELSE NULL END,
                'dek_alias', uc.dek_alias
            )
        ELSE NULL END,
    JSON('null')) AS TEXT) as user_credential
FROM tool_group pi
INNER JOIN resource_server_credential rsc ON rsc.id = pi.resource_server_credential_id
LEFT JOIN user_credential uc ON uc.id = pi.user_credential_id
WHERE (pi.created_at < sqlc.narg(cursor) OR sqlc.narg(cursor) IS NULL)
  AND (pi.status = sqlc.narg(status) OR sqlc.narg(status) IS NULL)
  AND (
    (rsc.next_rotation_time IS NOT NULL AND datetime(rsc.next_rotation_time) <= sqlc.narg(rotation_window_end))
    OR
    (uc.next_rotation_time IS NOT NULL AND datetime(uc.next_rotation_time) <= sqlc.narg(rotation_window_end))
    OR
    sqlc.narg(rotation_window_end) IS NULL
  )
ORDER BY pi.created_at DESC
LIMIT CAST(sqlc.arg(page_size) AS INTEGER) + 1;