"""Soma API client implementation."""

from dataclasses import dataclass, field
from typing import Any

import httpx

from soma_api_client.models import (
    CreateMessageRequest,
    CreateMessageResponse,
    TaskTimelineResponse,
)


@dataclass
class Configuration:
    """API client configuration."""

    host: str = "http://localhost:3000"
    api_key: str | None = None
    timeout: float = 30.0
    headers: dict[str, str] = field(default_factory=dict)


class V1Api:
    """Soma V1 API client.

    This is a simplified client that mirrors the TypeScript API client.
    For a full-featured client, use openapi-generator to generate from openapi.json.
    """

    def __init__(self, configuration: Configuration | None = None) -> None:
        """Initialize the API client.

        Args:
            configuration: Optional configuration object.
        """
        self.config = configuration or Configuration()
        self._client: httpx.AsyncClient | None = None

    @property
    def client(self) -> httpx.AsyncClient:
        """Get or create the HTTP client."""
        if self._client is None:
            headers = {"Content-Type": "application/json", **self.config.headers}
            if self.config.api_key:
                headers["Authorization"] = f"Bearer {self.config.api_key}"

            self._client = httpx.AsyncClient(
                base_url=self.config.host,
                headers=headers,
                timeout=self.config.timeout,
            )
        return self._client

    async def close(self) -> None:
        """Close the HTTP client."""
        if self._client:
            await self._client.aclose()
            self._client = None

    async def __aenter__(self) -> "V1Api":
        """Enter async context."""
        return self

    async def __aexit__(self, *args: Any) -> None:
        """Exit async context."""
        await self.close()

    # Task endpoints

    async def task_history(
        self,
        task_id: str,
        page_size: int = 100,
        page_token: str | None = None,
    ) -> TaskTimelineResponse:
        """Get task timeline/history.

        Args:
            task_id: The task ID.
            page_size: Maximum number of items to return.
            page_token: Optional pagination token.

        Returns:
            TaskTimelineResponse with items and pagination info.
        """
        params: dict[str, Any] = {"pageSize": page_size}
        if page_token:
            params["pageToken"] = page_token

        response = await self.client.get(
            f"/api/v1/tasks/{task_id}/timeline",
            params=params,
        )
        response.raise_for_status()
        data = response.json()
        return TaskTimelineResponse(
            items=data.get("items", []),
            next_page_token=data.get("nextPageToken"),
        )

    async def send_message(
        self,
        task_id: str,
        create_message_request: CreateMessageRequest | dict[str, Any],
    ) -> CreateMessageResponse:
        """Send a message to a task.

        Args:
            task_id: The task ID.
            create_message_request: The message to send.

        Returns:
            CreateMessageResponse with the created message.
        """
        if isinstance(create_message_request, CreateMessageRequest):
            body = create_message_request.model_dump()
        else:
            body = create_message_request

        response = await self.client.post(
            f"/api/v1/tasks/{task_id}/messages",
            json=body,
        )
        response.raise_for_status()
        return CreateMessageResponse(**response.json())

    async def update_task_status(
        self,
        task_id: str,
        update_task_status_request: dict[str, Any],
    ) -> dict[str, Any]:
        """Update task status.

        Args:
            task_id: The task ID.
            update_task_status_request: The status update request.

        Returns:
            The updated task.
        """
        response = await self.client.patch(
            f"/api/v1/tasks/{task_id}/status",
            json=update_task_status_request,
        )
        response.raise_for_status()
        return response.json()

    async def get_task(self, task_id: str) -> dict[str, Any]:
        """Get a task by ID.

        Args:
            task_id: The task ID.

        Returns:
            The task data.
        """
        response = await self.client.get(f"/api/v1/tasks/{task_id}")
        response.raise_for_status()
        return response.json()

    async def list_tasks(
        self,
        context_id: str | None = None,
        page_size: int = 100,
        page_token: str | None = None,
    ) -> dict[str, Any]:
        """List tasks.

        Args:
            context_id: Optional context ID to filter by.
            page_size: Maximum number of items to return.
            page_token: Optional pagination token.

        Returns:
            Paginated list of tasks.
        """
        params: dict[str, Any] = {"pageSize": page_size}
        if context_id:
            params["contextId"] = context_id
        if page_token:
            params["pageToken"] = page_token

        response = await self.client.get("/api/v1/tasks", params=params)
        response.raise_for_status()
        return response.json()

    # Context endpoints

    async def get_context(self, context_id: str) -> dict[str, Any]:
        """Get a context by ID.

        Args:
            context_id: The context ID.

        Returns:
            The context data.
        """
        response = await self.client.get(f"/api/v1/contexts/{context_id}")
        response.raise_for_status()
        return response.json()

    async def list_contexts(
        self,
        page_size: int = 100,
        page_token: str | None = None,
    ) -> dict[str, Any]:
        """List contexts.

        Args:
            page_size: Maximum number of items to return.
            page_token: Optional pagination token.

        Returns:
            Paginated list of contexts.
        """
        params: dict[str, Any] = {"pageSize": page_size}
        if page_token:
            params["pageToken"] = page_token

        response = await self.client.get("/api/v1/contexts", params=params)
        response.raise_for_status()
        return response.json()

    # Agent endpoints

    async def list_agents(
        self,
        page_size: int = 100,
        page_token: str | None = None,
    ) -> dict[str, Any]:
        """List agents.

        Args:
            page_size: Maximum number of items to return.
            page_token: Optional pagination token.

        Returns:
            Paginated list of agents.
        """
        params: dict[str, Any] = {"pageSize": page_size}
        if page_token:
            params["pageToken"] = page_token

        response = await self.client.get("/api/v1/agents", params=params)
        response.raise_for_status()
        return response.json()

    async def get_agent(self, agent_id: str) -> dict[str, Any]:
        """Get an agent by ID.

        Args:
            agent_id: The agent ID.

        Returns:
            The agent data.
        """
        response = await self.client.get(f"/api/v1/agents/{agent_id}")
        response.raise_for_status()
        return response.json()
