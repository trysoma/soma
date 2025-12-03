-- +goose Up
-- Update user type constraint from service_principal/federated_user to machine/human
-- SQLite doesn't support ALTER CONSTRAINT, so we need to recreate the table

-- Create new user table with updated constraint
CREATE TABLE `user_new` (
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

-- Copy data, converting old types to new types
INSERT INTO `user_new` (`id`, `type`, `email`, `role`, `description`, `created_at`, `updated_at`)
SELECT
  `id`,
  CASE
    WHEN `type` = 'service_principal' THEN 'machine'
    WHEN `type` = 'federated_user' THEN 'human'
    ELSE `type`
  END,
  `email`,
  `role`,
  `description`,
  `created_at`,
  `updated_at`
FROM `user`;

-- Drop old table and rename new one
DROP TABLE `user`;
ALTER TABLE `user_new` RENAME TO `user`;

-- +goose Down
-- Revert back to old constraint
CREATE TABLE `user_new` (
  `id` text NULL,
  `type` text NOT NULL,
  `email` text NULL,
  `role` text NOT NULL,
  `description` text NULL,
  `created_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  `updated_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  PRIMARY KEY (`id`),
  CONSTRAINT `type_check` CHECK (type IN ('service_principal', 'federated_user'))
);

INSERT INTO `user_new` (`id`, `type`, `email`, `role`, `description`, `created_at`, `updated_at`)
SELECT
  `id`,
  CASE
    WHEN `type` = 'machine' THEN 'service_principal'
    WHEN `type` = 'human' THEN 'federated_user'
    ELSE `type`
  END,
  `email`,
  `role`,
  `description`,
  `created_at`,
  `updated_at`
FROM `user`;

DROP TABLE `user`;
ALTER TABLE `user_new` RENAME TO `user`;
