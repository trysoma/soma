// Encryption logic module for managing envelope and data encryption keys
// This module provides high-level operations for encryption key management

pub mod crypto_services;
pub mod dek;
pub mod dek_alias;
pub mod envelope;

pub use crypto_services::*;
pub use dek::*;
pub use dek_alias::*;
pub use envelope::*;

// Event types for encryption key changes
#[derive(Clone, Debug)]
pub enum EncryptionKeyEvent {
    EnvelopeEncryptionKeyAdded(EnvelopeEncryptionKey),
    EnvelopeEncryptionKeyRemoved(String), // ID of removed key
    DataEncryptionKeyAdded(DataEncryptionKey),
    DataEncryptionKeyRemoved(String), // ID of removed DEK
    DataEncryptionKeyMigrated {
        old_dek_id: String,
        new_dek_id: String,
        from_envelope_key: EnvelopeEncryptionKey,
        to_envelope_key: EnvelopeEncryptionKey,
        aliases: Vec<String>, // Aliases that were copied from old DEK to new DEK
    },
    DataEncryptionKeyAliasAdded {
        alias: String,
        dek: DataEncryptionKey,
    },
    DataEncryptionKeyAliasRemoved {
        alias: String,
    },
    DataEncryptionKeyAliasUpdated {
        alias: String,
        dek: DataEncryptionKey,
    },
}

pub type EncryptionKeyEventSender = tokio::sync::broadcast::Sender<EncryptionKeyEvent>;
pub type EncryptionKeyEventReceiver = tokio::sync::broadcast::Receiver<EncryptionKeyEvent>;
