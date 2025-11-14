"use client";

import { Link } from "@tanstack/react-router";
import type { ReactNode } from "react";
import {
	Card,
	CardContent,
	CardDescription,
	CardHeader,
	CardTitle,
} from "@/components/ui/card";

interface CardItemProps {
	title: string;
	description?: string;
	href?: string;
	onClick?: () => void;
	children?: ReactNode;
	className?: string;
}

export function CardItem({
	title,
	description,
	href,
	onClick,
	children,
	className,
}: CardItemProps) {
	const content = (
		<Card
			className={`h-full hover:shadow-lg transition-shadow ${href || onClick ? "cursor-pointer" : ""} ${className || ""}`}
		>
			<CardHeader className="pb-3">
				<div className="space-y-2">
					<CardTitle className="text-base line-clamp-1">{title}</CardTitle>
					{description && (
						<CardDescription className="text-xs line-clamp-2 min-h-[2.5rem]">
							{description}
						</CardDescription>
					)}
				</div>
			</CardHeader>
			{children && <CardContent className="space-y-4">{children}</CardContent>}
		</Card>
	);

	if (href) {
		return (
			<Link
				to={href}
				className="block transition-transform hover:scale-[1.02]"
				onClick={onClick}
			>
				{content}
			</Link>
		);
	}

	if (onClick) {
		return (
			<div
				className="block transition-transform hover:scale-[1.02]"
				onClick={onClick}
			>
				{content}
			</div>
		);
	}

	return content;
}
