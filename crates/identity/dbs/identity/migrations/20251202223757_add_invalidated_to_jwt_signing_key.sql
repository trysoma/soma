-- +goose Up
-- add "invalidated" column to "jwt_signing_key" table
ALTER TABLE `jwt_signing_key` ADD COLUMN `invalidated` integer NOT NULL DEFAULT (0);

-- +goose Down
-- reverse: add "invalidated" column to "jwt_signing_key" table
ALTER TABLE `jwt_signing_key` DROP COLUMN `invalidated`;
