"""Insurance Claims Agent - Main agent definition."""

from typing import Any

from langchain_openai import ChatOpenAI
from langchain_core.messages import (
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
    WorkflowHandlerParams,
    WrappedChatHandlerParams,
    WrappedWorkflowHandlerParams,
)
from trysoma_sdk.langchain import create_soma_langchain_mcp_client
from trysoma_api_client import MessageRole, TaskStatus
from trysoma_api_client.models.create_message_request import CreateMessageRequest
from trysoma_api_client.models.message_part import MessagePart

# Normal imports - project root and soma are added to sys.path by standalone.py
from utils import convert_to_openai_messages
from soma.bridge import Bridge, get_bridge

# System prompt for the insurance claims agent
SYSTEM_PROMPT = """You are an insurance claims assistant. Your job is to help users file insurance claims by gathering the required information.

You need to collect the following information for a claim:
1. Date of the incident
2. Category (e.g., auto, home, health, travel)
3. Reason/description of what happened
4. Amount being claimed
5. Email address for correspondence

Be conversational and helpful. Ask for one or two pieces of information at a time.
Once you have all the required information, use the decodeClaim tool to submit the structured claim data.

If the user provides incomplete information, politely ask for the missing details."""


class InsuranceClaim(BaseModel):
    """Insurance claim details."""

    date: str
    category: str
    reason: str
    amount: float
    email: str


class Assessment(BaseModel):
    """Assessment containing the claim."""

    claim: InsuranceClaim


class DiscoverClaimInput(BaseModel):
    """Input for claim discovery."""

    class Config:
        arbitrary_types_allowed = True


class ProcessClaimInput(BaseModel):
    """Input for claim processing."""

    assessment: Assessment

    class Config:
        arbitrary_types_allowed = True


async def discover_claim_handler(
    params: ChatHandlerParams[Bridge, DiscoverClaimInput, Assessment],
) -> None:
    """Handler for discovering claim details through conversation."""
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

    # Create LangChain ChatOpenAI model
    model = ChatOpenAI(
        model="gpt-4o",
        temperature=0,
    )

    # Define the tool for structured output extraction
    decode_claim_tool = {
        "type": "function",
        "function": {
            "name": "decodeClaim",
            "description": "Decode a claim into a structured object when you have gathered all required information (date, category, reason, amount, email).",
            "parameters": {
                "type": "object",
                "properties": {
                    "claim": {
                        "type": "object",
                        "properties": {
                            "date": {
                                "type": "string",
                                "description": "The date of the incident",
                            },
                            "category": {
                                "type": "string",
                                "description": "Category of the claim (e.g., auto, home, health, travel)",
                            },
                            "reason": {
                                "type": "string",
                                "description": "Description of what happened",
                            },
                            "amount": {
                                "type": "number",
                                "description": "The amount being claimed",
                            },
                            "email": {
                                "type": "string",
                                "description": "Email address for correspondence",
                            },
                        },
                        "required": ["date", "category", "reason", "amount", "email"],
                    }
                },
                "required": ["claim"],
            },
        },
    }

    # Bind tools to the model
    model_with_tools = model.bind(tools=[decode_claim_tool])

    try:
        # Invoke the model
        response: Any = await model_with_tools.ainvoke(langchain_messages)

        # Check if the model called the decodeClaim tool
        if hasattr(response, "tool_calls") and response.tool_calls:
            for tool_call in response.tool_calls:
                if tool_call.get("name") == "decodeClaim":
                    args = tool_call.get("args", {})
                    claim_data = args.get("claim", {})

                    print(f"Extracted claim: {claim_data}")

                    # Create the assessment
                    claim = InsuranceClaim(
                        date=claim_data.get("date", ""),
                        category=claim_data.get("category", ""),
                        reason=claim_data.get("reason", ""),
                        amount=float(claim_data.get("amount", 0)),
                        email=claim_data.get("email", ""),
                    )
                    assessment = Assessment(claim=claim)

                    # Goal achieved
                    params.on_goal_achieved(assessment)
                    return

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
        print(f"Error in discover_claim_handler: {e}")
        # Send an error message to the user
        await params.send_message(
            CreateMessageRequest(
                metadata={},
                parts=[
                    MessagePart(
                        metadata={},
                        type="text-part",
                        text="I apologize, but I encountered an issue. Could you please provide the details of your insurance claim? I'll need the date, category, description, amount, and your email address.",
                    )
                ],
                reference_task_ids=[],
                role=MessageRole.AGENT,
            )
        )


async def process_claim_handler(
    params: WorkflowHandlerParams[Bridge, ProcessClaimInput, None],
) -> None:
    """Handler for processing the claim using MCP tools for research.

    This is a workflow handler that processes the claim in a single execution.
    It uses MCP tools to research and then makes a decision.
    """
    assessment = params.input.assessment
    print(f"Processing claim: {assessment}")

    # Create MCP client to access research tools using context manager
    async with create_soma_langchain_mcp_client(
        params.ctx,
        mcp_server_instance_id="test",
    ) as mcp_client:
        mcp_tools = await mcp_client.get_tools()
        print(f"Loaded {len(mcp_tools)} MCP tools for processing")

        # Build conversation messages for research
        research_messages: list[BaseMessage] = [
            SystemMessage(
                content=(
                    "You are now researching the claim to make an approval decision. "
                    "Use the available research tools to investigate the claim. "
                    "Once you have gathered enough information, use the make_decision tool "
                    "to approve or deny the claim."
                )
            ),
            HumanMessage(
                content=(
                    f"Please research and process this insurance claim:\n"
                    f"- Date: {assessment.claim.date}\n"
                    f"- Category: {assessment.claim.category}\n"
                    f"- Reason: {assessment.claim.reason}\n"
                    f"- Amount: ${assessment.claim.amount}\n"
                    f"- Email: {assessment.claim.email}"
                )
            ),
        ]

        # Create the decision tool - this signals workflow completion
        make_decision_tool = {
            "type": "function",
            "function": {
                "name": "make_decision",
                "description": "Make a final decision on the claim after researching.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "approved": {
                            "type": "boolean",
                            "description": "Whether to approve the claim",
                        },
                        "reason": {
                            "type": "string",
                            "description": "Reason for the decision",
                        },
                    },
                    "required": ["approved", "reason"],
                },
            },
        }

        # Convert MCP tools to OpenAI tool format
        all_tools = [make_decision_tool]
        for tool in mcp_tools:
            # Get parameters from args_schema if it's a Pydantic model
            parameters: dict[str, Any] = {"type": "object", "properties": {}}
            if (
                hasattr(tool, "args_schema")
                and tool.args_schema is not None
                and isinstance(tool.args_schema, type)
                and issubclass(tool.args_schema, BaseModel)
            ):
                parameters = tool.args_schema.model_json_schema()

            tool_dict = {
                "type": "function",
                "function": {
                    "name": tool.name,
                    "description": tool.description or f"Tool: {tool.name}",
                    "parameters": parameters,
                },
            }
            all_tools.append(tool_dict)

        model = ChatOpenAI(model="gpt-4o", temperature=0)
        model_with_tools = model.bind(tools=all_tools)

        approval_decision: bool | None = None
        decision_reason: str = ""

        try:
            # Invoke the model
            response: Any = await model_with_tools.ainvoke(research_messages)

            # Process tool calls
            if hasattr(response, "tool_calls") and response.tool_calls:
                research_messages.append(response)

                for tool_call in response.tool_calls:
                    tool_name = tool_call.get("name", "")
                    tool_args = tool_call.get("args", {})
                    tool_id = tool_call.get("id", "")

                    print(f"Tool called: {tool_name} with args: {tool_args}")

                    if tool_name == "make_decision":
                        approval_decision = tool_args.get("approved", False)
                        decision_reason = tool_args.get("reason", "")
                        print(
                            f"Decision made: {'Approved' if approval_decision else 'Denied'} - {decision_reason}"
                        )
                        break

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

                    research_messages.append(
                        ToolMessage(
                            content=str(tool_result),
                            tool_call_id=tool_id,
                        )
                    )

                # If we executed a tool but haven't made a decision yet, get follow-up
                if approval_decision is None:
                    follow_up: Any = await model_with_tools.ainvoke(research_messages)
                    if hasattr(follow_up, "tool_calls") and follow_up.tool_calls:
                        for fc in follow_up.tool_calls:
                            if fc.get("name") == "make_decision":
                                approval_decision = fc.get("args", {}).get(
                                    "approved", False
                                )
                                decision_reason = fc.get("args", {}).get("reason", "")
                                print(
                                    f"Decision made: {'Approved' if approval_decision else 'Denied'} - {decision_reason}"
                                )
                                break

        except Exception as e:
            print(f"Error in process_claim_handler: {e}")

        # Default to denied if no decision was made
        if approval_decision is None:
            approval_decision = False
            decision_reason = "Unable to complete research."

        # Send the final message to the user
        decision_text = "approved" if approval_decision else "denied"
        await params.send_message(
            CreateMessageRequest(
                metadata={},
                parts=[
                    MessagePart(
                        metadata={},
                        type="text-part",
                        text=(
                            f"Thank you! I have processed your claim.\n\n"
                            f"**Decision: {decision_text.upper()}**\n"
                            f"Reason: {decision_reason}\n\n"
                            f"You should receive an email at {assessment.claim.email} with the full details shortly."
                        ),
                    )
                ],
                reference_task_ids=[],
                role=MessageRole.AGENT,
            )
        )


# Create wrapped handlers using patterns
discover_claim = patterns.chat(discover_claim_handler)
process_claim = patterns.workflow(process_claim_handler)


async def entrypoint(params: HandlerParams) -> None:
    """Main agent entrypoint."""
    # Get bridge instance
    bridge = get_bridge(params.ctx)

    print("Starting insurance claims agent...")

    # Discover claim through conversation
    assessment = await discover_claim(
        WrappedChatHandlerParams(
            ctx=params.ctx,
            soma=params.soma,
            bridge=bridge,
            input=DiscoverClaimInput(),
            task_id=params.task_id,
            first_turn="agent",
        )
    )

    print(f"Claim discovered: {assessment}")

    # Process the claim using MCP tools for research
    await process_claim(
        WrappedWorkflowHandlerParams(
            ctx=params.ctx,
            soma=params.soma,
            bridge=bridge,
            input=ProcessClaimInput(assessment=assessment),
            task_id=params.task_id,
            interruptable=False,
        )
    )

    # Update task status to completed
    from uuid import UUID
    from trysoma_api_client.models.update_task_status_request import (
        UpdateTaskStatusRequest,
    )

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
                            text="Claim processed successfully!",
                        )
                    ],
                    reference_task_ids=[],
                    role=MessageRole.AGENT,
                ),
            ),
        )

    await params.ctx.run("update_task_status", update_status)

    print("Insurance claims agent completed.")


# Export the agent
default = create_soma_agent(
    project_id="acme",
    agent_id="insuranceClaimsAgent",
    name="Insurance Claims Agent",
    description="An agent that can process insurance claims.",
    entrypoint=entrypoint,
)
