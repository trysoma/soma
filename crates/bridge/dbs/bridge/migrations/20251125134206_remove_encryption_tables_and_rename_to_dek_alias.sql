-- +goose Up
-- disable the enforcement of foreign-keys constraints
PRAGMA foreign_keys = off;
-- create "new_resource_server_credential" table
CREATE TABLE `new_resource_server_credential` (
  `id` text NULL,
  `type_id` text NOT NULL,
  `metadata` json NOT NULL,
  `value` json NOT NULL,
  `created_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  `updated_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  `next_rotation_time` datetime NULL,
  `dek_alias` text NOT NULL,
  PRIMARY KEY (`id`)
);
-- copy rows from old table "resource_server_credential" to new temporary table "new_resource_server_credential"
INSERT INTO `new_resource_server_credential` (`id`, `type_id`, `metadata`, `value`, `created_at`, `updated_at`, `next_rotation_time`) SELECT `id`, `type_id`, `metadata`, `value`, `created_at`, `updated_at`, `next_rotation_time` FROM `resource_server_credential`;
-- drop "resource_server_credential" table after copying rows
DROP TABLE `resource_server_credential`;
-- rename temporary table "new_resource_server_credential" to "resource_server_credential"
ALTER TABLE `new_resource_server_credential` RENAME TO `resource_server_credential`;
-- create "new_user_credential" table
CREATE TABLE `new_user_credential` (
  `id` text NULL,
  `type_id` text NOT NULL,
  `metadata` json NOT NULL,
  `value` json NOT NULL,
  `created_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  `updated_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  `next_rotation_time` datetime NULL,
  `dek_alias` text NOT NULL,
  PRIMARY KEY (`id`)
);
-- copy rows from old table "user_credential" to new temporary table "new_user_credential"
INSERT INTO `new_user_credential` (`id`, `type_id`, `metadata`, `value`, `created_at`, `updated_at`, `next_rotation_time`) SELECT `id`, `type_id`, `metadata`, `value`, `created_at`, `updated_at`, `next_rotation_time` FROM `user_credential`;
-- drop "user_credential" table after copying rows
DROP TABLE `user_credential`;
-- rename temporary table "new_user_credential" to "user_credential"
ALTER TABLE `new_user_credential` RENAME TO `user_credential`;
-- drop "data_encryption_key" table
DROP TABLE `data_encryption_key`;
-- drop "envelope_encryption_key" table
DROP TABLE `envelope_encryption_key`;
-- enable back the enforcement of foreign-keys constraints
PRAGMA foreign_keys = on;

-- +goose Down
-- reverse: drop "envelope_encryption_key" table
CREATE TABLE `envelope_encryption_key` (
  `id` text NULL,
  `key_type` text NOT NULL,
  `local_location` text NULL,
  `aws_arn` text NULL,
  `aws_region` text NULL,
  `created_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  `updated_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  PRIMARY KEY (`id`),
  CHECK (key_type IN ('local', 'aws_kms')),
  CHECK (
        (key_type = 'local' AND local_location IS NOT NULL AND aws_arn IS NULL AND aws_region IS NULL) OR
        (key_type = 'aws_kms' AND aws_arn IS NOT NULL AND aws_region IS NOT NULL AND local_location IS NULL)
    )
);
-- reverse: drop "data_encryption_key" table
CREATE TABLE `data_encryption_key` (
  `id` text NULL,
  `envelope_encryption_key_id` text NOT NULL,
  `encryption_key` text NOT NULL,
  `created_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  `updated_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  PRIMARY KEY (`id`),
  CONSTRAINT `0` FOREIGN KEY (`envelope_encryption_key_id`) REFERENCES `envelope_encryption_key` (`id`) ON UPDATE NO ACTION ON DELETE NO ACTION
);
-- reverse: create "new_user_credential" table
DROP TABLE `new_user_credential`;
-- reverse: create "new_resource_server_credential" table
DROP TABLE `new_resource_server_credential`;
