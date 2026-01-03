-- name: insert_task :exec
INSERT INTO task (
    id,
    context_id,
    status,
    status_timestamp,
    metadata,
    created_at,
    updated_at
) VALUES (
    :id,
    :context_id,
    :status,
    :status_timestamp,
    :metadata,
    :created_at,
    :updated_at
);

-- name: update_task_status :exec
UPDATE task SET status = :status, status_message_id = :status_message_id, status_timestamp = :status_timestamp, updated_at = :updated_at WHERE id = :id;

-- name: insert_task_timeline_item :exec
INSERT INTO task_timeline (
    id,
    task_id,
    event_update_type,
    event_payload,
    created_at
) VALUES (
    :id,
    :task_id,
    :event_update_type,
    :event_payload,
    :created_at
);

-- name: get_tasks :many
SELECT * FROM task WHERE (created_at < sqlc.narg(cursor) OR sqlc.narg(cursor) IS NULL)
ORDER BY created_at DESC
LIMIT CAST(sqlc.arg(page_size) AS INTEGER) + 1;

-- name: get_unique_contexts :many
SELECT DISTINCT context_id, created_at FROM task WHERE (created_at < sqlc.narg(cursor) OR sqlc.narg(cursor) IS NULL)
ORDER BY created_at DESC
LIMIT CAST(sqlc.arg(page_size) AS INTEGER) + 1;

-- name: get_tasks_by_context_id :many
SELECT * FROM task WHERE context_id = :context_id AND (created_at < sqlc.narg(cursor) OR sqlc.narg(cursor) IS NULL)
ORDER BY created_at DESC
LIMIT CAST(sqlc.arg(page_size) AS INTEGER) + 1;


-- name: get_task_timeline_items :many
SELECT * FROM task_timeline WHERE task_id = :task_id AND (created_at < sqlc.narg(cursor) OR sqlc.narg(cursor) IS NULL)
ORDER BY created_at DESC
LIMIT CAST(sqlc.arg(page_size) AS INTEGER) + 1;

-- name: get_task_by_id :one
SELECT
    t.id,
    t.context_id,
    t.status,
    t.status_message_id,
    t.status_timestamp,
    t.metadata,
    t.created_at,
    t.updated_at,
    CAST(
        CASE
            WHEN sm.id IS NULL THEN JSON('[]')
            ELSE JSON_ARRAY(
                JSON_OBJECT(
                    'id', sm.id,
                    'task_id', sm.task_id,
                    'reference_task_ids', JSON(sm.reference_task_ids),
                    'role', sm.role,
                    'metadata', JSON(sm.metadata),
                    'parts', JSON(sm.parts),
                    'created_at', strftime('%Y-%m-%dT%H:%M:%fZ', sm.created_at)
                )
            )
        END AS TEXT
    ) AS status_message,
    (
        SELECT CAST(
            CASE
                WHEN COUNT(m2.id) = 0 THEN JSON('[]')
                ELSE JSON_GROUP_ARRAY(
                    JSON_OBJECT(
                        'id', m2.id,
                        'task_id', m2.task_id,
                        'reference_task_ids', JSON(m2.reference_task_ids),
                        'role', m2.role,
                        'metadata', JSON(m2.metadata),
                        'parts', JSON(m2.parts),
                        'created_at', strftime('%Y-%m-%dT%H:%M:%fZ', m2.created_at)
                    )
                )
            END AS TEXT
        )
        FROM message m2
        WHERE m2.task_id = t.id
        ORDER BY m2.created_at DESC
    ) AS messages
FROM task t
LEFT JOIN message sm ON t.status_message_id = sm.id
WHERE t.id = :id;
