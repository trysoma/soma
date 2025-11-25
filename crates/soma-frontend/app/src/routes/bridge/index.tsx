import { createFileRoute, Navigate } from "@tanstack/react-router";
import { LINKS } from "@/lib/links";

export const Route = createFileRoute("/bridge/")({
	component: RouteComponent,
});

function RouteComponent() {
	return <Navigate to={LINKS.BRIDGE_ENABLE_FUNCTIONS()} replace />;
}
