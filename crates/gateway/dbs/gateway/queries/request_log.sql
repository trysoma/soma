-- name: CreateRequestLog :exec
INSERT INTO request_log (
    id,
    model,
    prompt_tokens,
    completion_tokens,
    total_tokens,
    created_at,
    updated_at
) VALUES (?, ?, ?, ?, ?, ?, ?);

-- name: GetRequestLogById :one
SELECT * FROM request_log WHERE id = ?;

-- name: ListRequestLogs :many
SELECT * FROM request_log ORDER BY created_at DESC LIMIT ? OFFSET ?;
