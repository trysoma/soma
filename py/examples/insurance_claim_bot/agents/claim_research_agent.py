"""Claim Research Agent - Agent that researches insurance claims using MCP tools."""

from typing import Any

from langchain_openai import ChatOpenAI  # type: ignore[import-not-found]
from langchain_core.messages import (  # type: ignore[import-not-found]
    HumanMessage,
    AIMessage,
    SystemMessage,
    BaseMessage,
    ToolMessage,
)
from pydantic import BaseModel

from trysoma_sdk import create_soma_agent, HandlerParams, patterns
from trysoma_sdk.patterns import (
    ChatHandlerParams,
    WrappedChatHandlerParams,
)
from trysoma_sdk.langchain import create_soma_langchain_mcp_client
from trysoma_api_client import MessageRole, TaskStatus
from trysoma_api_client.models.create_message_request import CreateMessageRequest
from trysoma_api_client.models.message_part import MessagePart
from trysoma_api_client.models.update_task_status_request import UpdateTaskStatusRequest

# Normal imports - project root and soma are added to sys.path by standalone.py
from utils import convert_to_openai_messages
from soma.bridge import Bridge, get_bridge

# System prompt for the research agent
SYSTEM_PROMPT = """You are a research agent that can research insurance claims.
You are given a claim and you need to research it and return a summary of the research.

Use the available tools to research the claim. When you have gathered enough information,
use the output_research tool to summarize your findings."""


class ClaimResearchInput(BaseModel):
    """Input for claim research - empty as input comes from conversation."""

    class Config:
        arbitrary_types_allowed = True


class ClaimResearchOutput(BaseModel):
    """Output from claim research."""

    summary: str


async def claim_research_handler(
    params: ChatHandlerParams[Bridge, ClaimResearchInput, ClaimResearchOutput],
) -> None:
    """Handler for researching claims using MCP tools.

    This handler processes a single turn of the conversation. The chat pattern
    wrapper will call this repeatedly until on_goal_achieved is called.
    """
    # Convert Soma history to OpenAI message format
    openai_messages = convert_to_openai_messages(params.history)

    # Build LangChain messages with system prompt
    langchain_messages: list[BaseMessage] = [SystemMessage(content=SYSTEM_PROMPT)]

    for msg in openai_messages:
        role = msg.get("role", "user")
        content: str = str(msg.get("content", ""))
        if role == "user":
            langchain_messages.append(HumanMessage(content=content))
        elif role == "assistant":
            langchain_messages.append(AIMessage(content=content))

    print(f"LangChain messages count: {len(langchain_messages)}")

    # Create MCP client and load tools using context manager
    async with create_soma_langchain_mcp_client(
        params.ctx,
        mcp_server_instance_id="test",
    ) as mcp_client:
        mcp_tools = await mcp_client.get_tools()
        print(f"Loaded {len(mcp_tools)} MCP tools")

        # Define the output tool for structured output extraction
        # This is the "goal achieved" tool that signals completion
        output_research_tool = {
            "type": "function",
            "function": {
                "name": "output_research",
                "description": "Summarize your findings into a final output when you have completed your research.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "summary": {
                            "type": "string",
                            "description": "A summary of the research findings about the claim",
                        }
                    },
                    "required": ["summary"],
                },
            },
        }

        # Convert MCP tools to OpenAI tool format
        all_tools = [output_research_tool]
        for tool in mcp_tools:
            tool_dict = {
                "type": "function",
                "function": {
                    "name": tool.name,
                    "description": tool.description or f"Tool: {tool.name}",
                    "parameters": tool.args_schema.model_json_schema()
                    if hasattr(tool, "args_schema")
                    else {"type": "object", "properties": {}},
                },
            }
            all_tools.append(tool_dict)

        # Create LangChain ChatOpenAI model with tools
        model = ChatOpenAI(
            model="gpt-4o",
            temperature=0,
        )
        model_with_tools = model.bind(tools=all_tools)

        try:
            # Invoke the model for this turn
            response: Any = await model_with_tools.ainvoke(langchain_messages)

            # Check if the model called any tools
            if hasattr(response, "tool_calls") and response.tool_calls:
                for tool_call in response.tool_calls:
                    tool_name = tool_call.get("name", "")
                    tool_args = tool_call.get("args", {})
                    tool_id = tool_call.get("id", "")

                    print(f"Tool called: {tool_name} with args: {tool_args}")

                    if tool_name == "output_research":
                        # Goal achieved - extract the summary and signal completion
                        summary = tool_args.get("summary", "")
                        print(f"Research completed with summary: {summary}")

                        output = ClaimResearchOutput(summary=summary)
                        params.on_goal_achieved(output)
                        return

                    # Execute MCP tool
                    tool_result = None
                    for mcp_tool in mcp_tools:
                        if mcp_tool.name == tool_name:
                            try:
                                tool_result = await mcp_tool.ainvoke(tool_args)
                                print(f"Tool result: {tool_result}")
                            except Exception as e:
                                tool_result = f"Error executing tool: {e}"
                            break

                    if tool_result is None:
                        tool_result = f"Unknown tool: {tool_name}"

                    # Add tool result to messages and continue
                    langchain_messages.append(response)
                    langchain_messages.append(
                        ToolMessage(
                            content=str(tool_result),
                            tool_call_id=tool_id,
                        )
                    )

                    # Get the model's response after tool execution
                    follow_up: Any = await model_with_tools.ainvoke(langchain_messages)

                    # Check if follow-up calls output_research
                    if hasattr(follow_up, "tool_calls") and follow_up.tool_calls:
                        for fc in follow_up.tool_calls:
                            if fc.get("name") == "output_research":
                                summary = fc.get("args", {}).get("summary", "")
                                print(f"Research completed with summary: {summary}")
                                output = ClaimResearchOutput(summary=summary)
                                params.on_goal_achieved(output)
                                return

                    # Send the response back to user
                    if hasattr(follow_up, "content") and follow_up.content:
                        await params.send_message(
                            CreateMessageRequest(
                                metadata={},
                                parts=[
                                    MessagePart(
                                        metadata={},
                                        type="text-part",
                                        text=str(follow_up.content),
                                    )
                                ],
                                reference_task_ids=[],
                                role=MessageRole.AGENT,
                            )
                        )
            else:
                # No tool call - send the assistant's response back to the user
                response_content: str = ""
                if hasattr(response, "content"):
                    response_content = str(response.content)

                if response_content:
                    print(f"Assistant response: {response_content}")
                    await params.send_message(
                        CreateMessageRequest(
                            metadata={},
                            parts=[
                                MessagePart(
                                    metadata={},
                                    type="text-part",
                                    text=response_content,
                                )
                            ],
                            reference_task_ids=[],
                            role=MessageRole.AGENT,
                        )
                    )

        except Exception as e:
            print(f"Error in claim_research_handler: {e}")
            await params.send_message(
                CreateMessageRequest(
                    metadata={},
                    parts=[
                        MessagePart(
                            metadata={},
                            type="text-part",
                            text=f"I encountered an error while researching: {e}",
                        )
                    ],
                    reference_task_ids=[],
                    role=MessageRole.AGENT,
                )
            )


# Create wrapped handler using patterns
claim_research = patterns.chat(claim_research_handler)


async def entrypoint(params: HandlerParams) -> None:
    """Main agent entrypoint."""
    # Get bridge instance
    bridge = get_bridge(params.ctx)

    print("Starting claim research agent...")

    # Research the claim through conversation with MCP tools
    research_output = await claim_research(
        WrappedChatHandlerParams(
            ctx=params.ctx,
            soma=params.soma,
            bridge=bridge,
            input=ClaimResearchInput(),
            task_id=params.task_id,
            first_turn="agent",
        )
    )

    print(f"Research completed: {research_output}")

    # Update task status to completed
    from uuid import UUID

    async def update_status() -> None:
        params.soma.update_task_status(
            task_id=UUID(params.task_id),
            update_task_status_request=UpdateTaskStatusRequest(
                status=TaskStatus.COMPLETED,
                message=CreateMessageRequest(
                    metadata={},
                    parts=[
                        MessagePart(
                            metadata={},
                            type="text-part",
                            text=research_output.summary,
                        )
                    ],
                    reference_task_ids=[],
                    role=MessageRole.AGENT,
                ),
            ),
        )

    await params.ctx.run("update_task_status", update_status)

    print("Claim research agent completed.")


# Export the agent
default = create_soma_agent(
    project_id="acme",
    agent_id="claimResearchAgent",
    name="Claim Research Agent",
    description="An agent that can research insurance claims using MCP tools.",
    entrypoint=entrypoint,
)
