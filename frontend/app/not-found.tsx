import Link from "next/link";
import { ArrowUpRight } from "@/components/site-primitives";

export default function NotFound() {
  return (
    <main className="page-width not-found" id="main-content">
      <p className="pipeline-index">404 / NOT FOUND</p>
      <h1>This page is out of bounds.</h1>
      <p>The address may have changed. Return to the PrinciPL home page or browse the documentation.</p>
      <div className="hero-actions">
        <Link className="button-primary" href="/">
          Go to the home page <ArrowUpRight />
        </Link>
        <Link className="button-link button-secondary" href="/docs">
          Browse documentation <ArrowUpRight />
        </Link>
      </div>
    </main>
  );
}
