import Link from "next/link";
import { navigation, repositoryUrl } from "@/lib/site";
import { PrinciMark } from "@/components/princi-mark";

export function SiteFooter() {
  return (
    <footer className="site-footer">
      <div className="page-width footer-main">
        <div className="footer-brand">
          <PrinciMark small />
          <div>
            <p className="footer-wordmark">PrinciPL</p>
            <p className="footer-note">A language, still taking shape.</p>
          </div>
        </div>
        <nav aria-label="Footer navigation" className="footer-links">
          {navigation.map((item) => (
            <Link href={item.href} key={item.href}>
              {item.label}
            </Link>
          ))}
          <a href={repositoryUrl} rel="noreferrer" target="_blank">
            Source on GitHub <span aria-hidden="true">↗</span>
          </a>
        </nav>
      </div>
      <div className="page-width footer-bottom">
        <span>Princi v0.1.0</span>
        <span>Windows x86-64 target</span>
        <Link href="/docs">Documentation</Link>
      </div>
    </footer>
  );
}
