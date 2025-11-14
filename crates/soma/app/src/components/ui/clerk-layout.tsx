import { cn } from "@/lib/utils";

interface ClerkLayoutProps {
	children: React.ReactNode;
	className?: string;
	fullWidth?: boolean;
}

export function ClerkLayout({ children, className }: ClerkLayoutProps) {
	return (
		<div
			className={cn(
				"w-full h-full flex flex-col items-center justify-center min-h-screen",
				className,
			)}
		>
			{children}
		</div>
	);
}
