"use client";

import Link from "next/link";
import { useRef, useState } from "react";
import type { KeyboardEvent } from "react";
import { navigation, releaseVersion, repositoryUrl, windowsInstallerUrl } from "@/lib/site";
import { PrinciMark } from "@/components/princi-mark";
import { ArrowUpRight, Badge, DownloadIcon } from "@/components/site-primitives";
import { NavigationLink } from "@/components/navigation-link";
import { ThemeSwitcher } from "@/components/theme-switcher";

export function SiteHeader() {
  const [menuOpen, setMenuOpen] = useState(false);
  const menuButtonRef = useRef<HTMLButtonElement>(null);

  function handleMenuKeyDown(event: KeyboardEvent<HTMLElement>) {
    if (event.key === "Escape" && menuOpen) {
      setMenuOpen(false);
      menuButtonRef.current?.focus();
    }
  }

  return (
    <header className="site-header">
      <div className="site-header-inner page-width">
        <Link className="brand" href="/" aria-label="PrinciPL home">
          <PrinciMark />
          <span className="brand-copy">
            <span className="brand-line">
              <span className="brand-name">PrinciPL</span>
              <span className="brand-version">v0.1</span>
            </span>
            <span className="brand-descriptor">Princi programming language</span>
          </span>
        </Link>

        <div className="site-header-utilities">
          <Badge>v0.1</Badge>
          <ThemeSwitcher />
        </div>
        <button
          aria-controls="primary-navigation"
          aria-expanded={menuOpen}
          aria-label={menuOpen ? "Close menu" : "Open menu"}
          className="menu-toggle"
          onKeyDown={handleMenuKeyDown}
          onClick={() => setMenuOpen((open) => !open)}
          ref={menuButtonRef}
          type="button"
        >
          <span aria-hidden="true" className="menu-icon" />
          <span className="menu-toggle-label">Menu</span>
        </button>
        <nav
          aria-label="Primary navigation"
          className={menuOpen ? "primary-navigation is-open" : "primary-navigation"}
          id="primary-navigation"
          onKeyDown={handleMenuKeyDown}
        >
          {navigation.map((item) => (
            <NavigationLink href={item.href} key={item.href} onNavigate={() => setMenuOpen(false)}>
              {item.label}
            </NavigationLink>
          ))}
          <a
            aria-label={`Download Princi ${releaseVersion} for Windows x86-64`}
            className="nav-download button-link button-primary"
            href={windowsInstallerUrl}
            onClick={() => setMenuOpen(false)}
          >
            <DownloadIcon /> Download
          </a>
          <a
            className="nav-github"
            href={repositoryUrl}
            rel="noreferrer"
            target="_blank"
          >
            GitHub <ArrowUpRight />
          </a>
        </nav>
      </div>
    </header>
  );
}
