use std::{ops::Deref, sync::Arc};

use derive_builder::Builder;

use crate::{request_handlers::request_handler::RequestHandler, types::AgentCard};

#[derive(Debug, Clone)]
pub struct RequestContext {
    pub request_uri: http::Uri,
    pub headers: http::HeaderMap,
}

pub trait A2aServiceLike {
    fn agent_card(&self, context: RequestContext) -> AgentCard;
    fn extended_agent_card(&self, context: RequestContext) -> Option<AgentCard>;
    fn request_handler(&self, context: RequestContext) -> Arc<dyn RequestHandler + Send + Sync>;
}

#[derive(Builder, Clone)]
#[builder(pattern = "owned")]
pub struct A2aService {
    agent_card: Arc<AgentCard>,
    extended_agent_card: Arc<Option<AgentCard>>,
    request_handler: Arc<dyn RequestHandler + Send + Sync>,
}

impl A2aServiceLike for A2aService {
    fn request_handler(&self, _context: RequestContext) -> Arc<dyn RequestHandler + Send + Sync> {
        self.request_handler.clone()
    }

    fn agent_card(&self, _context: RequestContext) -> AgentCard {
        self.agent_card.deref().clone()
    }

    fn extended_agent_card(&self, _context: RequestContext) -> Option<AgentCard> {
        self.extended_agent_card.deref().clone()
    }
}
