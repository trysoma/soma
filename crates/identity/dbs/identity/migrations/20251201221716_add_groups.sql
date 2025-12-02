-- +goose Up
-- create "group" table
CREATE TABLE `group` (
  `id` text NULL,
  `name` text NOT NULL,
  `created_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  `updated_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  PRIMARY KEY (`id`)
);
-- create "group_membership" table
CREATE TABLE `group_membership` (
  `group_id` text NOT NULL,
  `user_id` text NOT NULL,
  `created_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  `updated_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  PRIMARY KEY (`group_id`, `user_id`),
  CONSTRAINT `0` FOREIGN KEY (`user_id`) REFERENCES `user` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE,
  CONSTRAINT `1` FOREIGN KEY (`group_id`) REFERENCES `group` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE
);

-- +goose Down
-- reverse: create "group_membership" table
DROP TABLE `group_membership`;
-- reverse: create "group" table
DROP TABLE `group`;
