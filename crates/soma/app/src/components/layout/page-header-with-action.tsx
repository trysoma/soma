import type React from "react";
import { PageHeader } from "@/components/ui/page-header";

interface PageHeaderWithActionProps {
	title: string;
	description?: string;
	actions?: React.ReactNode;
}

export function PageHeaderWithAction({
	title,
	description,
	actions,
}: PageHeaderWithActionProps) {
	return (
		<div className="py-6 space-y-4">
			<div className="flex items-center justify-between">
				<PageHeader
					title={title}
					description={description}
					className="mb-0"
				/>
				{actions && <div className="flex items-center gap-2">{actions}</div>}
			</div>
		</div>
	);
}







