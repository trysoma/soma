"use client";

import { Link, useLocation, useNavigate } from "@tanstack/react-router";
import { ChevronDown, CornerDownRightIcon, Plus } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { Button } from "@/components/ui/button";
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { cn } from "@/lib/utils";

export interface DropdownItem {
	label: string;
	value: string;
	href: string;
}

export interface NavLinkItem {
	label: string;
	href: string;
}

interface SubNavigationDropdownProps {
	items: DropdownItem[];
	selectedValue?: string;
	placeholder?: string;
	isLoading?: boolean;
	emptyMessage?: string;
	nestLevel?: "second" | "third";
	className?: string;
	onCreateNew?: () => void;
	createLabel?: string;
	/** Navigation links to show next to the dropdown */
	navItems?: NavLinkItem[];
}

export function SubNavigationDropdown({
	items,
	selectedValue,
	placeholder = "Select...",
	isLoading = false,
	emptyMessage = "No items available",
	nestLevel = "second",
	className,
	onCreateNew,
	createLabel = "Create new",
	navItems,
}: SubNavigationDropdownProps) {
	const navigate = useNavigate();
	const { pathname } = useLocation();

	const selectedItem = items.find((item) => item.value === selectedValue);
	const isThirdLevel = nestLevel === "third";

	// Refs and state for animated indicator
	const navRefs = useRef<(HTMLAnchorElement | null)[]>([]);
	const navContainerRef = useRef<HTMLDivElement | null>(null);
	const [indicatorStyle, setIndicatorStyle] = useState({ left: 0, width: 0 });
	const [hoveredIndex, setHoveredIndex] = useState<number | null>(null);

	// Find active nav item
	const activeNavIndex = navItems
		? navItems.findIndex((item) => pathname === item.href) !== -1
			? navItems.findIndex((item) => pathname === item.href)
			: navItems.reduce((bestMatch, item, index) => {
					if (item.href !== "/" && pathname.startsWith(item.href)) {
						if (
							bestMatch === -1 ||
							item.href.length > navItems[bestMatch].href.length
						) {
							return index;
						}
					}
					return bestMatch;
				}, -1)
		: -1;

	// Update indicator position
	useEffect(() => {
		const targetIndex = hoveredIndex !== null ? hoveredIndex : activeNavIndex;
		if (targetIndex >= 0 && navRefs.current[targetIndex]) {
			const element = navRefs.current[targetIndex];
			setIndicatorStyle({
				left: element.offsetLeft,
				width: element.offsetWidth,
			});
		}
	}, [hoveredIndex, activeNavIndex]);

	const navHeight = isThirdLevel ? "h-[34px]" : "h-[38px]";

	if (isLoading) {
		return (
			<nav
				className={cn("border-b overflow-hidden bg-card", navHeight, className)}
			>
				<div className="container mx-auto">
					<div
						className={cn("flex items-center justify-between px-2", navHeight)}
					>
						<div className="flex items-center">
							<div className={cn(isThirdLevel ? "pl-8" : "pl-5")}>
								<CornerDownRightIcon
									className={cn(isThirdLevel ? "size-3" : "size-4")}
								/>
							</div>
							<span className="text-sm text-muted-foreground ml-3">
								Loading...
							</span>
						</div>
					</div>
				</div>
			</nav>
		);
	}

	return (
		<nav
			className={cn("border-b overflow-hidden bg-card", navHeight, className)}
		>
			<div className="container mx-auto">
				<div className={cn("flex items-center px-2", navHeight)}>
					{/* Left side: dropdown, create button, and navigation links */}
					<div className="flex items-center">
						<div className={cn(isThirdLevel ? "pl-8" : "pl-5")}>
							<CornerDownRightIcon
								className={cn(isThirdLevel ? "size-3" : "size-4")}
							/>
						</div>

						<DropdownMenu>
							<DropdownMenuTrigger asChild>
								<Button
									variant="ghost"
									size="sm"
									className={cn(
										"ml-2 h-7 px-2 gap-1 font-normal",
										selectedItem ? "text-foreground" : "text-muted-foreground",
									)}
									disabled={items.length === 0 && !onCreateNew}
								>
									{selectedItem?.label || placeholder}
									<ChevronDown className="size-3" />
								</Button>
							</DropdownMenuTrigger>
							<DropdownMenuContent align="start" className="min-w-[200px]">
								{items.length === 0 ? (
									<DropdownMenuItem disabled className="text-muted-foreground">
										{emptyMessage}
									</DropdownMenuItem>
								) : (
									items.map((item) => (
										<DropdownMenuItem
											key={item.value}
											onClick={() => navigate({ to: item.href })}
											className={cn(
												selectedValue === item.value && "bg-accent font-medium",
											)}
										>
											{item.label}
										</DropdownMenuItem>
									))
								)}
							</DropdownMenuContent>
						</DropdownMenu>

						{onCreateNew && (
							<Button
								variant="ghost"
								size="sm"
								className="ml-1 h-7 px-2 gap-1 text-muted-foreground hover:text-foreground"
								onClick={onCreateNew}
							>
								<Plus className="size-3" />
								{createLabel}
							</Button>
						)}

						{/* Navigation links with animated indicator */}
						{navItems && navItems.length > 0 && (
							<>
								<div className="h-4 w-px bg-border ml-3" />
								<div
									ref={navContainerRef}
									className={cn("relative flex items-center ml-2", navHeight)}
									onMouseLeave={() => setHoveredIndex(null)}
								>
									{navItems.map((item, index) => {
										const isActive = index === activeNavIndex;
										return (
											<Link
												key={item.href}
												to={item.href}
												ref={(el) => {
													navRefs.current[index] = el;
												}}
												className={cn(
													"relative inline-flex items-center select-none px-3 h-full text-sm no-underline transition-colors duration-200 whitespace-nowrap z-10",
													isActive
														? "text-foreground font-medium"
														: "text-muted-foreground hover:text-foreground",
												)}
												onMouseEnter={() => setHoveredIndex(index)}
											>
												{item.label}
											</Link>
										);
									})}
									{/* Animated indicator */}
									<span
										className={cn(
											"absolute bottom-0 h-[2px] bg-foreground transition-all duration-300 ease-out",
											activeNavIndex < 0 &&
												hoveredIndex === null &&
												"opacity-0",
										)}
										style={{
											left: `${indicatorStyle.left}px`,
											width: `${indicatorStyle.width}px`,
										}}
									/>
								</div>
							</>
						)}
					</div>
				</div>
			</div>
		</nav>
	);
}
