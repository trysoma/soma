//! Business logic for Vercel AI SDK message generation
//!
//! This module contains the core logic for generating messages in UIMessage and TextMessage
//! formats compatible with the Vercel AI SDK.

mod generate;

pub use generate::{
    GenerateParams, GenerateResponse, GenerateTextParams, GenerateTextResponse, GenerateUiParams,
    GenerateUiResponse, StreamItem, TextStreamItem, UiStreamItem,
};
