-- +goose Up
-- disable the enforcement of foreign-keys constraints
PRAGMA foreign_keys = off;
-- create "new_data_encryption_key" table
CREATE TABLE `new_data_encryption_key` (
  `id` text NULL,
  `envelope_encryption_key_id` text NOT NULL,
  `encryption_key` text NOT NULL,
  `created_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  `updated_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  PRIMARY KEY (`id`),
  CONSTRAINT `0` FOREIGN KEY (`envelope_encryption_key_id`) REFERENCES `envelope_encryption_key` (`id`) ON UPDATE NO ACTION ON DELETE NO ACTION
);
-- copy rows from old table "data_encryption_key" to new temporary table "new_data_encryption_key"
INSERT INTO `new_data_encryption_key` (`id`, `envelope_encryption_key_id`, `encryption_key`, `created_at`, `updated_at`) SELECT `id`, `envelope_encryption_key_id`, `encryption_key`, `created_at`, `updated_at` FROM `data_encryption_key`;
-- drop "data_encryption_key" table after copying rows
DROP TABLE `data_encryption_key`;
-- rename temporary table "new_data_encryption_key" to "data_encryption_key"
ALTER TABLE `new_data_encryption_key` RENAME TO `data_encryption_key`;
-- create "envelope_encryption_key" table
CREATE TABLE `envelope_encryption_key` (
  `id` text NULL,
  `type` text NOT NULL,
  `local_location` text NULL,
  `aws_arn` text NULL,
  `aws_region` text NULL,
  `created_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  `updated_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  PRIMARY KEY (`id`),
  CHECK (type IN ('local', 'aws_kms')),
  CHECK (
        (type = 'local' AND local_location IS NOT NULL AND aws_arn IS NULL AND aws_region IS NULL) OR
        (type = 'aws_kms' AND aws_arn IS NOT NULL AND aws_region IS NOT NULL AND local_location IS NULL)
    )
);
-- enable back the enforcement of foreign-keys constraints
PRAGMA foreign_keys = on;

-- +goose Down
-- reverse: create "envelope_encryption_key" table
DROP TABLE `envelope_encryption_key`;
-- reverse: create "new_data_encryption_key" table
DROP TABLE `new_data_encryption_key`;
