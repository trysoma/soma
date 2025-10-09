import { ArrowLeft } from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

interface PageHeaderProps {
	title: string;
	description?: string;
	className?: string;
	backUrl?: string;
	backLabel?: string;
	onBack?: () => void;
}

export function PageHeader({
	title,
	description,
	className,
	backUrl,
	backLabel = "Back",
	onBack,
}: PageHeaderProps) {
	const handleBack = () => {
		if (onBack) {
			onBack();
		} else if (backUrl) {
			window.location.href = backUrl;
		}
	};

	return (
		<div className={cn("space-y-3", className)}>
			<div className="space-y-1">
				<h1 className="text-2xl font-bold">{title}</h1>
				{description && (
					<p className="text-sm text-muted-foreground">{description}</p>
				)}
			</div>
			{(backUrl || onBack) && (
				<div className="flex items-center gap-4">
					<Button
						variant="ghost"
						size="sm"
						onClick={handleBack}
						className="gap-2"
					>
						<ArrowLeft className="h-4 w-4" />
						{backLabel}
					</Button>
				</div>
			)}
		</div>
	);
}
