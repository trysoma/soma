-- +goose Up
-- create "message_v2" table
CREATE TABLE `message_v2` (`id` text NULL, `task_id` text NOT NULL, `reference_task_ids` json NOT NULL, `role` text NOT NULL, `metadata` json NOT NULL, `parts` json NOT NULL, `created_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP), PRIMARY KEY (`id`));

-- +goose Down
-- reverse: create "message_v2" table
DROP TABLE `message_v2`;
