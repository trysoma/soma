-- +goose Up
-- create "environment_variable" table for storing environment variables
CREATE TABLE `environment_variable` (`id` text NOT NULL, `key` text NOT NULL UNIQUE, `value` text NOT NULL, `created_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP), `updated_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP), PRIMARY KEY (`id`));

-- +goose Down
-- reverse: create "environment_variable" table
DROP TABLE `environment_variable`;
