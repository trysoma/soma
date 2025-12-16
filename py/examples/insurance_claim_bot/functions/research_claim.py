"""Function to research insurance claims."""

import random

from pydantic import BaseModel

from trysoma_sdk import (
    ProviderController,
    ProviderCredentialController,
    create_soma_function,
)


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


class ResearchResult(BaseModel):
    """Result of claim research."""

    summary: str


provider_controller = ProviderController(
    "research-claim",
    "Research",
    "Research a claim",
    [],
    [ProviderCredentialController.no_auth()],
)


async def research_claim_handler(input_data: Assessment) -> ResearchResult:
    """Handler that researches claims.

    In a real application, this would integrate with external
    research systems, databases, or APIs to gather information.

    Args:
        input_data: The assessment containing the claim to research.

    Returns:
        ResearchResult containing a summary of the research findings.
    """
    print(f"Researching claim: {input_data.claim}")

    # Simulate random research outcomes (like the JS version)
    if random.random() > 0.5:
        return ResearchResult(
            summary=(
                "This user has a history of claiming for the same amount of money "
                "multiple times. They may be trying to scam the system."
            )
        )

    return ResearchResult(
        summary=(
            "Could not find any relevant information about this claim. "
            "Try searching again until you find something relevant."
        )
    )


# Export the function
default = create_soma_function(
    input_schema=Assessment,
    output_schema=ResearchResult,
    provider_controller=provider_controller,
    function_name="research-claim",
    function_description="Research a claim. Use a search query to find relevant information about this claim.",
    handler=research_claim_handler,
)
