"""Insurance Claims Agent - Main agent definition."""

from typing import Any

from pydantic import BaseModel
from openai import AsyncOpenAI

from soma_sdk import create_soma_agent, HandlerParams, patterns
from soma_sdk.patterns import (
    ChatHandlerParams,
    WorkflowHandlerParams,
    WrappedChatHandlerParams,
    WrappedWorkflowHandlerParams,
)
from soma_api_client import (
    MessagePartTypeEnum,
    MessageRole,
    TaskStatus,
)

# Import from parent directory
import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent.parent))
from utils import convert_to_openai_messages


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

    client: AsyncOpenAI


class ProcessClaimInput(BaseModel):
    """Input for claim processing."""

    assessment: Assessment

    class Config:
        arbitrary_types_allowed = True


# Bridge type placeholder - will be replaced with generated bridge
BridgeType = Any


async def discover_claim_handler(
    params: ChatHandlerParams[BridgeType, DiscoverClaimInput, Assessment],
) -> None:
    """Handler for discovering claim details through conversation."""
    messages = convert_to_openai_messages(params.history)

    print("Messages:", messages)

    # Create the OpenAI request
    tools = [
        {
            "type": "function",
            "function": {
                "name": "decodeClaim",
                "description": "Decode a claim into a structured object.",
                "parameters": Assessment.model_json_schema(),
            },
        }
    ]

    response = await params.input.client.chat.completions.create(
        model="gpt-4o",
        messages=messages,
        tools=tools,
    )

    choice = response.choices[0]
    message = choice.message

    # Check if tool was called
    if message.tool_calls:
        for tool_call in message.tool_calls:
            if tool_call.function.name == "decodeClaim":
                import json
                args = json.loads(tool_call.function.arguments)
                assessment = Assessment(**args)
                params.on_goal_achieved(assessment)
                return

    # Send the assistant's response back
    if message.content:
        print(message.content)
        await params.send_message({
            "metadata": {},
            "parts": [
                {
                    "text": message.content,
                    "metadata": {},
                    "type": MessagePartTypeEnum.TEXT_PART.value,
                }
            ],
            "referenceTaskIds": [],
            "role": MessageRole.AGENT.value,
        })


async def process_claim_handler(
    params: WorkflowHandlerParams[BridgeType, ProcessClaimInput, None],
) -> None:
    """Handler for processing the claim."""
    print("Assessment:", params.input.assessment)

    await params.send_message({
        "metadata": {},
        "parts": [
            {
                "text": "Please wait while we process your claim... You should receive an email with the results shortly.",
                "metadata": {},
                "type": MessagePartTypeEnum.TEXT_PART.value,
            }
        ],
        "referenceTaskIds": [],
        "role": MessageRole.AGENT.value,
    })


# Create wrapped handlers using patterns
discover_claim = patterns.chat(discover_claim_handler)
process_claim = patterns.workflow(process_claim_handler)


async def entrypoint(params: HandlerParams) -> None:
    """Main agent entrypoint."""
    # Get bridge - in a real app this would be the generated bridge
    bridge = None  # get_bridge(params.ctx)

    # Create OpenAI client
    client = AsyncOpenAI()

    print("Discovering claim...")
    assessment = await discover_claim(WrappedChatHandlerParams(
        ctx=params.ctx,
        soma=params.soma,
        bridge=bridge,
        input=DiscoverClaimInput(client=client),
        task_id=params.task_id,
        first_turn="agent",
    ))

    await process_claim(WrappedWorkflowHandlerParams(
        ctx=params.ctx,
        soma=params.soma,
        bridge=bridge,
        input=ProcessClaimInput(assessment=assessment),
        task_id=params.task_id,
        interruptable=False,
    ))

    # Update task status to completed
    await params.ctx.run(lambda: params.soma.update_task_status(
        task_id=params.task_id,
        update_task_status_request={
            "status": TaskStatus.COMPLETED.value,
            "message": {
                "metadata": {},
                "parts": [
                    {
                        "metadata": {},
                        "type": MessagePartTypeEnum.TEXT_PART.value,
                        "text": "Claim processed",
                    }
                ],
                "referenceTaskIds": [],
                "role": "agent",
            },
        },
    ))


# Export the agent
default = create_soma_agent(
    project_id="acme",
    agent_id="insuranceClaimsAgent",
    name="Insurance Claims Agent",
    description="An agent that can process insurance claims.",
    entrypoint=entrypoint,
)
