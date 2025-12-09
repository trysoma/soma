import { useCallback, useEffect, useState } from "react";
import type { components } from "@/@types/openapi";
import $api from "@/lib/api-client";

type AgentListItem = components["schemas"]["AgentListItem"];

interface UseAgentListResult {
	agents: AgentListItem[];
	isLoading: boolean;
	error: string | null;
	refetch: () => Promise<void>;
}

export function useAgentList(): UseAgentListResult {
	const [agents, setAgents] = useState<AgentListItem[]>([]);
	const [isLoading, setIsLoading] = useState(true);
	const [error, setError] = useState<string | null>(null);

	const fetchAgents = useCallback(async () => {
		setIsLoading(true);
		setError(null);
		try {
			const res = await $api.GET("/api/agent", {});
			if ("error" in res && res.error) {
				setError("Failed to fetch agents");
				setAgents([]);
			} else if (res.data) {
				setAgents(res.data.agents);
			}
		} catch (e) {
			setError(e instanceof Error ? e.message : "Unknown error");
			setAgents([]);
		} finally {
			setIsLoading(false);
		}
	}, []);

	useEffect(() => {
		fetchAgents();
	}, [fetchAgents]);

	return {
		agents,
		isLoading,
		error,
		refetch: fetchAgents,
	};
}
