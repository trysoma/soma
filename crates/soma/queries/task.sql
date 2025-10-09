-- name: insert_task :exec
INSERT INTO task (
    id,
    context_id,
    status,
    metadata,
    created_at,
    updated_at
) VALUES (
    :id,
    :context_id,
    :status,
    :metadata,
    :created_at,
    :updated_at
);

-- name: update_task_status :exec
UPDATE task SET status = :status, updated_at = :updated_at WHERE id = :id;

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

-- name: get_task_timeline_items :many
SELECT * FROM task_timeline WHERE task_id = :task_id AND (created_at < sqlc.narg(cursor) OR sqlc.narg(cursor) IS NULL)
ORDER BY created_at DESC
LIMIT CAST(sqlc.arg(page_size) AS INTEGER) + 1;

-- name: get_task_by_id :one
SELECT * FROM task WHERE id = :id;
