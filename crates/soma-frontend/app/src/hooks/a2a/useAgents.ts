import type { AgentCard } from "@a2a-js/sdk";
import { A2AClient } from "@a2a-js/sdk/client";
import React from "react";
import { toast } from "sonner";

/**
 * Constructs the agent card URL for a specific agent
 */
export function getAgentCardUrl(projectId: string, agentId: string): string {
	return `/api/agent/${projectId}/${agentId}/a2a/.well-known/agent.json`;
}

/**
 * Constructs the A2A endpoint URL for a specific agent
 */
export function getAgentA2AEndpoint(
	projectId: string,
	agentId: string,
): string {
	return `/api/agent/${projectId}/${agentId}/a2a`;
}

export interface AgentIdentifier {
	projectId: string;
	agentId: string;
}

export interface UseAgentsReturn {
	agent: AgentCard | null;
	isLoading: boolean;
	error: Error | null;
	agentIdentifier: AgentIdentifier | null;
}

export interface UseAgentsParams {
	projectId: string;
	agentId: string;
}

export const useAgents = (params: UseAgentsParams): UseAgentsReturn => {
	const { projectId, agentId } = params;
	const [agent, setAgent] = React.useState<AgentCard | null>(null);
	const [isLoading, setIsLoading] = React.useState<boolean>(true);
	const [error, setError] = React.useState<Error | null>(null);

	const agentIdentifier = React.useMemo(
		() => ({ projectId, agentId }),
		[projectId, agentId],
	);

	React.useEffect(() => {
		const loadAgent = async () => {
			try {
				setIsLoading(true);
				const agentCardUrl = getAgentCardUrl(projectId, agentId);
				const client: A2AClient = await A2AClient.fromCardUrl(agentCardUrl);
				const agentCard: AgentCard = await client.getAgentCard();

				setAgent(agentCard);
				setError(null);
			} catch (err) {
				console.error("Error loading agent:", err);
				const errorMessage =
					err instanceof Error ? err.message : "Unknown error occurred";
				const errorObj = new Error(
					`Failed to fetch agent card for ${projectId}/${agentId}: ${errorMessage}`,
				);
				setError(errorObj);
				toast.error(errorObj.message);
			} finally {
				setIsLoading(false);
			}
		};

		loadAgent();
	}, [projectId, agentId]);

	return {
		agent,
		isLoading,
		error,
		agentIdentifier,
	};
};
