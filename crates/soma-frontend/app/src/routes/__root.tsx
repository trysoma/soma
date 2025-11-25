import { TanstackDevtools } from "@tanstack/react-devtools";
import { ReactQueryDevtoolsPanel } from "@tanstack/react-query-devtools";
import { createRootRouteWithContext, Outlet } from "@tanstack/react-router";
import { TanStackRouterDevtoolsPanel } from "@tanstack/react-router-devtools";
import { Header } from "@/components/layout/header";
import { SubNavigation } from "@/components/layout/sub-navigation";
import ReactQueryProvider from "@/context/request-query-provider";
import { LINKS } from "@/lib/links";

export type RouterContext = {};

export const Route = createRootRouteWithContext<RouterContext>()({
	component: () => (
		<>
			<ReactQueryProvider>
				<div className="h-screen bg-background antialiased w-full mx-auto scroll-smooth font-sans flex flex-col">
					<div className="shrink-0">
						<Header />
						<SubNavigation
							items={[
								{
									label: "Agent 2 Agent",
									href: LINKS.A2A(),
								},
								{
									label: "Bridge MCP",
									href: LINKS.BRIDGE(),
								},
							]}
						/>
					</div>
					<div className="flex-1 overflow-hidden">
						<Outlet />
					</div>
					<TanstackDevtools
						config={{
							position: "bottom-left",
						}}
						plugins={[
							{
								name: "Tanstack Router",
								render: <TanStackRouterDevtoolsPanel />,
							},
							{
								name: "React Query",
								render: <ReactQueryDevtoolsPanel />,
							},
						]}
					/>
				</div>
			</ReactQueryProvider>
		</>
	),
});
