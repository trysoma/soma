"use client";

import type React from "react";
import { PageHeader } from "@/components/ui/page-header";
import { PageLayout } from "@/components/ui/page-layout";

interface DashboardLayoutProps {
	title: string;
	description?: string;
	headerActions?: React.ReactNode;
	children: React.ReactNode;
	className?: string;
	fullWidth?: boolean;
}

export function DashboardLayout({
	title,
	description,
	headerActions,
	children,
	className,
	fullWidth = false,
}: DashboardLayoutProps) {
	return (
		<PageLayout className={className} fullWidth={fullWidth}>
			<div className="flex items-center justify-between mb-8">
				<PageHeader title={title} description={description} />

				{headerActions && (
					<div className="flex items-center gap-2">{headerActions}</div>
				)}
			</div>

			{children}
		</PageLayout>
	);
}
