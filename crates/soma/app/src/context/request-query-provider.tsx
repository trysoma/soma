"use client";

import { queryClient } from "@/main";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useState } from "react";

function ReactQueryProvider({ children }: React.PropsWithChildren) {

	return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>;
}

export default ReactQueryProvider;
