-- +goose Up
-- Rename idp_configuration table to user_auth_flow_configuration
ALTER TABLE idp_configuration RENAME TO user_auth_flow_configuration;

-- Update oauth_state foreign key reference
-- SQLite doesn't support ALTER TABLE to modify foreign keys,
-- but the foreign key constraint name stays the same and the
-- renamed table will be referenced correctly

-- +goose Down
ALTER TABLE user_auth_flow_configuration RENAME TO idp_configuration;
