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
-- create "tool_group_deployment" table
CREATE TABLE `tool_group_deployment` (
  `type_id` text NOT NULL,
  `deployment_id` text NOT NULL,
  `name` text NOT NULL,
  `documentation` text NOT NULL,
  `categories` json NOT NULL,
  `endpoint_type` text NOT NULL,
  `endpoint_configuration` json NOT NULL,
  `credential_deployments` json NOT NULL,
  `metadata` json NOT NULL,
  `created_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  `updated_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  PRIMARY KEY (`type_id`, `deployment_id`),
  CHECK (endpoint_type IN ('http'))
);
-- create "tool_deployment" table
CREATE TABLE `tool_deployment` (
  `type_id` text NOT NULL,
  `tool_group_deployment_type_id` text NOT NULL,
  `tool_group_deployment_deployment_id` text NOT NULL,
  `name` text NOT NULL,
  `documentation` text NOT NULL,
  `categories` json NOT NULL,
  `metadata` json NOT NULL,
  `created_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  `updated_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  PRIMARY KEY (`type_id`, `tool_group_deployment_type_id`, `tool_group_deployment_deployment_id`),
  CONSTRAINT `0` FOREIGN KEY (`tool_group_deployment_type_id`, `tool_group_deployment_deployment_id`) REFERENCES `tool_group_deployment` (`type_id`, `deployment_id`) ON UPDATE NO ACTION ON DELETE CASCADE
);
-- create "tool_group_deployment_alias" table
CREATE TABLE `tool_group_deployment_alias` (
  `tool_group_deployment_type_id` text NOT NULL,
  `tool_group_deployment_deployment_id` text NOT NULL,
  `alias` text NOT NULL,
  `created_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  `updated_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  PRIMARY KEY (`tool_group_deployment_type_id`, `tool_group_deployment_deployment_id`, `alias`),
  CONSTRAINT `0` FOREIGN KEY (`tool_group_deployment_type_id`, `tool_group_deployment_deployment_id`) REFERENCES `tool_group_deployment` (`type_id`, `deployment_id`) ON UPDATE NO ACTION ON DELETE CASCADE
);
-- create index "idx_tool_group_deployment_alias_unique" to table: "tool_group_deployment_alias"
CREATE UNIQUE INDEX `idx_tool_group_deployment_alias_unique` ON `tool_group_deployment_alias` (`alias`);
-- create "tool_group" table
CREATE TABLE `tool_group` (
  `id` text NULL,
  `display_name` text NOT NULL,
  `alias` text NULL,
  `resource_server_credential_id` text NOT NULL,
  `user_credential_id` text NULL,
  `tool_group_deployment_type_id` text NOT NULL,
  `tool_group_deployment_deployment_id` text NOT NULL,
  `credential_deployment_type_id` text NOT NULL,
  `status` text NOT NULL,
  `return_on_successful_brokering` json NULL,
  `created_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  `updated_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  PRIMARY KEY (`id`),
  CONSTRAINT `0` FOREIGN KEY (`tool_group_deployment_type_id`, `tool_group_deployment_deployment_id`) REFERENCES `tool_group_deployment` (`type_id`, `deployment_id`) ON UPDATE NO ACTION ON DELETE NO ACTION,
  CONSTRAINT `1` FOREIGN KEY (`user_credential_id`) REFERENCES `user_credential` (`id`) ON UPDATE NO ACTION ON DELETE NO ACTION,
  CONSTRAINT `2` FOREIGN KEY (`resource_server_credential_id`) REFERENCES `resource_server_credential` (`id`) ON UPDATE NO ACTION ON DELETE NO ACTION,
  CHECK (status IN ('brokering_initiated', 'active', 'disabled'))
);
-- create index "idx_tool_group_alias" to table: "tool_group"
CREATE INDEX `idx_tool_group_alias` ON `tool_group` (`alias`) WHERE alias IS NOT NULL;
-- create "tool" table
CREATE TABLE `tool` (
  `tool_deployment_type_id` text NOT NULL,
  `tool_group_deployment_type_id` text NOT NULL,
  `tool_group_deployment_deployment_id` text NOT NULL,
  `tool_group_id` text NOT NULL,
  `created_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  `updated_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  PRIMARY KEY (`tool_deployment_type_id`, `tool_group_deployment_type_id`, `tool_group_deployment_deployment_id`, `tool_group_id`),
  CONSTRAINT `0` FOREIGN KEY (`tool_group_id`) REFERENCES `tool_group` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE,
  CONSTRAINT `1` FOREIGN KEY (`tool_deployment_type_id`, `tool_group_deployment_type_id`, `tool_group_deployment_deployment_id`) REFERENCES `tool_deployment` (`type_id`, `tool_group_deployment_type_id`, `tool_group_deployment_deployment_id`) ON UPDATE NO ACTION ON DELETE CASCADE
);
-- create "broker_state" table
CREATE TABLE `broker_state` (
  `id` text NULL,
  `tool_group_id` text NOT NULL,
  `tool_group_deployment_type_id` text NOT NULL,
  `tool_group_deployment_deployment_id` text NOT NULL,
  `credential_deployment_type_id` text NOT NULL,
  `metadata` json NOT NULL,
  `action` json NOT NULL,
  `created_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  `updated_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  PRIMARY KEY (`id`)
);
-- create "mcp_server_instance" table
CREATE TABLE `mcp_server_instance` (
  `id` text NULL,
  `name` text NOT NULL,
  `created_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  `updated_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  PRIMARY KEY (`id`)
);
-- create "mcp_server_instance_tool" table
CREATE TABLE `mcp_server_instance_tool` (
  `mcp_server_instance_id` text NOT NULL,
  `tool_deployment_type_id` text NOT NULL,
  `tool_group_deployment_type_id` text NOT NULL,
  `tool_group_deployment_deployment_id` text NOT NULL,
  `tool_group_id` text NOT NULL,
  `tool_name` text NOT NULL,
  `tool_description` text NULL,
  `created_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  `updated_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  PRIMARY KEY (`mcp_server_instance_id`, `tool_deployment_type_id`, `tool_group_deployment_type_id`, `tool_group_deployment_deployment_id`, `tool_group_id`),
  CONSTRAINT `0` FOREIGN KEY (`tool_deployment_type_id`, `tool_group_deployment_type_id`, `tool_group_deployment_deployment_id`, `tool_group_id`) REFERENCES `tool` (`tool_deployment_type_id`, `tool_group_deployment_type_id`, `tool_group_deployment_deployment_id`, `tool_group_id`) ON UPDATE NO ACTION ON DELETE CASCADE,
  CONSTRAINT `1` FOREIGN KEY (`mcp_server_instance_id`) REFERENCES `mcp_server_instance` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE
);
-- create index "idx_mcp_server_instance_tool_name" to table: "mcp_server_instance_tool"
CREATE UNIQUE INDEX `idx_mcp_server_instance_tool_name` ON `mcp_server_instance_tool` (`mcp_server_instance_id`, `tool_name`);

-- +goose Down
-- reverse: create index "idx_mcp_server_instance_tool_name" to table: "mcp_server_instance_tool"
DROP INDEX `idx_mcp_server_instance_tool_name`;
-- reverse: create "mcp_server_instance_tool" table
DROP TABLE `mcp_server_instance_tool`;
-- reverse: create "mcp_server_instance" table
DROP TABLE `mcp_server_instance`;
-- reverse: create "broker_state" table
DROP TABLE `broker_state`;
-- reverse: create "tool" table
DROP TABLE `tool`;
-- reverse: create index "idx_tool_group_alias" to table: "tool_group"
DROP INDEX `idx_tool_group_alias`;
-- reverse: create "tool_group" table
DROP TABLE `tool_group`;
-- reverse: create index "idx_tool_group_deployment_alias_unique" to table: "tool_group_deployment_alias"
DROP INDEX `idx_tool_group_deployment_alias_unique`;
-- reverse: create "tool_group_deployment_alias" table
DROP TABLE `tool_group_deployment_alias`;
-- reverse: create "tool_deployment" table
DROP TABLE `tool_deployment`;
-- reverse: create "tool_group_deployment" table
DROP TABLE `tool_group_deployment`;
-- reverse: create "user_credential" table
DROP TABLE `user_credential`;
-- reverse: create "resource_server_credential" table
DROP TABLE `resource_server_credential`;
