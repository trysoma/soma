import { TrendingDownIcon, TrendingUpIcon } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { cn } from "@/lib/utils";

interface MetricCardProps {
	title: string;
	value: string | number;
	unit?: string;
	change?: number;
	description?: string;
	className?: string;
}

export function MetricCard({
	title,
	value,
	unit = "",
	change,
	description = "over the last 24 hours",
	className,
}: MetricCardProps) {
	return (
		<Card className={cn("", className)}>
			<CardHeader className="pb-2">
				<CardTitle className="text-sm font-medium text-muted-foreground">
					{title}
				</CardTitle>
			</CardHeader>
			<CardContent className="space-y-1">
				<div className="text-2xl font-bold">
					{value}
					{unit}
				</div>
				{change !== undefined && (
					<div className="flex items-center gap-1">
						<Badge variant="outline" className="gap-1">
							{change > 0 ? (
								<TrendingUpIcon className="h-3 w-3" />
							) : (
								<TrendingDownIcon className="h-3 w-3" />
							)}
							{Math.abs(change)}%
						</Badge>
					</div>
				)}
				{description && (
					<p className="text-xs text-muted-foreground">{description}</p>
				)}
			</CardContent>
		</Card>
	);
}
