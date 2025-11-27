-- +goose Up
-- create "secret" table for storing encrypted secrets
CREATE TABLE `secret` (`id` text NOT NULL, `key` text NOT NULL UNIQUE, `encrypted_secret` text NOT NULL, `dek_alias` text NOT NULL, `created_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP), `updated_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP), PRIMARY KEY (`id`));

-- +goose Down
-- reverse: create "secret" table
DROP TABLE `secret`;
