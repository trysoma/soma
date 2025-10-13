use rmcp::model::{CallToolResult, ErrorData};
use serde::Serialize;
pub trait McpErrorMsg {
    fn to_mcp_error(&self) -> String;
}

pub struct StructuredResponse<T, E: Serialize + McpErrorMsg>(Result<T, E>);

impl<T: Serialize, E: Serialize + McpErrorMsg> StructuredResponse<T, E> {
    pub fn new_error(error: E) -> Self {
        Self(Err(error))
    }

    pub fn new_ok(value: T) -> Self {
        Self(Ok(value))
    }

    pub fn new(result: Result<T, E>) -> Self {
        Self(result)
    }
}

impl<T: Serialize, E: Serialize + McpErrorMsg + Into<ErrorData>> StructuredResponse<T, E> {
    pub fn into_response(self) -> Result<CallToolResult, ErrorData> {
        match self.0 {
            Ok(value) => {
                let value = serde_json::to_value(value)
                    .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
                Ok(CallToolResult::structured(value))
            }
            Err(error) => Err(error.into()),
        }
    }
}

impl<T: Serialize, E: Serialize + McpErrorMsg + Into<ErrorData>> From<StructuredResponse<T, E>>
    for Result<CallToolResult, ErrorData>
{
    fn from(value: StructuredResponse<T, E>) -> Self {
        value.into_response()
    }
}
