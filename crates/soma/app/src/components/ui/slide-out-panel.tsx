"use client";

import type { ReactNode } from "react";
import { AnimatePresence, motion } from "framer-motion";
import { X } from "lucide-react";
import { Button } from "@/components/ui/button";

interface SlideOutPanelProps {
	isOpen?: boolean;
	onClose: () => void;
	title: string;
	subtitle?: string;
	icon?: ReactNode;
	children: ReactNode;
}

export const SlideOutPanel = ({
	isOpen = true,
	onClose,
	title,
	subtitle,
	icon,
	children,
}: SlideOutPanelProps) => {
	return (
		<AnimatePresence>
			{isOpen && (
				<>
					{/* Backdrop overlay */}
					<motion.div
						initial={{ opacity: 0 }}
						animate={{ opacity: 1 }}
						exit={{ opacity: 0 }}
						transition={{ duration: 0.2 }}
						className="fixed inset-0 bg-black/50 z-40"
						onClick={onClose}
					/>

					{/* Slide-out panel */}
					<motion.div
						initial={{ x: "100%" }}
						animate={{ x: 0 }}
						exit={{ x: "100%" }}
						transition={{ type: "spring", damping: 30, stiffness: 300 }}
						className="fixed top-0 right-0 h-screen w-2/3 bg-background border-l shadow-2xl z-50 flex flex-col"
						onClick={(e) => e.stopPropagation()}
					>
						{/* Header */}
						<div className="flex items-center justify-between p-4 border-b">
							<div className="flex items-center gap-3">
								{icon}
								<div>
									<h2 className="font-semibold">{title}</h2>
									{subtitle && (
										<p className="text-sm text-muted-foreground">{subtitle}</p>
									)}
								</div>
							</div>
							<Button variant="ghost" size="icon" onClick={onClose}>
								<X className="h-4 w-4" />
							</Button>
						</div>

						{/* Content */}
						{children}
					</motion.div>
				</>
			)}
		</AnimatePresence>
	);
};
