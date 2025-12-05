"""Function to approve insurance claims."""

from pydantic import BaseModel

from soma_sdk.bridge import (
    ProviderController,
    FunctionController,
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


class ApprovalResult(BaseModel):
    """Result of claim approval."""

    approved: bool


provider_controller = ProviderController(
    type_id="approve-claim",
    name="Approve Claim",
    documentation="Approve a claim",
    categories=[],
    credential_controllers=[
        {"type": "NoAuth"},
    ],
)

function_controller = FunctionController(
    name="approve-claim",
    description="Approve a claim",
)


async def approve_claim_handler(input_data: Assessment) -> ApprovalResult:
    """Handler that approves claims.

    In a real application, this would integrate with your
    claims processing system.

    Args:
        input_data: The assessment containing the claim to approve.

    Returns:
        ApprovalResult indicating whether the claim was approved.
    """
    # In a real app, you would:
    # 1. Validate the claim
    # 2. Check business rules
    # 3. Update your database
    # 4. Send notifications
    print(f"Approving claim: {input_data.claim}")
    return ApprovalResult(approved=True)


# Export the function
default = create_soma_function(
    input_schema=Assessment,
    output_schema=ApprovalResult,
    provider_controller=provider_controller,
    function_controller=function_controller,
    handler=approve_claim_handler,
)
