-- +goose Up
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
