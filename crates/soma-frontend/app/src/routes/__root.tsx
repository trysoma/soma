import { TanstackDevtools } from "@tanstack/react-devtools";
import { ReactQueryDevtoolsPanel } from "@tanstack/react-query-devtools";
import { createRootRouteWithContext, Outlet } from "@tanstack/react-router";
import { TanStackRouterDevtoolsPanel } from "@tanstack/react-router-devtools";
import { Header } from "@/components/layout/header";
import { SubNavigation } from "@/components/layout/sub-navigation";
import { IdentityProvider } from "@/context/identity";
import ReactQueryProvider from "@/context/request-query-provider";
import { LINKS } from "@/lib/links";

export type RouterContext = {};

export const Route = createRootRouteWithContext<RouterContext>()({
	component: () => (
		<>
			<ReactQueryProvider>
				<IdentityProvider>
					<div className="h-screen bg-background antialiased w-full mx-auto scroll-smooth font-sans flex flex-col">
						<div className="shrink-0">
							<Header />
							<SubNavigation
								items={[
									{
										label: "Agents",
										href: LINKS.AGENTS(),
									},
									{
										label: "MCP",
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
				</IdentityProvider>
			</ReactQueryProvider>
		</>
	),
});
