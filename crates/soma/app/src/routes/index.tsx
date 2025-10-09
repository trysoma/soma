"use client";
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/')({
  component: HomePage,
})


export default function HomePage() {
	return <div>Home</div>
}