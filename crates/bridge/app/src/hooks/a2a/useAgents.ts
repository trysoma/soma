import type { AgentCard } from "@a2a-js/sdk";
import React from "react";
import { toast } from "sonner";

import { A2AClient } from "@a2a-js/sdk/client";

const AGENT_URL = "/api/agent/v1/.well-known/agent.json";

export interface UseAgentsReturn {
  agent: AgentCard | null;
  isLoading: boolean;
  error: Error | null;
}

export const useAgents = (): UseAgentsReturn => {
  const [agent, setAgent] = React.useState<AgentCard | null>(null);
  const [isLoading, setIsLoading] = React.useState<boolean>(true);
  const [error, setError] = React.useState<Error | null>(null);

  React.useEffect(() => {
    const loadAgent = async () => {
      try {
        setIsLoading(true);
        const client: A2AClient = await A2AClient.fromCardUrl(AGENT_URL);
        const agentCard: AgentCard = await client.getAgentCard();

        setAgent(agentCard);
        setError(null);
      } catch (err) {
        console.error("Error loading agent:", err);
        const errorMessage = err instanceof Error ? err.message : "Unknown error occurred";
        const errorObj = new Error(`Failed to fetch agent card: ${errorMessage}`);
        setError(errorObj);
        toast.error(errorObj.message);
      } finally {
        setIsLoading(false);
      }
    };

    loadAgent();
  }, []);

  return {
    agent,
    isLoading,
    error,
  };
};
