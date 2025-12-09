import { createFileRoute } from "@tanstack/react-router";
import { A2aChatLayout } from "@/components/a2a-chat";
import { SidebarProvider } from "@/components/ui/sidebar";

export const Route = createFileRoute(
	"/agent/$projectId/$agentId/a2a/chat-debugger",
)({
	component: RouteComponent,
});

function RouteComponent() {
	return (
		<SidebarProvider>
			<A2aChatLayout />
		</SidebarProvider>
	);
}
