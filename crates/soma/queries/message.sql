-- name: insert_message :exec
INSERT INTO message (
    id,
    task_id,
    reference_task_ids,
    role,
    metadata,
    parts,
    created_at
) VALUES (
    :id,
    :task_id,
    :reference_task_ids,
    :role,
    :metadata,
    :parts,
    :created_at
);

-- name: get_messages_by_task_id :many
SELECT * FROM message WHERE task_id = :task_id AND (created_at < sqlc.narg(cursor) OR sqlc.narg(cursor) IS NULL)
ORDER BY created_at DESC
LIMIT CAST(sqlc.arg(page_size) AS INTEGER) + 1;
