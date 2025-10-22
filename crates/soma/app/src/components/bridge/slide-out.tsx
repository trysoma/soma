import { AnimatePresence, motion } from "framer-motion";
import type { ReactNode } from "react";

interface SlideOutProps {
	isOpen: boolean;
	onClose: () => void;
	children: ReactNode;
}

export const SlideOut = ({ isOpen, onClose, children }: SlideOutProps) => {
	return (
		<AnimatePresence>
			{isOpen && (
				<>
					{/* Backdrop */}
					<motion.div
						className="fixed inset-0 bg-black/20 z-40"
						initial={{ opacity: 0 }}
						animate={{ opacity: 1 }}
						exit={{ opacity: 0 }}
						onClick={onClose}
					/>
					{/* Slide-out panel */}
					<motion.div
						className="fixed top-0 right-0 h-screen w-[50vw] border-l bg-background shadow-2xl z-50"
						initial={{ x: "100%" }}
						animate={{ x: 0 }}
						exit={{ x: "100%" }}
						transition={{ type: "spring", stiffness: 300, damping: 30 }}
					>
						{children}
					</motion.div>
				</>
			)}
		</AnimatePresence>
	);
};

