"use client";

import { Search } from "lucide-react";
import type React from "react";
import { useState } from "react";
import { Input } from "@/components/ui/input";
import { PageHeader } from "@/components/ui/page-header";
import { PageLayout } from "@/components/ui/page-layout";
import type { SearchableTableProps } from "@/lib/types/shared";

interface DataTableLayoutProps extends SearchableTableProps {
	title: string;
	description?: string;
	searchPlaceholder?: string;
	headerActions?: React.ReactNode;
	children: React.ReactNode;
	showSearch?: boolean;
	searchValue?: string;
	onSearchChange?: (value: string) => void;
}

export function DataTableLayout({
	title,
	description,
	searchPlaceholder = "Search...",
	headerActions,
	children,
	className,
	showSearch = true,
	searchValue,
	onSearchChange,
}: DataTableLayoutProps) {
	const [internalSearchValue, setInternalSearchValue] = useState("");

	const searchVal =
		searchValue !== undefined ? searchValue : internalSearchValue;
	const handleSearchChange = onSearchChange || setInternalSearchValue;

	return (
		<PageLayout className={className}>
			<PageHeader title={title} description={description} />

			{(showSearch || headerActions) && (
				<div className="flex items-center justify-between gap-4 mb-6">
					{showSearch && (
						<div className="relative flex-1 max-w-md">
							<Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-muted-foreground h-4 w-4" />
							<Input
								placeholder={searchPlaceholder}
								value={searchVal}
								onChange={(e) => handleSearchChange(e.target.value)}
								className="pl-10"
							/>
						</div>
					)}

					{headerActions && (
						<div className="flex items-center gap-2">{headerActions}</div>
					)}
				</div>
			)}

			<div className="space-y-4">{children}</div>
		</PageLayout>
	);
}
