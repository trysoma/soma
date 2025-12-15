"""
Durable MCP (Model Context Protocol) client for Restate.
Provides a client wrapper that makes all MCP operations replayable via Restate.
"""

import logging
import os
from contextlib import asynccontextmanager
from collections.abc import AsyncIterator
from typing import cast

logger = logging.getLogger(__name__)

from mcp import ClientSession
from mcp.client.streamable_http import streamablehttp_client
from mcp.types import (
    CallToolResult,
    EmptyResult,
    GetPromptResult,
    ListPromptsResult,
    ListResourcesResult,
    ListResourceTemplatesResult,
    ListToolsResult,
    ReadResourceResult,
)
from pydantic import AnyUrl
from restate import ObjectContext


class SomaMcpClientConfig:
    """Configuration options for creating a Soma MCP client."""

    def __init__(self, soma_base_url: str | None = None) -> None:
        """
        Initialize config.

        Args:
            soma_base_url: The base URL of the Soma server.
                          Defaults to SOMA_SERVER_BASE_URL env var or 'http://localhost:3000'.
        """
        self.soma_base_url = soma_base_url


def get_mcp_url(
    mcp_server_instance_id: str,
    config: SomaMcpClientConfig | None = None,
) -> str:
    """
    Get the MCP URL for a Soma MCP server instance.

    Args:
        mcp_server_instance_id: The ID of the MCP server instance to connect to.
        config: Optional configuration including base URL.

    Returns:
        The MCP server URL.
    """
    base_url = (
        config.soma_base_url
        if config and config.soma_base_url
        else os.environ.get("SOMA_SERVER_BASE_URL", "http://localhost:3000")
    )

    return f"{base_url}/api/bridge/v1/mcp-instance/{mcp_server_instance_id}/mcp"


class SomaMcpClient:
    """
    Durable MCP client wrapper that makes all MCP operations replayable via Restate.

    The client maintains a persistent connection to the MCP server and wraps
    all operations with ctx.run() for durability.
    """

    def __init__(
        self,
        ctx: ObjectContext,
        session: ClientSession,
        client_name: str,
    ) -> None:
        """
        Initialize the durable MCP client.

        Args:
            ctx: The Restate ObjectContext for durability
            session: The MCP ClientSession (must be initialized)
            client_name: A unique name for this client (used for durability keys)
        """
        self._ctx = ctx
        self._session = session
        self._client_name = client_name
        self._request_index = 0

    async def ping(self) -> EmptyResult:
        """Ping the MCP server (durable)."""
        logger.debug("Pinging MCP server %s", self._client_name)
        index = self._request_index
        self._request_index += 1

        async def do_ping() -> dict[str, object]:
            result = await self._session.send_ping()
            return cast(dict[str, object], result.model_dump())

        data = await self._ctx.run(
            f"mcp-{self._client_name}-ping-index-{index}", do_ping
        )
        logger.debug("Pinging MCP server %s completed", self._client_name)
        return EmptyResult.model_validate(data)

    async def list_tools(
        self,
        cursor: str | None = None,
    ) -> ListToolsResult:
        """List available tools (durable)."""
        logger.debug("Listing tools on MCP server %s", self._client_name)
        index = self._request_index
        self._request_index += 1

        async def do_list() -> dict[str, object]:
            result = await self._session.list_tools(cursor=cursor)
            return cast(dict[str, object], result.model_dump())

        data = await self._ctx.run(
            f"mcp-{self._client_name}-listTools-index-{index}", do_list
        )
        result = ListToolsResult.model_validate(data)
        logger.debug("Listing tools on MCP server %s completed (count=%d)", self._client_name, len(result.tools))
        return result

    async def call_tool(
        self,
        name: str,
        arguments: dict[str, object] | None = None,
    ) -> CallToolResult:
        """Call a tool on the MCP server (durable)."""
        logger.debug("Calling tool %s on MCP server %s", name, self._client_name)
        index = self._request_index
        self._request_index += 1

        async def do_call() -> dict[str, object]:
            result = await self._session.call_tool(name, arguments or {})
            return cast(dict[str, object], result.model_dump())

        data = await self._ctx.run(
            f"mcp-{self._client_name}-callTool-index-{index}", do_call
        )
        result = CallToolResult.model_validate(data)
        logger.debug("Calling tool %s on MCP server %s completed", name, self._client_name)
        return result

    async def list_resources(
        self,
        cursor: str | None = None,
    ) -> ListResourcesResult:
        """List available resources (durable)."""
        logger.debug("Listing resources on MCP server %s", self._client_name)
        index = self._request_index
        self._request_index += 1

        async def do_list() -> dict[str, object]:
            result = await self._session.list_resources(cursor=cursor)
            return cast(dict[str, object], result.model_dump())

        data = await self._ctx.run(
            f"mcp-{self._client_name}-listResources-index-{index}", do_list
        )
        result = ListResourcesResult.model_validate(data)
        logger.debug("Listing resources on MCP server %s completed", self._client_name)
        return result

    async def list_resource_templates(
        self,
        cursor: str | None = None,
    ) -> ListResourceTemplatesResult:
        """List available resource templates (durable)."""
        index = self._request_index
        self._request_index += 1

        async def do_list() -> dict[str, object]:
            result = await self._session.list_resource_templates(cursor=cursor)
            return cast(dict[str, object], result.model_dump())

        data = await self._ctx.run(
            f"mcp-{self._client_name}-listResourceTemplates-index-{index}", do_list
        )
        return ListResourceTemplatesResult.model_validate(data)

    async def read_resource(self, uri: str) -> ReadResourceResult:
        """Read a resource by URI (durable)."""
        logger.debug("Reading resource %s on MCP server %s", uri, self._client_name)
        index = self._request_index
        self._request_index += 1

        async def do_read() -> dict[str, object]:
            result = await self._session.read_resource(AnyUrl(uri))
            return cast(dict[str, object], result.model_dump())

        data = await self._ctx.run(
            f"mcp-{self._client_name}-readResource-index-{index}", do_read
        )
        result = ReadResourceResult.model_validate(data)
        logger.debug("Reading resource %s on MCP server %s completed", uri, self._client_name)
        return result

    async def subscribe_resource(self, uri: str) -> EmptyResult:
        """Subscribe to a resource for updates (durable)."""
        index = self._request_index
        self._request_index += 1

        async def do_subscribe() -> dict[str, object]:
            result = await self._session.subscribe_resource(AnyUrl(uri))
            return cast(dict[str, object], result.model_dump())

        data = await self._ctx.run(
            f"mcp-{self._client_name}-subscribeResource-index-{index}", do_subscribe
        )
        return EmptyResult.model_validate(data)

    async def unsubscribe_resource(self, uri: str) -> EmptyResult:
        """Unsubscribe from a resource (durable)."""
        index = self._request_index
        self._request_index += 1

        async def do_unsubscribe() -> dict[str, object]:
            result = await self._session.unsubscribe_resource(AnyUrl(uri))
            return cast(dict[str, object], result.model_dump())

        data = await self._ctx.run(
            f"mcp-{self._client_name}-unsubscribeResource-index-{index}", do_unsubscribe
        )
        return EmptyResult.model_validate(data)

    async def list_prompts(
        self,
        cursor: str | None = None,
    ) -> ListPromptsResult:
        """List available prompts (durable)."""
        index = self._request_index
        self._request_index += 1

        async def do_list() -> dict[str, object]:
            result = await self._session.list_prompts(cursor=cursor)
            return cast(dict[str, object], result.model_dump())

        data = await self._ctx.run(
            f"mcp-{self._client_name}-listPrompts-index-{index}", do_list
        )
        return ListPromptsResult.model_validate(data)

    async def get_prompt(
        self,
        name: str,
        arguments: dict[str, str] | None = None,
    ) -> GetPromptResult:
        """Get a prompt by name (durable)."""
        logger.debug("Getting prompt %s on MCP server %s", name, self._client_name)
        index = self._request_index
        self._request_index += 1

        async def do_get() -> dict[str, object]:
            result = await self._session.get_prompt(name, arguments)
            return cast(dict[str, object], result.model_dump())

        data = await self._ctx.run(
            f"mcp-{self._client_name}-getPrompt-index-{index}", do_get
        )
        result = GetPromptResult.model_validate(data)
        logger.debug("Getting prompt %s on MCP server %s completed", name, self._client_name)
        return result

    async def send_roots_list_changed(self) -> None:
        """Send roots list changed notification (durable)."""
        index = self._request_index
        self._request_index += 1

        async def do_send() -> None:
            await self._session.send_roots_list_changed()

        await self._ctx.run(
            f"mcp-{self._client_name}-sendRootsListChanged-index-{index}", do_send
        )


@asynccontextmanager
async def create_soma_mcp_client(
    ctx: ObjectContext,
    mcp_server_instance_id: str,
    config: SomaMcpClientConfig | None = None,
) -> AsyncIterator[SomaMcpClient]:
    """
    Create an MCP client connected to a Soma MCP server instance.

    This is an async context manager that maintains the connection for the
    duration of the context. Use it with `async with`:

    Args:
        ctx: The Restate ObjectContext for durability
        mcp_server_instance_id: The ID of the MCP server instance to connect to.
        config: Optional configuration including base URL.

    Yields:
        A connected SomaMcpClient instance.

    Example:
        ```python
        from trysoma_sdk.mcp import create_soma_mcp_client

        async with create_soma_mcp_client(ctx, 'my-mcp-instance-id') as client:
            # List available tools
            tools = await client.list_tools()

            # Call a tool
            result = await client.call_tool('my-tool', {'arg': 'value'})
        ```
    """
    logger.debug("Creating MCP client for instance %s", mcp_server_instance_id)
    mcp_url = get_mcp_url(mcp_server_instance_id, config)

    async with streamablehttp_client(mcp_url) as (
        read_stream,
        write_stream,
        _get_session_id,
    ):
        async with ClientSession(
            read_stream=read_stream,
            write_stream=write_stream,
        ) as session:
            await session.initialize()
            logger.debug("Creating MCP client for instance %s completed", mcp_server_instance_id)
            yield SomaMcpClient(ctx, session, mcp_server_instance_id)


# Re-export for convenience
__all__ = [
    # Main client
    "SomaMcpClient",
    "SomaMcpClientConfig",
    "create_soma_mcp_client",
    "get_mcp_url",
    # Re-exported types
    "CallToolResult",
    "ListToolsResult",
    "ListResourcesResult",
    "ListResourceTemplatesResult",
    "ReadResourceResult",
    "ListPromptsResult",
    "GetPromptResult",
    "EmptyResult",
]
