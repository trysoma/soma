"use client";

import { Link } from "@tanstack/react-router";

const Divider = () => {
	return (
		<svg
			aria-label="pencil"
			role="img"
			data-testid="geist-icon"
			height="16"
			strokeLinejoin="round"
			style={{ width: "16px", height: "16px", color: "var(--foreground)" }}
			viewBox="0 0 16 16"
			width="16"
		>
			<path
				fillRule="evenodd"
				clipRule="evenodd"
				d="M4.01526 15.3939L4.3107 14.7046L10.3107 0.704556L10.6061 0.0151978L11.9849 0.606077L11.6894 1.29544L5.68942 15.2954L5.39398 15.9848L4.01526 15.3939Z"
				fill="currentColor"
			></path>
		</svg>
	);
};


export function Header() {
	

	return (
		<header className="sticky top-0 h-[var(--header-height)] z-50 p-0 bg-background/60 backdrop-blur">
			<div className="flex justify-between items-center container mx-auto p-2 container ">
				<div className="flex  items-center gap-2">
					<Link
						to="/"
						title="brand-logo"
						className="relative  flex items-center space-x-2"
					>
						<span className="font-semibold text-3xl font-serif text-foreground">
							soma
						</span>
					</Link>


					

				</div>
			</div>
			<hr className="absolute w-full bottom-0" />
		</header>
	);
}

