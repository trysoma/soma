-- +goose Up
-- create "api_key" table
CREATE TABLE `api_key` (
  `id` text NOT NULL,
  `hashed_value` text NOT NULL,
  `description` text NULL,
  `user_id` text NOT NULL,
  `created_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  `updated_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  PRIMARY KEY (`id`),
  CONSTRAINT `0` FOREIGN KEY (`user_id`) REFERENCES `user` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE
);
-- create index "api_key_hashed_value" to table: "api_key"
CREATE UNIQUE INDEX `api_key_hashed_value` ON `api_key` (`hashed_value`);
-- create index "idx_api_key_hashed_value" to table: "api_key"
CREATE INDEX `idx_api_key_hashed_value` ON `api_key` (`hashed_value`);
-- create "user" table
CREATE TABLE `user` (
  `id` text NULL,
  `type` text NOT NULL,
  `email` text NULL,
  `role` text NOT NULL,
  `description` text NULL,
  `created_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  `updated_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  PRIMARY KEY (`id`),
  CONSTRAINT `type_check` CHECK (type IN ('machine', 'human'))
);
-- create "group" table
CREATE TABLE `group` (
  `id` text NOT NULL,
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
-- create "jwt_signing_key" table
CREATE TABLE `jwt_signing_key` (
  `kid` text NOT NULL,
  `encrypted_private_key` text NOT NULL,
  `expires_at` datetime NOT NULL,
  `public_key` text NOT NULL,
  `dek_alias` text NOT NULL,
  `invalidated` boolean NOT NULL DEFAULT 0,
  `created_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  `updated_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  PRIMARY KEY (`kid`)
);
-- create "sts_configuration" table
CREATE TABLE `sts_configuration` (
  `id` text NOT NULL,
  `type` text NOT NULL,
  `value` text NULL,
  `created_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  `updated_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  PRIMARY KEY (`id`),
  CONSTRAINT `type_check` CHECK (type IN ('jwt_template', 'dev'))
);
-- create "user_auth_flow_configuration" table
CREATE TABLE `user_auth_flow_configuration` (
  `id` text NOT NULL,
  `type` text NOT NULL,
  `config` text NOT NULL,
  `created_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  `updated_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  PRIMARY KEY (`id`),
  CONSTRAINT `type_check` CHECK (type IN ('oidc_authorization_code_flow', 'oauth_authorization_code_flow', 'oidc_authorization_code_pkce_flow', 'oauth_authorization_code_pkce_flow'))
);
-- create "oauth_state" table
CREATE TABLE `oauth_state` (
  `state` text NOT NULL,
  `config_id` text NOT NULL,
  `code_verifier` text NULL,
  `nonce` text NULL,
  `redirect_uri` text NULL,
  `created_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  `expires_at` datetime NOT NULL,
  PRIMARY KEY (`state`),
  CONSTRAINT `0` FOREIGN KEY (`config_id`) REFERENCES `user_auth_flow_configuration` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE
);
-- create index "idx_oauth_state_expires_at" to table: "oauth_state"
CREATE INDEX `idx_oauth_state_expires_at` ON `oauth_state` (`expires_at`);

-- +goose Down
-- reverse: create index "idx_oauth_state_expires_at" to table: "oauth_state"
DROP INDEX `idx_oauth_state_expires_at`;
-- reverse: create "oauth_state" table
DROP TABLE `oauth_state`;
-- reverse: create "user_auth_flow_configuration" table
DROP TABLE `user_auth_flow_configuration`;
-- reverse: create "sts_configuration" table
DROP TABLE `sts_configuration`;
-- reverse: create "jwt_signing_key" table
DROP TABLE `jwt_signing_key`;
-- reverse: create "group_membership" table
DROP TABLE `group_membership`;
-- reverse: create "group" table
DROP TABLE `group`;
-- reverse: create "user" table
DROP TABLE `user`;
-- reverse: create index "idx_api_key_hashed_value" to table: "api_key"
DROP INDEX `idx_api_key_hashed_value`;
-- reverse: create index "api_key_hashed_value" to table: "api_key"
DROP INDEX `api_key_hashed_value`;
-- reverse: create "api_key" table
DROP TABLE `api_key`;
