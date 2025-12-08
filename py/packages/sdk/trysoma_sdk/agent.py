"""Agent creation and management for Soma SDK."""

from dataclasses import dataclass
from typing import Awaitable, Callable, Protocol

from restate import ObjectContext
from trysoma_api_client import V1Api as SomaV1Api


@dataclass
class HandlerParams:
    """Parameters passed to agent handlers."""

    task_id: str
    context_id: str
    ctx: ObjectContext
    soma: SomaV1Api


class SomaAgent(Protocol):
    """Protocol for Soma agents."""

    project_id: str
    agent_id: str
    name: str
    description: str

    async def entrypoint(self, params: HandlerParams) -> None:
        """Main entry point for the agent."""
        ...


@dataclass
class _SomaAgentImpl:
    """Implementation of SomaAgent."""

    project_id: str
    agent_id: str
    name: str
    description: str
    _entrypoint: Callable[[HandlerParams], Awaitable[None]]

    async def entrypoint(self, params: HandlerParams) -> None:
        """Main entry point for the agent."""
        await self._entrypoint(params)


def create_soma_agent(
    *,
    project_id: str,
    agent_id: str,
    name: str,
    description: str,
    entrypoint: Callable[[HandlerParams], Awaitable[None]],
) -> SomaAgent:
    """Create a new Soma agent.

    Args:
        project_id: The project ID this agent belongs to.
        agent_id: Unique identifier for this agent.
        name: Human-readable name for the agent.
        description: Description of what this agent does.
        entrypoint: Async function that handles agent invocations.

    Returns:
        A SomaAgent instance.

    Example:
        ```python
        async def handle_claim(params: HandlerParams) -> None:
            # Process the claim
            pass

        agent = create_soma_agent(
            project_id="acme",
            agent_id="claims-agent",
            name="Claims Agent",
            description="Processes insurance claims",
            entrypoint=handle_claim,
        )
        ```
    """
    return _SomaAgentImpl(
        project_id=project_id,
        agent_id=agent_id,
        name=name,
        description=description,
        _entrypoint=entrypoint,
    )
