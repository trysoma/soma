-- +goose Up
-- create "resource_server_credential" table
CREATE TABLE `resource_server_credential` (
  `id` text NULL,
  `type_id` text NOT NULL,
  `metadata` json NOT NULL,
  `value` json NOT NULL,
  `created_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  `updated_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  `next_rotation_time` datetime NULL,
  `dek_alias` text NOT NULL,
  PRIMARY KEY (`id`)
);
-- create "user_credential" table
CREATE TABLE `user_credential` (
  `id` text NULL,
  `type_id` text NOT NULL,
  `metadata` json NOT NULL,
  `value` json NOT NULL,
  `created_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  `updated_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  `next_rotation_time` datetime NULL,
  `dek_alias` text NOT NULL,
  PRIMARY KEY (`id`)
);
-- create "provider_instance" table
CREATE TABLE `provider_instance` (
  `id` text NULL,
  `display_name` text NOT NULL,
  `resource_server_credential_id` text NOT NULL,
  `user_credential_id` text NULL,
  `created_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  `updated_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  `provider_controller_type_id` text NOT NULL,
  `credential_controller_type_id` text NOT NULL,
  `status` text NOT NULL,
  `return_on_successful_brokering` json NULL,
  PRIMARY KEY (`id`),
  CONSTRAINT `0` FOREIGN KEY (`user_credential_id`) REFERENCES `user_credential` (`id`) ON UPDATE NO ACTION ON DELETE NO ACTION,
  CONSTRAINT `1` FOREIGN KEY (`resource_server_credential_id`) REFERENCES `resource_server_credential` (`id`) ON UPDATE NO ACTION ON DELETE NO ACTION,
  CHECK (status IN ('brokering_initiated', 'active', 'disabled'))
);
-- create "function_instance" table
CREATE TABLE `function_instance` (
  `function_controller_type_id` text NOT NULL,
  `provider_controller_type_id` text NOT NULL,
  `provider_instance_id` text NOT NULL,
  `created_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  `updated_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  PRIMARY KEY (`function_controller_type_id`, `provider_controller_type_id`, `provider_instance_id`),
  CONSTRAINT `0` FOREIGN KEY (`provider_instance_id`) REFERENCES `provider_instance` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE
);
-- create "broker_state" table
CREATE TABLE `broker_state` (
  `id` text NULL,
  `created_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  `updated_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  `provider_instance_id` text NOT NULL,
  `provider_controller_type_id` text NOT NULL,
  `credential_controller_type_id` text NOT NULL,
  `metadata` json NOT NULL,
  `action` json NOT NULL,
  PRIMARY KEY (`id`)
);

-- +goose Down
-- reverse: create "broker_state" table
DROP TABLE `broker_state`;
-- reverse: create "function_instance" table
DROP TABLE `function_instance`;
-- reverse: create "provider_instance" table
DROP TABLE `provider_instance`;
-- reverse: create "user_credential" table
DROP TABLE `user_credential`;
-- reverse: create "resource_server_credential" table
DROP TABLE `resource_server_credential`;
