"use client";

import type { LucideIcon } from "lucide-react";
import type { ReactNode } from "react";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";

interface EmptyStateProps {
	icon?: LucideIcon;
	title: string;
	description?: string;
	action?: {
		label: string;
		icon?: LucideIcon;
		onClick: () => void;
	};
	children?: ReactNode;
}

export function EmptyState({
	icon: Icon,
	title,
	description,
	action,
	children,
}: EmptyStateProps) {
	return (
		<Card className="p-8">
			<div className="text-center space-y-3">
				{Icon && (
					<div className="w-12 h-12 rounded-full bg-muted mx-auto flex items-center justify-center">
						<Icon className="h-6 w-6 text-muted-foreground" />
					</div>
				)}
				<h3 className="font-semibold">{title}</h3>
				{description && (
					<p className="text-sm text-muted-foreground max-w-sm mx-auto">
						{description}
					</p>
				)}
				{action && (
					<Button onClick={action.onClick}>
						{action.icon && <action.icon className="h-4 w-4 mr-2" />}
						{action.label}
					</Button>
				)}
				{children}
			</div>
		</Card>
	);
}
