export const repositoryUrl = "https://github.com/0xPolybit/principl";

export const navigation = [
  { label: "Home", href: "/" },
  { label: "Docs", href: "/docs" },
  { label: "Learn", href: "/language" },
  { label: "Examples", href: "/examples" },
];

export const docsNavigation = [
  { label: "Overview", href: "/docs" },
  { label: "Installation", href: "/installation" },
  { label: "Language guide", href: "/language" },
  { label: "Architecture", href: "/architecture" },
  { label: "Examples", href: "/examples" },
  { label: "Roadmap", href: "/roadmap" },
];

export function pageTitle(title: string) {
  return `${title} | PrinciPL`;
}
