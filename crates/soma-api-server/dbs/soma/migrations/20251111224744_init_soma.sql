-- +goose Up
-- create "task" table
CREATE TABLE `task` (`id` text NULL, `context_id` text NOT NULL, `status` text NOT NULL, `status_message_id` text NULL, `status_timestamp` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP), `metadata` json NOT NULL, `created_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP), `updated_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP), PRIMARY KEY (`id`), CONSTRAINT `status_check` CHECK (status IN ("submitted", "working", "input-required", "completed", "canceled", "failed", "rejected", "auth-required", "unknown")));
-- create "task_timeline" table
CREATE TABLE `task_timeline` (`id` text NULL, `task_id` text NOT NULL, `event_update_type` text NOT NULL, `event_payload` json NOT NULL, `created_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP), PRIMARY KEY (`id`), CONSTRAINT `event_update_type_check` CHECK (event_update_type IN ('task-status-update', 'message')));
-- create "message" table
CREATE TABLE `message` (`id` text NULL, `task_id` text NOT NULL, `reference_task_ids` json NOT NULL, `role` text NOT NULL, `metadata` json NOT NULL, `parts` json NOT NULL, `created_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP), PRIMARY KEY (`id`));

-- +goose Down
-- reverse: create "message" table
DROP TABLE `message`;
-- reverse: create "task_timeline" table
DROP TABLE `task_timeline`;
-- reverse: create "task" table
DROP TABLE `task`;
