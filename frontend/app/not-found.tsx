import Link from "next/link";

export default function NotFound() {
  return (
    <main className="page-width not-found" id="main-content">
      <p className="pipeline-index">404 / NOT FOUND</p>
      <h1>This page is out of bounds.</h1>
      <p>The address may have changed. Return to the PrinciPL home page.</p>
      <Link className="button-primary" href="/">
        Go to the home page <span aria-hidden="true">↗</span>
      </Link>
    </main>
  );
}
