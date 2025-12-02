-- +goose Up
-- create "api_key" table
CREATE TABLE `api_key` (
  `hashed_value` text NOT NULL,
  `user_id` text NOT NULL,
  `created_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  `updated_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  PRIMARY KEY (`hashed_value`),
  CONSTRAINT `0` FOREIGN KEY (`user_id`) REFERENCES `user` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE
);
-- create "user" table
CREATE TABLE `user` (
  `id` text NULL,
  `type` text NOT NULL,
  `email` text NULL,
  `role` text NOT NULL,
  `created_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  `updated_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  PRIMARY KEY (`id`),
  CONSTRAINT `type_check` CHECK (type IN ('service_principal', 'federated_user'))
);

-- +goose Down
-- reverse: create "user" table
DROP TABLE `user`;
-- reverse: create "api_key" table
DROP TABLE `api_key`;
