"use client";
import { LINKS } from '@/lib/links';
import { createFileRoute } from '@tanstack/react-router'
import { Navigate } from '@tanstack/react-router'
import { useEffect } from 'react';
import { useNavigate } from '@tanstack/react-router';
export const Route = createFileRoute('/')({
  // component: HomePage,
  beforeLoad: () => {
    return <Navigate to={LINKS.A2A()} />
  }
})


export default function HomePage() {
  const navigate = useNavigate();
  useEffect(() => {
    navigate({ to: LINKS.A2A() })
  }, [])
  return <Navigate to={LINKS.A2A()} />
}