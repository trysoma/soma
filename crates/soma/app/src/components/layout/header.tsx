"use client";

import { Link } from "@tanstack/react-router";



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

