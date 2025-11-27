-- +goose Up
-- create "envelope_encryption_key" table
CREATE TABLE `envelope_encryption_key` (
  `id` text NULL,
  `key_type` text NOT NULL,
  `local_file_name` text NULL,
  `aws_arn` text NULL,
  `aws_region` text NULL,
  `created_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  `updated_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  PRIMARY KEY (`id`),
  CHECK (key_type IN ('local', 'aws_kms')),
  CHECK (
        (key_type = 'local' AND local_file_name IS NOT NULL AND aws_arn IS NULL AND aws_region IS NULL) OR
        (key_type = 'aws_kms' AND aws_arn IS NOT NULL AND aws_region IS NOT NULL AND local_file_name IS NULL)
    )
);
-- create "data_encryption_key" table
CREATE TABLE `data_encryption_key` (
  `id` text NULL,
  `envelope_encryption_key_id` text NOT NULL,
  `encryption_key` text NOT NULL,
  `created_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  `updated_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  PRIMARY KEY (`id`),
  CONSTRAINT `0` FOREIGN KEY (`envelope_encryption_key_id`) REFERENCES `envelope_encryption_key` (`id`) ON UPDATE NO ACTION ON DELETE NO ACTION
);
-- create "data_encryption_key_alias" table
CREATE TABLE `data_encryption_key_alias` (
  `alias` text NULL,
  `data_encryption_key_id` text NOT NULL,
  `created_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  PRIMARY KEY (`alias`),
  CONSTRAINT `0` FOREIGN KEY (`data_encryption_key_id`) REFERENCES `data_encryption_key` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE
);
-- create index "idx_dek_alias_dek_id" to table: "data_encryption_key_alias"
CREATE INDEX `idx_dek_alias_dek_id` ON `data_encryption_key_alias` (`data_encryption_key_id`);

-- +goose Down
-- reverse: create index "idx_dek_alias_dek_id" to table: "data_encryption_key_alias"
DROP INDEX `idx_dek_alias_dek_id`;
-- reverse: create "data_encryption_key_alias" table
DROP TABLE `data_encryption_key_alias`;
-- reverse: create "data_encryption_key" table
DROP TABLE `data_encryption_key`;
-- reverse: create "envelope_encryption_key" table
DROP TABLE `envelope_encryption_key`;
