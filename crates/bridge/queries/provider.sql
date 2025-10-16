-- name: create_resource_server_credential :exec
INSERT INTO resource_server_credential (id, type_id, metadata, value, created_at, updated_at, next_rotation_time)
VALUES (?, ?, ?, ?, ?, ?, ?);

-- name: get_resource_server_credential_by_id :one
SELECT id, type_id, metadata, value, created_at, updated_at, next_rotation_time
FROM resource_server_credential
WHERE id = ?;

-- name: create_user_credential :exec
INSERT INTO user_credential (id, type_id, metadata, value, created_at, updated_at, next_rotation_time)
VALUES (?, ?, ?, ?, ?, ?, ?);

-- name: get_user_credential_by_id :one
SELECT id, type_id, metadata, value, created_at, updated_at, next_rotation_time
FROM user_credential
WHERE id = ?;

-- name: create_provider_instance :exec
INSERT INTO provider_instance (id, resource_server_credential_id, user_credential_id, created_at, updated_at, provider_controller_type_id, credential_controller_type_id)
VALUES (?, ?, ?, ?, ?, ?, ?);

-- name: get_provider_instance_by_id :one
SELECT id, resource_server_credential_id, user_credential_id, created_at, updated_at, provider_controller_type_id, credential_controller_type_id
FROM provider_instance
WHERE id = ?;

-- name: create_function_instance :exec
INSERT INTO function_instance (id, created_at, updated_at, provider_instance_id, function_controller_type_id)
VALUES (?, ?, ?, ?, ?);

-- name: get_function_instance_by_id :one
SELECT id, created_at, updated_at, provider_instance_id, function_controller_type_id
FROM function_instance
WHERE id = ?;

-- name: delete_function_instance :exec
DELETE FROM function_instance WHERE id = ?;

-- name: create_broker_state :exec
INSERT INTO broker_state (id, created_at, updated_at, resource_server_cred_id, provider_controller_type_id, credential_controller_type_id, metadata, action)
VALUES (?, ?, ?, ?, ?, ?, ?, ?);

-- name: get_broker_state_by_id :one
SELECT id, created_at, updated_at, resource_server_cred_id, provider_controller_type_id, credential_controller_type_id, metadata, action
FROM broker_state
WHERE id = ?;

-- name: delete_broker_state :exec
DELETE FROM broker_state WHERE id = ?;

-- name: get_function_instance_with_credentials :one
SELECT
    fi.id as function_instance_id,
    fi.created_at as function_instance_created_at,
    fi.updated_at as function_instance_updated_at,
    fi.provider_instance_id as function_instance_provider_instance_id,
    fi.function_controller_type_id,
    pi.id as provider_instance_id,
    pi.resource_server_credential_id as provider_instance_resource_server_credential_id,
    pi.user_credential_id as provider_instance_user_credential_id,
    pi.created_at as provider_instance_created_at,
    pi.updated_at as provider_instance_updated_at,
    pi.provider_controller_type_id,
    pi.credential_controller_type_id,
    rsc.id as resource_server_credential_id,
    rsc.type_id as resource_server_credential_type_id,
    rsc.metadata as resource_server_credential_metadata,
    rsc.value as resource_server_credential_value,
    rsc.created_at as resource_server_credential_created_at,
    rsc.updated_at as resource_server_credential_updated_at,
    rsc.next_rotation_time as resource_server_credential_next_rotation_time,
    uc.id as user_credential_id,
    uc.type_id as user_credential_type_id,
    uc.metadata as user_credential_metadata,
    uc.value as user_credential_value,
    uc.created_at as user_credential_created_at,
    uc.updated_at as user_credential_updated_at,
    uc.next_rotation_time as user_credential_next_rotation_time
FROM function_instance fi
JOIN provider_instance pi ON fi.provider_instance_id = pi.id
JOIN resource_server_credential rsc ON pi.resource_server_credential_id = rsc.id
JOIN user_credential uc ON pi.user_credential_id = uc.id
WHERE fi.id = ?;

-- name: create_data_encryption_key :exec
INSERT INTO data_encryption_key (id, envelope_encryption_key_id, encryption_key, created_at, updated_at)
VALUES (?, ?, ?, ?, ?);

-- name: get_data_encryption_key_by_id :one
SELECT id, envelope_encryption_key_id, encryption_key, created_at, updated_at
FROM data_encryption_key
WHERE id = ?;
