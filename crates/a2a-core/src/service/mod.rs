use std::{ops::Deref, sync::Arc};

use crate::{
    errors::A2aServerError, request_handlers::request_handler::RequestHandler, types::AgentCard,
};
use async_trait::async_trait;
use derive_builder::Builder;

#[derive(Debug, Clone)]
pub struct RequestContext {
    pub request_uri: http::Uri,
    pub headers: http::HeaderMap,
}

#[async_trait]
pub trait A2aServiceLike: Send + Sync {
    async fn agent_card(&self, context: RequestContext) -> Result<AgentCard, A2aServerError>;
    async fn extended_agent_card(
        &self,
        context: RequestContext,
    ) -> Result<Option<AgentCard>, A2aServerError>;
    fn request_handler(&self, context: RequestContext) -> Arc<dyn RequestHandler + Send + Sync>;
}

#[derive(Builder, Clone)]
#[builder(pattern = "owned")]
pub struct A2aService {
    agent_card: Arc<AgentCard>,
    extended_agent_card: Arc<Option<AgentCard>>,
    request_handler: Arc<dyn RequestHandler + Send + Sync>,
}

#[async_trait]
impl A2aServiceLike for A2aService {
    fn request_handler(&self, _context: RequestContext) -> Arc<dyn RequestHandler + Send + Sync> {
        self.request_handler.clone()
    }

    async fn agent_card(&self, _context: RequestContext) -> Result<AgentCard, A2aServerError> {
        Ok(self.agent_card.deref().clone())
    }

    async fn extended_agent_card(
        &self,
        _context: RequestContext,
    ) -> Result<Option<AgentCard>, A2aServerError> {
        Ok(self.extended_agent_card.deref().clone())
    }
}
