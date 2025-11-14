import { createRouter, RouterProvider } from "@tanstack/react-router";
import { Suspense } from "react";
import { NotFound } from "./components/layout/not-found";
// Import the generated route tree
import { routeTree } from "./routeTree.gen";

// Create a new router instance
const router = createRouter({
	routeTree,
	context: {
		user: null,
	},
	defaultPreload: "intent",
	scrollRestoration: true,
	defaultStructuralSharing: true,
	defaultPreloadStaleTime: 0,
	defaultNotFoundComponent: NotFound,
});

// Register the router instance for type safety
declare module "@tanstack/react-router" {
	interface Register {
		router: typeof router;
	}
}

function AppInner() {
	return <RouterProvider router={router} />;
}

export default function App() {
	return (
		<Suspense fallback={<div>Loading...</div>}>
			<AppInner />
		</Suspense>
	);
}
