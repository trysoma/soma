-- +goose Up
-- disable the enforcement of foreign-keys constraints
PRAGMA foreign_keys = off;
-- create "new_envelope_encryption_key" table
CREATE TABLE `new_envelope_encryption_key` (
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
-- copy rows from old table "envelope_encryption_key" to new temporary table "new_envelope_encryption_key"
INSERT INTO `new_envelope_encryption_key` (`id`, `local_location`, `aws_arn`, `aws_region`, `created_at`, `updated_at`) SELECT `id`, `local_location`, `aws_arn`, `aws_region`, `created_at`, `updated_at` FROM `envelope_encryption_key`;
-- drop "envelope_encryption_key" table after copying rows
DROP TABLE `envelope_encryption_key`;
-- rename temporary table "new_envelope_encryption_key" to "envelope_encryption_key"
ALTER TABLE `new_envelope_encryption_key` RENAME TO `envelope_encryption_key`;
-- enable back the enforcement of foreign-keys constraints
PRAGMA foreign_keys = on;

-- +goose Down
-- reverse: create "new_envelope_encryption_key" table
DROP TABLE `new_envelope_encryption_key`;
