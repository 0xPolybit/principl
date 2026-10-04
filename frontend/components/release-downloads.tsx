import { Badge, DownloadIcon } from "@/components/site-primitives";
import {
  releasePageUrl,
  releaseVersion,
  windowsInstallerUrl,
  windowsPortableArchiveUrl,
} from "@/lib/site";

export function ReleaseDownloads() {
  return (
    <div className="release-downloads">
      <div aria-label={`Princi ${releaseVersion} downloads`} className="release-download-actions" role="group">
        <a
          aria-label="Download Princi v0.1.0 Windows x86-64 installer"
          className="button-link button-primary"
          href={windowsInstallerUrl}
        >
          <DownloadIcon />
          Windows installer <code>.exe</code>
        </a>
        <a
          aria-label="Download Princi v0.1.0 Windows x86-64 portable archive"
          className="button-link button-secondary"
          href={windowsPortableArchiveUrl}
        >
          <DownloadIcon />
          Portable archive <code>.zip</code>
        </a>
      </div>
      <p className="release-download-details">
        <Badge tone="current">{releaseVersion}</Badge>
        <span>Windows x86-64 release</span>
      </p>
      <p className="release-download-note">
        The installer offers optional user <code>PATH</code> integration; the ZIP must be extracted manually. Compiling Princi programs requires LLVM/Clang and MinGW-w64, which are not bundled. See the <a href="/docs/installation#prerequisites">installation guide</a> or the <a href={releasePageUrl}>release notes and checksums</a>.
      </p>
    </div>
  );
}
