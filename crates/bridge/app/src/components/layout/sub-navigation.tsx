"use client";

import { CornerDownRightIcon, ExternalLinkIcon } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { cn } from "@/lib/utils";
import { Link, useLocation } from "@tanstack/react-router";
import { LINKS } from "@/lib/links";

export interface SubNavItem {
	label: string;
	href: string;
	external?: boolean;
}

interface SubNavProps {
	items: SubNavItem[];
	nestLevel?: "first" | "second";
	className?: string;
}

export function SubNavigation({ items, nestLevel = "first", className }: SubNavProps) {
	const {pathname} = useLocation();
	const [indicatorStyle, setIndicatorStyle] = useState({ left: 0, width: 0 });
	const [hoveredIndex, setHoveredIndex] = useState<number | null>(null);
	const navRefs = useRef<(HTMLAnchorElement | null)[]>([]);

	// Find the best matching item - exact match first, then longest prefix match
	const activeIndex =
		items.findIndex((item) => pathname === item.href) !== -1
			? items.findIndex((item) => pathname === item.href)
			: items.reduce((bestMatch, item, index) => {
					if (item.href !== "/" && pathname.startsWith(item.href)) {
						// If no match yet or this href is longer (more specific), use it
						if (
							bestMatch === -1 ||
							item.href.length > items[bestMatch].href.length
						) {
							return index;
						}
					}
					return bestMatch;
				}, -1);

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

	const isSecondLevel = nestLevel === "second";

	return (
		<nav
			className={cn(
				"border-b overflow-hidden ",
				isSecondLevel ? "bg-card" : "bg-background",
				className,
			)}
		>
			<div className="container mx-auto">
				<div className="relative">
					<nav
						className={cn(
							"flex items-center px-2 overflow-x-auto overflow-y-hidden scrollbar-none relative",
							isSecondLevel ? "h-[38px]" : "h-[46px]",
						)}
						onMouseLeave={() => setHoveredIndex(null)}
					>
						{isSecondLevel && (
							<div className="pl-5">
								<CornerDownRightIcon className="size-4" />
							</div>
						)}
						{items.map((item, index) => {
							const isActive = index === activeIndex;

							return (
								<Link
									to={item.href}
									key={item.href}
									ref={(el) => {
										navRefs.current[index] = el;
									}}
									target={item.external ? "_blank" : undefined}
									rel={item.external ? "noopener noreferrer" : undefined}
									className={cn(
										"relative inline-flex items-center select-none px-3 h-full text-sm no-underline transition-colors duration-200 whitespace-nowrap z-10",
										isActive
											? "text-foreground font-medium"
											: "text-muted-foreground hover:text-foreground",
									)}
									onMouseEnter={() => setHoveredIndex(index)}
								>
									{item.external ? (
										<>
											{item.label}
											<ExternalLinkIcon className="size-3 ml-1" />
										</>
									) : (
										item.label
									)}
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
	);
}
