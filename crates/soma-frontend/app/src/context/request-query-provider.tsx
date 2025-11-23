"use client";

import { QueryClientProvider } from "@tanstack/react-query";
import { queryClient } from "@/main";

function ReactQueryProvider({ children }: React.PropsWithChildren) {
	return (
		<QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
	);
}

export default ReactQueryProvider;
