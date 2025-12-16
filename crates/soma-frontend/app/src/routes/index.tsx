"use client";
import { createFileRoute, Navigate, useNavigate } from "@tanstack/react-router";
import { useEffect } from "react";
import { LINKS } from "@/lib/links";
export const Route = createFileRoute("/")({
	// component: HomePage,
	beforeLoad: () => {
		return <Navigate to={LINKS.AGENTS()} />;
	},
});

export default function HomePage() {
	const navigate = useNavigate();
	useEffect(() => {
		navigate({ to: LINKS.AGENTS() });
	}, [navigate]);
	return <Navigate to={LINKS.AGENTS()} />;
}
