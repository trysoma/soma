-- name: create_resource_server_credential :exec
INSERT INTO resource_server_credential (id, credential_type, credential_data, metadata, run_refresh_before) VALUES (?, ?, ?, ?, ?);

-- name: create_user_credential :exec
INSERT INTO user_credential (id, credential_type, credential_data, metadata, run_refresh_before) VALUES (?, ?, ?, ?, ?);

-- name: create_provider_instance :exec
INSERT INTO provider_instance (id, provider_id, resource_server_credential_id, user_credential_id) VALUES (?, ?, ?, ?);

-- name: create_function_instance :exec
INSERT INTO function_instance (id, function_id, provider_instance_id) VALUES (?, ?, ?);

-- name: create_credential_exchange_state :exec
INSERT INTO credential_exchange_state (id, state) VALUES (?, ?);

-- name: get_credential_exchange_state_by_id :one
SELECT id, state, created_at, updated_at FROM credential_exchange_state WHERE id = ?;
