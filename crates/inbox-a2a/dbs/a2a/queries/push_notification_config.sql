-- name: insert_push_notification_config :exec
INSERT INTO push_notification_config (
    id,
    task_id,
    url,
    token,
    authentication,
    created_at,
    updated_at
) VALUES (
    :id,
    :task_id,
    :url,
    :token,
    :authentication,
    :created_at,
    :updated_at
);

-- name: update_push_notification_config :exec
UPDATE push_notification_config SET
    url = :url,
    token = :token,
    authentication = :authentication,
    updated_at = :updated_at
WHERE id = :id;

-- name: get_push_notification_configs_by_task_id :many
SELECT * FROM push_notification_config WHERE task_id = :task_id;

-- name: get_push_notification_config_by_id :one
SELECT * FROM push_notification_config WHERE id = :id;

-- name: delete_push_notification_config :exec
DELETE FROM push_notification_config WHERE id = :id;

-- name: delete_push_notification_configs_by_task_id :exec
DELETE FROM push_notification_config WHERE task_id = :task_id;
