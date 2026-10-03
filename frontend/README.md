# PrinciPL website

The separate Next.js website introduces PrinciPL and documents the compiler's
current v0.1 behavior. It does not modify or run the Rust compiler. Product
claims are based on the repository README, `docs/v0.1-scope.md`, and checked-in
examples.

## Prerequisites

- Node.js 20.9 or newer
- npm

## Install and run locally

From this directory:

```powershell
npm install
npm run dev
```

Open <http://localhost:3000>. The development server reloads when files change.

## Lint and production build

```powershell
npm run lint
npm run build
npm run start
```

`npm run start` serves the production build locally, by default at
<http://localhost:3000>.

## Main directories

| Path | Contents |
| --- | --- |
| `app/` | App Router pages, shared layout, metadata, and global styles |
| `components/` | Shared shell, reusable design primitives, article, code sample, tabs, and copy controls |
| `content/` | Compiler-grounded landing-page details, example source, and Markdown docs |
| `lib/` | Shared site helpers and Markdown outline extraction |
| `public/` | Public static assets |
| `styles/` | Shared design tokens |
| `briefs/` | Internal page briefs used during site design |
| `licenses/` | Third-party font license texts |

The landing page and guide pages are Server Components. The responsive menu
theme preference, extension tabs, and code-copy control are small Client
Components because they use browser interaction. DM Sans and IBM Plex Mono
are bundled locally from Fontsource packages; their SIL Open Font License
texts are in `licenses/`. The site does not request fonts from an external
service.

## Design system

The shared design is a compact compiler field guide: paper and graphite
surfaces, a restrained copper action color, and a small citron status accent.
DM Sans is used for interface and reading text; IBM Plex Mono is reserved for
source, commands, filenames, and technical labels. Color tokens are semantic
and support light, dark, and system themes. The selected preference is stored
in local browser storage, while system mode follows the operating-system
setting.

`components/site-primitives.tsx` contains reusable button links, badges,
cards, callouts, section headings, feature grids, and inline code. The shared
shell and navigation live in `site-header.tsx`, `site-footer.tsx`, and
`navigation-link.tsx`; `tabs.tsx` provides keyboard-operable tabs. Focus rings,
active navigation, and selected-tab states remain visible in both themes.

The landing page introduces only implemented v0.1 behavior as current. It
separates the process-lifetime managed heap from future memory-control ideas,
shows the class/reference and struct/value distinction, and labels possible
future work as roadmap rather than shipped capability.

## Documentation architecture

Documentation is Markdown under `content/docs/`, organized by reader task:
getting started, language guide, compiler, reference, and project. The
registry in `content/docs/navigation.ts` is the single source for page paths,
legacy path aliases, titles, descriptions, sidebar order, breadcrumbs,
active-page links, previous/next navigation, and GitHub edit links. Add a
Markdown file and one registry entry to publish a page; the route is statically
generated from that registry.

The `/docs` route and `/docs/[...slug]` use the shared shell in
`components/docs/`. It renders a collapsible section sidebar, breadcrumbs,
the v0.1 marker, generated page outline, and page navigation. Markdown headings
provide deep-link IDs and the right-side “On this page” list. At tablet/mobile
widths the section list and outline become keyboard-accessible `<details>`
drawers.

`react-markdown` renders content as Server Components. `remark-gfm` adds GFM
tables; `rehype-slug` creates heading IDs; `rehype-highlight` maps Princi
examples to the Rust grammar; `rehype-highlight-code-lines` supports numbered
and selected lines with fenced-code metadata such as
`~~~princi showLineNumbers {2}`. Code blocks use the existing copy control.
Blockquotes render as callouts. Raw HTML in Markdown is not enabled.

The former `/installation`, `/language`, `/architecture`, and `/roadmap` URLs
redirect to the matching canonical documentation pages so content does not
need to be maintained twice. Keep claims aligned with the root `README.md` and
`docs/v0.1-scope.md`.
