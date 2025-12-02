-- +goose Up
-- create "jwt_signing_key" table
CREATE TABLE `jwt_signing_key` (
  `kid` text NOT NULL,
  `encrypted_private_key` text NOT NULL,
  `expires_at` datetime NOT NULL,
  `public_key` text NOT NULL,
  `dek_alias` text NOT NULL,
  `created_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  `updated_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  PRIMARY KEY (`kid`)
);

-- +goose Down
-- reverse: create "jwt_signing_key" table
DROP TABLE `jwt_signing_key`;

