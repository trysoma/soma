"""Basic tests for trysoma_sdk package."""


def test_import_sdk() -> None:
    """Test that the SDK can be imported."""
    from trysoma_sdk import create_soma_agent, HandlerParams

    assert create_soma_agent is not None
    assert HandlerParams is not None


def test_import_patterns() -> None:
    """Test that patterns module can be imported."""
    from trysoma_sdk import patterns

    assert patterns.chat is not None
    assert patterns.workflow is not None


def test_import_bridge() -> None:
    """Test that bridge module can be imported."""
    from trysoma_sdk.bridge import create_soma_function, SomaFunction

    assert create_soma_function is not None
    assert SomaFunction is not None


def test_import_standalone() -> None:
    """Test that standalone module can be imported."""
    from trysoma_sdk.standalone import generate_standalone, watch_and_regenerate

    assert generate_standalone is not None
    assert watch_and_regenerate is not None


class TestSomaAgent:
    """Tests for SomaAgent class."""

    def test_create_agent_with_minimal_params(self) -> None:
        """Test creating an agent with minimal parameters."""
        from trysoma_sdk import create_soma_agent, HandlerParams

        async def dummy_entrypoint(params: HandlerParams) -> None:
            pass

        agent = create_soma_agent(
            project_id="test-project",
            agent_id="test-agent",
            name="Test Agent",
            description="A test agent",
            entrypoint=dummy_entrypoint,
        )

        assert agent.project_id == "test-project"
        assert agent.agent_id == "test-agent"
        assert agent.name == "Test Agent"
        assert agent.description == "A test agent"


class TestPatterns:
    """Tests for pattern decorators."""

    def test_chat_pattern_wraps_handler(self) -> None:
        """Test that chat pattern wraps a handler function."""
        from trysoma_sdk import patterns
        from trysoma_sdk.patterns import ChatHandlerParams
        from typing import Any

        async def my_handler(params: ChatHandlerParams[Any, Any, str]) -> None:
            params.on_goal_achieved("done")

        wrapped = patterns.chat(my_handler)
        assert callable(wrapped)

    def test_workflow_pattern_wraps_handler(self) -> None:
        """Test that workflow pattern wraps a handler function."""
        from trysoma_sdk import patterns
        from trysoma_sdk.patterns import WorkflowHandlerParams
        from typing import Any

        async def my_handler(params: WorkflowHandlerParams[Any, Any, None]) -> None:
            pass

        wrapped = patterns.workflow(my_handler)
        assert callable(wrapped)
