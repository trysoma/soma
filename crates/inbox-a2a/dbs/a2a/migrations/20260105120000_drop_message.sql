-- +goose Up
-- drop "message" table
DROP TABLE IF EXISTS `message`;
-- remove status_message_id column from task table
ALTER TABLE `task` DROP COLUMN `status_message_id`;

-- +goose Down
-- reverse: re-add status_message_id column to task table
ALTER TABLE `task` ADD COLUMN `status_message_id` text NULL;
-- reverse: create "message" table
CREATE TABLE `message` (
  `id` text NULL,
  `task_id` text NOT NULL,
  `reference_task_ids` json NOT NULL,
  `role` text NOT NULL,
  `metadata` json NOT NULL,
  `parts` json NOT NULL,
  `created_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  PRIMARY KEY (`id`)
);
