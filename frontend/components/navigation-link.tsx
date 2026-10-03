"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import type { ReactNode } from "react";

type NavigationLinkProps = {
  href: string;
  children: ReactNode;
  onNavigate?: () => void;
};

export function NavigationLink({ href, children, onNavigate }: NavigationLinkProps) {
  const pathname = usePathname();
  const isCurrentPage = pathname === href;

  return (
    <Link
      aria-current={isCurrentPage ? "page" : undefined}
      className="nav-link"
      href={href}
      onClick={onNavigate}
    >
      {children}
    </Link>
  );
}
