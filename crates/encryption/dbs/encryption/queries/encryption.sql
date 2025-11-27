-- name: create_envelope_encryption_key :exec
INSERT INTO envelope_encryption_key (id, key_type, local_file_name, aws_arn, aws_region, created_at, updated_at)
VALUES (?, ?, ?, ?, ?, ?, ?);

-- name: get_envelope_encryption_key_by_id :one
SELECT id, key_type, local_file_name, aws_arn, aws_region, created_at, updated_at
FROM envelope_encryption_key
WHERE id = ?;

-- name: get_envelope_encryption_keys :many
SELECT id, key_type, local_file_name, aws_arn, aws_region, created_at, updated_at
FROM envelope_encryption_key
ORDER BY created_at DESC;

-- name: get_envelope_encryption_keys_paginated :many
SELECT id, key_type, local_file_name, aws_arn, aws_region, created_at, updated_at
FROM envelope_encryption_key 
WHERE (created_at < sqlc.narg(cursor) OR sqlc.narg(cursor) IS NULL)
ORDER BY created_at DESC
LIMIT CAST(sqlc.arg(page_size) AS INTEGER) + 1;

-- name: get_data_encryption_keys_by_envelope_key_id :many
SELECT id, envelope_encryption_key_id, created_at, updated_at
FROM data_encryption_key 
WHERE envelope_encryption_key_id = ?
  AND (created_at < sqlc.narg(cursor) OR sqlc.narg(cursor) IS NULL)
ORDER BY created_at DESC
LIMIT CAST(sqlc.arg(page_size) AS INTEGER) + 1;

-- name: delete_envelope_encryption_key :exec
DELETE FROM envelope_encryption_key WHERE id = ?;

-- name: create_data_encryption_key :exec
INSERT INTO data_encryption_key (id, envelope_encryption_key_id, encryption_key, created_at, updated_at)
VALUES (?, ?, ?, ?, ?);

-- name: get_data_encryption_key_by_id :one
SELECT id, envelope_encryption_key_id, encryption_key, created_at, updated_at
FROM data_encryption_key
WHERE id = ?;

-- name: get_data_encryption_key_by_id_with_envelope :one
SELECT 
    dek.id,
    dek.envelope_encryption_key_id,
    dek.encryption_key,
    dek.created_at,
    dek.updated_at,
    eek.key_type,
    eek.local_file_name,
    eek.aws_arn,
    eek.aws_region
FROM data_encryption_key dek
JOIN envelope_encryption_key eek ON dek.envelope_encryption_key_id = eek.id
WHERE dek.id = ?;

-- name: delete_data_encryption_key :exec
DELETE FROM data_encryption_key WHERE id = ?;

-- name: get_data_encryption_keys :many
SELECT id, envelope_encryption_key_id, created_at, updated_at
FROM data_encryption_key 
WHERE (created_at < sqlc.narg(cursor) OR sqlc.narg(cursor) IS NULL)
ORDER BY created_at DESC
LIMIT CAST(sqlc.arg(page_size) AS INTEGER) + 1;

-- name: get_all_data_encryption_keys_with_envelope_keys :many
SELECT
    dek.id,
    dek.envelope_encryption_key_id,
    dek.encryption_key,
    dek.created_at,
    dek.updated_at,
    eek.key_type,
    eek.local_file_name,
    eek.aws_arn,
    eek.aws_region
FROM data_encryption_key dek
JOIN envelope_encryption_key eek ON dek.envelope_encryption_key_id = eek.id;

-- name: create_data_encryption_key_alias :exec
INSERT INTO data_encryption_key_alias (alias, data_encryption_key_id, created_at)
VALUES (?, ?, ?);

-- name: get_data_encryption_key_alias_by_alias :one
SELECT alias, data_encryption_key_id, created_at
FROM data_encryption_key_alias
WHERE alias = ?;

-- name: get_data_encryption_key_by_alias :one
SELECT dek.id, dek.envelope_encryption_key_id, dek.encryption_key, dek.created_at, dek.updated_at
FROM data_encryption_key dek
JOIN data_encryption_key_alias alias ON dek.id = alias.data_encryption_key_id
WHERE alias.alias = ?;

-- name: delete_data_encryption_key_alias :exec
DELETE FROM data_encryption_key_alias WHERE alias = ?;

-- name: list_aliases_for_dek :many
SELECT alias, data_encryption_key_id, created_at
FROM data_encryption_key_alias
WHERE data_encryption_key_id = ?
ORDER BY created_at ASC;

-- name: update_data_encryption_key_alias :exec
UPDATE data_encryption_key_alias
SET data_encryption_key_id = ?
WHERE alias = ?;
