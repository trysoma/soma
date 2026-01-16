use std::sync::Arc;

use async_trait::async_trait;
use tool::logic::ToolLike;
use tool::logic::Metadata;
use tool::logic::ToolGroupLike;
use tool::logic::CredentialSourceLike;
use tool::logic::no_auth::NoAuthSource;
use tool::logic::no_auth::NoAuthStaticCredentialConfiguration;

use crate::repository::Repository;

/// Soma provider controller that provides soma-specific functions
pub struct SomaProviderController {
    _repository: Repository,
}

impl SomaProviderController {
    pub fn new(repository: Repository) -> Self {
        Self {
            _repository: repository,
        }
    }
}

#[async_trait]
impl ToolGroupLike for SomaProviderController {
    fn type_id(&self) -> String {
        "soma".to_string()
    }

    fn documentation(&self) -> String {
        "".to_string()
    }

    fn name(&self) -> String {
        "Soma".to_string()
    }

    fn categories(&self) -> Vec<String> {
        vec![]
    }

    fn tools(&self) -> Vec<Arc<dyn ToolLike>> {
        vec![]
    }

    fn credential_sources(&self) -> Vec<Arc<dyn CredentialSourceLike>> {
        vec![Arc::new(NoAuthSource {
            static_credentials: NoAuthStaticCredentialConfiguration {
                metadata: Metadata::new(),
            },
        })]
    }

    fn metadata(&self) -> Metadata {
        Metadata::new()
    }
}
