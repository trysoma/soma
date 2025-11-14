"use client";

import { Outlet } from "@tanstack/react-router";
import type { ReactNode } from "react";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";

export interface TabConfig {
	value: string;
	label: string;
	pathPattern: string; // Used to match against pathname to determine active tab
	component: ReactNode;
}

interface SlideOutTabsProps {
	tabs: TabConfig[];
	getCurrentTab: () => string;
	className?: string;
}

export const SlideOutTabs = ({
	tabs,
	getCurrentTab,
	className,
}: SlideOutTabsProps) => {
	return (
		<Tabs
			value={getCurrentTab()}
			className="flex-1 flex flex-col overflow-scroll"
		>
			<TabsList
				className={`mx-4 mt-4 grid w-fit flex-shrink-0 ${className}`}
				style={{
					gridTemplateColumns: `repeat(${tabs.length}, minmax(0, 1fr))`,
				}}
			>
				{tabs.map((tab) => (
					<TabsTrigger key={tab.value} value={tab.value} asChild>
						{tab.component}
					</TabsTrigger>
				))}
			</TabsList>

			<ScrollArea className="flex-1">
				<Outlet />
			</ScrollArea>
		</Tabs>
	);
};
