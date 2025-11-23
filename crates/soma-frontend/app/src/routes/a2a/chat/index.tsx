import { createFileRoute } from "@tanstack/react-router";
import { A2aChatLayout } from "@/components/a2a-chat";
import { SidebarProvider } from "@/components/ui/sidebar";

export const Route = createFileRoute("/a2a/chat/")({
	component: RouteComponent,
});

function RouteComponent() {
	return (
		<SidebarProvider>
			<A2aChatLayout />
		</SidebarProvider>
	);
}
