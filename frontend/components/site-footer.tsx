import Link from "next/link";
import { navigation, repositoryUrl } from "@/lib/site";
import { PrinciMark } from "@/components/princi-mark";
import { ArrowUpRight, Badge } from "@/components/site-primitives";

export function SiteFooter() {
  return (
    <footer className="site-footer">
      <div className="page-width footer-main">
        <div className="footer-brand">
          <PrinciMark small />
          <div>
            <p className="footer-wordmark">PrinciPL</p>
            <p className="footer-note">A statically typed language for native programs.</p>
          </div>
        </div>
        <nav aria-label="Footer navigation" className="footer-links">
          {navigation.map((item) => (
            <Link href={item.href} key={item.href}>
              {item.label}
            </Link>
          ))}
          <Link href="/docs/installation">Install</Link>
          <Link href="/architecture">Architecture</Link>
          <Link href="/roadmap">Roadmap</Link>
          <a href={repositoryUrl} rel="noreferrer" target="_blank">
            GitHub <ArrowUpRight />
          </a>
        </nav>
      </div>
      <div className="page-width footer-bottom">
        <span><Badge>Princi v0.1</Badge></span>
        <span>Windows x86-64 target</span>
        <Link href="/docs">Documentation</Link>
      </div>
    </footer>
  );
}
