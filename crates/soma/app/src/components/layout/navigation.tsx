"use client";

import { useEffect, useRef, useState } from "react";
import { cn } from "@/lib/utils";
import { Link, useLocation } from "@tanstack/react-router";
import { LINKS } from "@/lib/links";

interface NavItem {
	name: string;
	href: string;
	segment: string;
	requiresDeployment?: boolean;
}

const navItems: NavItem[] = [
	{
		name: "Overview",
		href: "",
		segment: "(overview)",
		requiresDeployment: false,
	},
	{
		name: "Chat",
		href: LINKS.CHAT(),
		segment: "chat",
		requiresDeployment: false,
	}
];

export function Navigation() {
	const location = useLocation();
	const basePath = location.pathname;
	
	const [indicatorStyle, setIndicatorStyle] = useState({ left: 0, width: 0 });
	const [hoveredIndex, setHoveredIndex] = useState<number | null>(null);
	const navRefs = useRef<(HTMLAnchorElement | null)[]>([]);


	const getActiveSegment = () => {
		const currentPath = location.pathname;
		if (!currentPath || currentPath === "") return "(overview)";
		const segments = currentPath.split("/").filter(Boolean);
		const segment = segments[0];
		return segment || "(overview)";
	};

	const activeSegment = getActiveSegment();
	const activeIndex = navItems.findIndex(
		(item) => item.segment === activeSegment,
	);

	useEffect(() => {
		const targetIndex = hoveredIndex !== null ? hoveredIndex : activeIndex;
		if (targetIndex >= 0 && navRefs.current[targetIndex]) {
			const element = navRefs.current[targetIndex];
			setIndicatorStyle({
				left: element.offsetLeft,
				width: element.offsetWidth,
			});
		}
	}, [hoveredIndex, activeIndex]);

	return (
		<>
			{/* Navigation */}
			<nav className="sticky top-[var(--header-height)] z-40 bg-background border-b overflow-hidden">
				<div className="container mx-auto">
					<div className="relative">
						<nav
							className="flex h-[46px] items-center px-2 overflow-x-auto overflow-y-hidden scrollbar-none relative"
							onMouseLeave={() => setHoveredIndex(null)}
						>
							{navItems.map((item, index) => {
								const isActive = item.segment === activeSegment;
								const href = `${basePath}${item.href}`;

								return (
									<Link
										key={item.segment}
										ref={(el: HTMLAnchorElement | null) => {
											navRefs.current[index] = el;
										}}
										to={href}
										className={cn(
											"relative inline-flex items-center select-none px-3 h-full text-sm no-underline transition-colors duration-200 whitespace-nowrap z-10",
											isActive
												? "text-foreground font-medium"
												: "text-muted-foreground hover:text-foreground",
										)}
										onMouseEnter={() => setHoveredIndex(index)}
									>
										{item.name}
									</Link>
								);
							})}
							<span
								className="absolute bottom-0 h-[2px] bg-foreground transition-all duration-300 ease-out"
								style={{
									left: `${indicatorStyle.left}px`,
									width: `${indicatorStyle.width}px`,
								}}
							/>
						</nav>
					</div>
				</div>
			</nav>
		</>
	);
}
