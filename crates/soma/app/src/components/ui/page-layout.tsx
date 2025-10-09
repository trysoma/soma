import { cn } from "@/lib/utils";

interface PageLayoutProps {
	children: React.ReactNode;
	className?: string;
	fullWidth?: boolean;
}

export function PageLayout({
	children,
	className,
	fullWidth = false,
}: PageLayoutProps) {
	return (
		<div
			className={cn(
				"w-full",
				!fullWidth && "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8",
				className,
			)}
		>
			{children}
		</div>
	);
}
