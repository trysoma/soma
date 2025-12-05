-- ============================================================================
-- OAuth state table queries
-- ============================================================================

-- name: create_oauth_state :exec
INSERT INTO oauth_state (state, config_id, code_verifier, nonce, redirect_uri, created_at, expires_at)
VALUES (sqlc.arg(state), sqlc.arg(config_id), sqlc.arg(code_verifier), sqlc.arg(nonce), sqlc.arg(redirect_uri), sqlc.arg(created_at), sqlc.arg(expires_at));

-- name: get_oauth_state_by_state :one
SELECT state, config_id, code_verifier, nonce, redirect_uri, created_at, expires_at
FROM oauth_state
WHERE state = ?;

-- name: delete_oauth_state :exec
DELETE FROM oauth_state WHERE state = ?;

-- name: delete_expired_oauth_states :exec
DELETE FROM oauth_state WHERE expires_at < sqlc.arg(now);
