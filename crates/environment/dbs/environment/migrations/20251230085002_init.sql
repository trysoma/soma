-- +goose Up
-- create "secret" table
CREATE TABLE `secret` (
  `id` text NULL,
  `key` text NOT NULL,
  `encrypted_secret` text NOT NULL,
  `dek_alias` text NOT NULL,
  `created_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  `updated_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  PRIMARY KEY (`id`)
);
-- create index "secret_key" to table: "secret"
CREATE UNIQUE INDEX `secret_key` ON `secret` (`key`);
-- create "variable" table
CREATE TABLE `variable` (
  `id` text NULL,
  `key` text NOT NULL,
  `value` text NOT NULL,
  `created_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  `updated_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  PRIMARY KEY (`id`)
);
-- create index "variable_key" to table: "variable"
CREATE UNIQUE INDEX `variable_key` ON `variable` (`key`);

-- +goose Down
-- reverse: create index "variable_key" to table: "variable"
DROP INDEX `variable_key`;
-- reverse: create "variable" table
DROP TABLE `variable`;
-- reverse: create index "secret_key" to table: "secret"
DROP INDEX `secret_key`;
-- reverse: create "secret" table
DROP TABLE `secret`;
