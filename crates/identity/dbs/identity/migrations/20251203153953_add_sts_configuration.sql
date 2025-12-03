-- +goose Up
-- Create sts_configuration table to store STS configuration templates
CREATE TABLE `sts_configuration` (
  `id` text NOT NULL,
  `type` text NOT NULL,
  `value` text NULL,
  `created_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  `updated_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  PRIMARY KEY (`id`),
  CONSTRAINT `type_check` CHECK (type IN ('jwt_template', 'dev'))
);

-- +goose Down
DROP TABLE `sts_configuration`;
