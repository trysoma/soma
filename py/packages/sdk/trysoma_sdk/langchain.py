"""
LangChain MCP adapter for Soma.
Provides LangChain-compatible tools from Soma MCP servers with Restate durability.
"""

import json
from collections.abc import AsyncIterator, Awaitable, Callable
from contextlib import asynccontextmanager
from typing import Any

from langchain_core.tools import BaseTool, StructuredTool  # type: ignore[import-not-found]
from mcp.types import CallToolResult, Tool as McpTool
from pydantic import BaseModel, create_model
from restate import ObjectContext

from trysoma_sdk.mcp import (
    GetPromptResult,
    ListPromptsResult,
    ListResourcesResult,
    ListResourceTemplatesResult,
    ReadResourceResult,
    SomaMcpClient,
    SomaMcpClientConfig,
    create_soma_mcp_client,
)


def _json_schema_to_pydantic_model(
    name: str, schema: dict[str, Any]
) -> type[BaseModel]:
    """
    Convert a JSON schema to a Pydantic model.

    Args:
        name: The name for the model class
        schema: JSON schema dict with 'properties' and 'required' fields

    Returns:
        A dynamically created Pydantic model class
    """
    properties = schema.get("properties", {})
    required = set(schema.get("required", []))

    field_definitions: dict[str, Any] = {}

    for field_name, field_schema in properties.items():
        field_type = _json_type_to_python(field_schema)
        if field_name in required:
            field_definitions[field_name] = (field_type, ...)
        else:
            field_definitions[field_name] = (field_type | None, None)

    return create_model(name, **field_definitions)


def _json_type_to_python(schema: dict[str, Any]) -> type[Any]:
    """Convert JSON schema type to Python type."""
    json_type = schema.get("type", "string")

    if json_type == "string":
        return str
    elif json_type == "integer":
        return int
    elif json_type == "number":
        return float
    elif json_type == "boolean":
        return bool
    elif json_type == "array":
        return list
    elif json_type == "object":
        # For nested objects, just use dict
        return dict
    else:
        return object  # Fallback to object for unknown types


def _mcp_tool_to_langchain_tool(
    mcp_tool: McpTool,
    call_tool_fn: Callable[[str, dict[str, Any]], Awaitable[CallToolResult]],
) -> BaseTool:
    """
    Convert an MCP tool to a LangChain StructuredTool.

    Args:
        mcp_tool: The MCP tool definition
        call_tool_fn: Async function to call the tool

    Returns:
        A LangChain StructuredTool
    """
    # Create input schema model from MCP tool's input schema
    input_schema = mcp_tool.inputSchema
    schema_dict = (
        input_schema if isinstance(input_schema, dict) else input_schema.model_dump()
    )
    args_schema = _json_schema_to_pydantic_model(f"{mcp_tool.name}Input", schema_dict)

    async def _tool_func(**kwargs: Any) -> str:
        """Execute the MCP tool and return the result."""
        result = await call_tool_fn(mcp_tool.name, kwargs)

        # Extract text content from the result
        if result.content:
            texts: list[str] = []
            for item in result.content:
                if hasattr(item, "text"):
                    text_value = getattr(item, "text", None)
                    if text_value:
                        texts.append(str(text_value))
            if texts:
                return "\n".join(texts)

        # If structured content is available, return it as JSON
        if result.structuredContent is not None:
            return json.dumps(result.structuredContent)

        # Fallback: return the full result as JSON
        return json.dumps(result.model_dump())

    return StructuredTool(
        name=mcp_tool.name,
        description=mcp_tool.description or f"Tool: {mcp_tool.name}",
        args_schema=args_schema,
        coroutine=_tool_func,
    )


class SomaLangchainMcpClient:
    """
    LangChain MCP client wrapper that provides LangChain-compatible tools
    from Soma MCP servers with Restate durability.

    This is the Python equivalent of the TypeScript SomaVercelAiSdkMcpClient.

    Example:
        ```python
        from trysoma_sdk.langchain import create_soma_langchain_mcp_client

        # Create the client
        mcp_client = await create_soma_langchain_mcp_client(
            ctx,
            mcp_server_instance_id="test",
        )

        # Get LangChain tools
        tools = await mcp_client.get_tools()

        # Use with a LangChain agent
        from langchain_openai import ChatOpenAI
        model = ChatOpenAI(model="gpt-4o")
        model_with_tools = model.bind_tools(tools)
        ```
    """

    def __init__(self, mcp_client: SomaMcpClient) -> None:
        """
        Initialize the LangChain MCP client.

        Args:
            mcp_client: The underlying SomaMcpClient instance
        """
        self._mcp_client = mcp_client

    async def get_tools(self) -> list[BaseTool]:
        """
        Get LangChain-compatible tools from the MCP server.

        Returns:
            A list of LangChain BaseTool instances
        """
        # List tools from MCP server (durable)
        result = await self._mcp_client.list_tools()

        tools: list[BaseTool] = []
        for mcp_tool in result.tools:
            # Create a closure that captures the tool name for each tool
            tool = _mcp_tool_to_langchain_tool(
                mcp_tool,
                self._mcp_client.call_tool,
            )
            tools.append(tool)

        return tools

    async def list_resources(self, cursor: str | None = None) -> ListResourcesResult:
        """List available resources from the MCP server."""
        return await self._mcp_client.list_resources(cursor=cursor)

    async def read_resource(self, uri: str) -> ReadResourceResult:
        """Read a resource by URI from the MCP server."""
        return await self._mcp_client.read_resource(uri)

    async def list_resource_templates(
        self, cursor: str | None = None
    ) -> ListResourceTemplatesResult:
        """List available resource templates from the MCP server."""
        return await self._mcp_client.list_resource_templates(cursor=cursor)

    async def list_prompts(self, cursor: str | None = None) -> ListPromptsResult:
        """List available prompts from the MCP server."""
        return await self._mcp_client.list_prompts(cursor=cursor)

    async def get_prompt(
        self, name: str, arguments: dict[str, str] | None = None
    ) -> GetPromptResult:
        """Get a prompt by name from the MCP server."""
        return await self._mcp_client.get_prompt(name, arguments)


@asynccontextmanager
async def create_soma_langchain_mcp_client(
    ctx: ObjectContext,
    mcp_server_instance_id: str,
    config: SomaMcpClientConfig | None = None,
) -> AsyncIterator[SomaLangchainMcpClient]:
    """
    Create a LangChain MCP client connected to a Soma MCP server instance.

    This is an async context manager that maintains the connection for the
    duration of the context. Use it with `async with`:

    Args:
        ctx: The Restate ObjectContext for durability
        mcp_server_instance_id: The ID of the MCP server instance to connect to
        config: Optional configuration including base URL

    Yields:
        A SomaLangchainMcpClient instance

    Example:
        ```python
        from trysoma_sdk.langchain import create_soma_langchain_mcp_client
        from langchain_openai import ChatOpenAI

        # Create the MCP client using context manager
        async with create_soma_langchain_mcp_client(
            ctx,
            mcp_server_instance_id="test",
        ) as mcp_client:
            # Get tools as LangChain tools
            tools = await mcp_client.get_tools()

            # Use with a chat model
            model = ChatOpenAI(model="gpt-4o")
            model_with_tools = model.bind_tools(tools)

            # Invoke the model
            response = await model_with_tools.ainvoke(messages)
        ```
    """
    # Create the underlying durable MCP client using context manager
    async with create_soma_mcp_client(
        ctx, mcp_server_instance_id, config
    ) as mcp_client:
        yield SomaLangchainMcpClient(mcp_client)


# Re-export for convenience
__all__ = [
    "SomaLangchainMcpClient",
    "create_soma_langchain_mcp_client",
]
