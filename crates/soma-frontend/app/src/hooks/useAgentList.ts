"use client";
import type { components } from "@/@types/openapi";
import $api from "@/lib/api-client.client";

type AgentListItem = components["schemas"]["AgentListItem"];

interface UseAgentListResult {
	agents: AgentListItem[];
	isLoading: boolean;
	error: string | null;
	refetch: () => void;
}

export function useAgentList(): UseAgentListResult {
	const { data, isLoading, error, refetch } = $api.useQuery(
		"get",
		"/api/agent",
		{},
		{
			// Refetch every 5 seconds while agents list is empty to handle race condition
			// where frontend loads before agents are registered
			refetchInterval: (query) => {
				const agents = query.state.data?.agents;
				return !agents || agents.length === 0 ? 5000 : false;
			},
		},
	);

	return {
		agents: data?.agents || [],
		isLoading,
		error: error ? String(error) : null,
		refetch,
	};
}
