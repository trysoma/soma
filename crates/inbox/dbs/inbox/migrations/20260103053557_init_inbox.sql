-- +goose Up
-- create "thread" table
CREATE TABLE `thread` (
  `id` text NULL,
  `title` text NULL,
  `metadata` json NULL,
  `inbox_settings` json NOT NULL DEFAULT '{}',
  `created_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  `updated_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  PRIMARY KEY (`id`)
);
-- create index "idx_thread_created_at" to table: "thread"
CREATE INDEX `idx_thread_created_at` ON `thread` (`created_at`);
-- create "message" table
CREATE TABLE `message` (
  `id` text NULL,
  `thread_id` text NOT NULL,
  `kind` text NOT NULL,
  `role` text NOT NULL,
  `body` json NOT NULL,
  `metadata` json NULL,
  `inbox_settings` json NOT NULL DEFAULT '{}',
  `created_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  `updated_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  PRIMARY KEY (`id`),
  CONSTRAINT `0` FOREIGN KEY (`thread_id`) REFERENCES `thread` (`id`) ON UPDATE NO ACTION ON DELETE CASCADE,
  CONSTRAINT `kind_check` CHECK (kind IN ('text', 'ui')),
  CONSTRAINT `role_check` CHECK (role IN ('system', 'user', 'assistant'))
);
-- create index "idx_message_thread_id" to table: "message"
CREATE INDEX `idx_message_thread_id` ON `message` (`thread_id`);
-- create index "idx_message_created_at" to table: "message"
CREATE INDEX `idx_message_created_at` ON `message` (`created_at`);
-- create index "idx_message_thread_created" to table: "message"
CREATE INDEX `idx_message_thread_created` ON `message` (`thread_id`, `created_at`);
-- create "event" table
CREATE TABLE `event` (
  `id` text NULL,
  `kind` text NOT NULL,
  `payload` json NOT NULL,
  `inbox_id` text NULL,
  `inbox_settings` json NOT NULL DEFAULT '{}',
  `created_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  PRIMARY KEY (`id`)
);
-- create index "idx_event_created_at" to table: "event"
CREATE INDEX `idx_event_created_at` ON `event` (`created_at`);
-- create index "idx_event_inbox_id" to table: "event"
CREATE INDEX `idx_event_inbox_id` ON `event` (`inbox_id`);
-- create index "idx_event_kind" to table: "event"
CREATE INDEX `idx_event_kind` ON `event` (`kind`);
-- create "inbox" table
CREATE TABLE `inbox` (
  `id` text NULL,
  `provider_id` text NOT NULL,
  `destination_type` text NOT NULL,
  `destination_id` text NOT NULL,
  `configuration` json NOT NULL,
  `settings` json NOT NULL DEFAULT '{}',
  `created_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  `updated_at` datetime NOT NULL DEFAULT (CURRENT_TIMESTAMP),
  PRIMARY KEY (`id`),
  CONSTRAINT `destination_type_check` CHECK (destination_type IN ('agent', 'workflow'))
);
-- create index "idx_inbox_provider_id" to table: "inbox"
CREATE INDEX `idx_inbox_provider_id` ON `inbox` (`provider_id`);
-- create index "idx_inbox_destination" to table: "inbox"
CREATE INDEX `idx_inbox_destination` ON `inbox` (`destination_type`, `destination_id`);

-- +goose Down
-- reverse: create index "idx_inbox_destination" to table: "inbox"
DROP INDEX `idx_inbox_destination`;
-- reverse: create index "idx_inbox_provider_id" to table: "inbox"
DROP INDEX `idx_inbox_provider_id`;
-- reverse: create "inbox" table
DROP TABLE `inbox`;
-- reverse: create index "idx_event_kind" to table: "event"
DROP INDEX `idx_event_kind`;
-- reverse: create index "idx_event_inbox_id" to table: "event"
DROP INDEX `idx_event_inbox_id`;
-- reverse: create index "idx_event_created_at" to table: "event"
DROP INDEX `idx_event_created_at`;
-- reverse: create "event" table
DROP TABLE `event`;
-- reverse: create index "idx_message_thread_created" to table: "message"
DROP INDEX `idx_message_thread_created`;
-- reverse: create index "idx_message_created_at" to table: "message"
DROP INDEX `idx_message_created_at`;
-- reverse: create index "idx_message_thread_id" to table: "message"
DROP INDEX `idx_message_thread_id`;
-- reverse: create "message" table
DROP TABLE `message`;
-- reverse: create index "idx_thread_created_at" to table: "thread"
DROP INDEX `idx_thread_created_at`;
-- reverse: create "thread" table
DROP TABLE `thread`;
