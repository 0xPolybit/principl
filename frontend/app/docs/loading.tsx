export default function DocsLoading() {
  return (
    <main aria-busy="true" aria-label="Loading documentation" id="main-content">
      <p aria-live="polite" className="sr-only" role="status">Loading documentation page</p>
      <div aria-hidden="true" className="docs-layout docs-loading-skeleton">
        <aside>
          <div className="docs-loading-line" />
          <div className="docs-loading-line docs-loading-line-short" />
          <div className="docs-loading-line" />
          <div className="docs-loading-line docs-loading-line-short" />
        </aside>
        <div className="docs-loading-main">
          <div className="docs-loading-line docs-loading-line-short" />
          <div className="docs-loading-line docs-loading-line-title" />
          <div className="docs-loading-line" />
          <div className="docs-loading-line docs-loading-line-short" />
          <div className="docs-loading-block" />
          <div className="docs-loading-line" />
          <div className="docs-loading-line docs-loading-line-short" />
        </div>
        <aside>
          <div className="docs-loading-line" />
          <div className="docs-loading-line docs-loading-line-short" />
          <div className="docs-loading-line" />
        </aside>
      </div>
    </main>
  );
}
