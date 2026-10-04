export const repositoryUrl = "https://github.com/0xPolybit/principl";
export const releaseVersion = "v0.1.0";
export const releasePageUrl = `${repositoryUrl}/releases/tag/${releaseVersion}`;
export const latestReleasePageUrl = `${repositoryUrl}/releases/latest`;
const releaseAssetBaseUrl = `${repositoryUrl}/releases/download/${releaseVersion}`;
const releaseAssetVersion = releaseVersion.slice(1);
export const windowsInstallerUrl = `${releaseAssetBaseUrl}/Princi-Setup-${releaseAssetVersion}-x64.exe`;
export const windowsPortableArchiveUrl = `${releaseAssetBaseUrl}/princi-${releaseAssetVersion}-windows-x64.zip`;

export const navigation = [
  { label: "Home", href: "/" },
  { label: "Docs", href: "/docs" },
  { label: "Learn", href: "/docs/language/syntax" },
  { label: "Examples", href: "/examples" },
];

export function pageTitle(title: string) {
  return `${title} | PrinciPL`;
}
