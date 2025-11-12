-- atlas:txtar

-- migration.sql --
-- Create "data_encryption_key" table
CREATE TABLE `data_encryption_key` (
  `id` text NULL,
  `envelope_encryption_key_id` json NOT NULL,
  `encryption_key` text NOT NULL,
  `created_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  `updated_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  PRIMARY KEY (`id`)
);
-- Create "resource_server_credential" table
CREATE TABLE `resource_server_credential` (
  `id` text NULL,
  `type_id` text NOT NULL,
  `metadata` json NOT NULL,
  `value` json NOT NULL,
  `created_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  `updated_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  `next_rotation_time` datetime NULL,
  `data_encryption_key_id` text NOT NULL,
  PRIMARY KEY (`id`),
  CONSTRAINT `0` FOREIGN KEY (`data_encryption_key_id`) REFERENCES `data_encryption_key` (`id`) ON UPDATE NO ACTION ON DELETE NO ACTION
);
-- Create "user_credential" table
CREATE TABLE `user_credential` (
  `id` text NULL,
  `type_id` text NOT NULL,
  `metadata` json NOT NULL,
  `value` json NOT NULL,
  `created_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  `updated_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  `next_rotation_time` datetime NULL,
  `data_encryption_key_id` text NOT NULL,
  PRIMARY KEY (`id`),
  CONSTRAINT `0` FOREIGN KEY (`data_encryption_key_id`) REFERENCES `data_encryption_key` (`id`) ON UPDATE NO ACTION ON DELETE NO ACTION
);
-- Create "provider_instance" table
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
-- Create "function_instance" table
CREATE TABLE `function_instance` (
  `function_controller_type_id` text NOT NULL,
  `provider_controller_type_id` text NOT NULL,
  `provider_instance_id` text NOT NULL,
  `created_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  `updated_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  PRIMARY KEY (`function_controller_type_id`, `provider_controller_type_id`, `provider_instance_id`),
  CONSTRAINT `0` FOREIGN KEY (`provider_instance_id`) REFERENCES `provider_instance` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE
);
-- Create "broker_state" table
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


-- down.sql --
DROP TABLE IF EXISTS broker_state;
DROP TABLE IF EXISTS function_instance;
DROP TABLE IF EXISTS provider_instance;
DROP TABLE IF EXISTS user_credential;
DROP TABLE IF EXISTS resource_server_credential;
DROP TABLE IF EXISTS data_encryption_key;
