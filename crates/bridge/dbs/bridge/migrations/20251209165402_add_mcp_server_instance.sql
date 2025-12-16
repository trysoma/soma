-- +goose Up
-- create "mcp_server_instance" table
CREATE TABLE `mcp_server_instance` (
    `id` text NOT NULL,
    `name` text NOT NULL,
    `created_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
    `updated_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
    PRIMARY KEY (`id`)
);

-- create "mcp_server_instance_function" table
CREATE TABLE `mcp_server_instance_function` (
    `mcp_server_instance_id` text NOT NULL,
    `function_controller_type_id` text NOT NULL,
    `provider_controller_type_id` text NOT NULL,
    `provider_instance_id` text NOT NULL,
    `function_name` text NOT NULL,
    `function_description` text NULL,
    `created_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
    `updated_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
    PRIMARY KEY (`mcp_server_instance_id`, `function_controller_type_id`, `provider_controller_type_id`, `provider_instance_id`),
    CONSTRAINT `fk_mcp_server_instance` FOREIGN KEY (`mcp_server_instance_id`) REFERENCES `mcp_server_instance` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE,
    CONSTRAINT `fk_function_instance` FOREIGN KEY (`function_controller_type_id`, `provider_controller_type_id`, `provider_instance_id`) REFERENCES `function_instance` (`function_controller_type_id`, `provider_controller_type_id`, `provider_instance_id`) ON UPDATE NO ACTION ON DELETE CASCADE
);

-- create unique index for function_name within MCP server instance
CREATE UNIQUE INDEX `idx_mcp_server_instance_function_name` ON `mcp_server_instance_function` (`mcp_server_instance_id`, `function_name`);

-- +goose Down
-- reverse: drop index
DROP INDEX `idx_mcp_server_instance_function_name`;
-- reverse: drop "mcp_server_instance_function" table
DROP TABLE `mcp_server_instance_function`;
-- reverse: drop "mcp_server_instance" table
DROP TABLE `mcp_server_instance`;
