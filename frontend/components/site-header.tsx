"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { useState } from "react";
import { navigation, repositoryUrl } from "@/lib/site";
import { PrinciMark } from "@/components/princi-mark";

export function SiteHeader() {
  const pathname = usePathname();
  const [menuOpen, setMenuOpen] = useState(false);

  return (
    <header className="site-header">
      <div className="site-header-inner page-width">
        <Link className="brand" href="/" aria-label="PrinciPL home">
          <PrinciMark />
          <span className="brand-copy">
            <span className="brand-name">PrinciPL</span>
            <span className="brand-descriptor">Princi programming language</span>
          </span>
        </Link>

        <button
          aria-controls="primary-navigation"
          aria-expanded={menuOpen}
          aria-label={menuOpen ? "Close navigation" : "Open navigation"}
          className="menu-toggle"
          onClick={() => setMenuOpen((open) => !open)}
          type="button"
        >
          <span />
          <span />
        </button>

        <nav
          aria-label="Primary navigation"
          className={menuOpen ? "primary-navigation is-open" : "primary-navigation"}
          id="primary-navigation"
        >
          {navigation.map((item) => (
            <Link
              aria-current={pathname === item.href ? "page" : undefined}
              className="nav-link"
              href={item.href}
              key={item.href}
              onClick={() => setMenuOpen(false)}
            >
              {item.label}
            </Link>
          ))}
          <a
            className="nav-github"
            href={repositoryUrl}
            rel="noreferrer"
            target="_blank"
          >
            GitHub <span aria-hidden="true">↗</span>
          </a>
        </nav>
      </div>
    </header>
  );
}
