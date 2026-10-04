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
| `components/` | Shared shell, reusable design primitives, release download controls, docs diagrams, example gallery, syntax display, and copy controls |
| `content/` | Compiler-grounded page data, the example catalog, and Markdown docs |
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

## Examples gallery

`/examples` reads its single catalog from `content/examples.ts`. Each entry
contains a complete v0.1 source file, its `.prnc` or `.princi` filename,
difficulty, concepts, explanation, deterministic static output, and related
language-guide links. `components/examples/example-gallery.tsx` provides
client-side search and category filters; `components/princi-code.tsx` applies
a small tokenizer for the language's current keywords, types, literals,
operators, and comments. Copy controls write the original, unhighlighted source
string. The output panels are labeled static: the browser does not execute
Princi code.

The gallery examples are built and run locally through the repository's
Windows compiler during authoring. Keep them synchronized with the Rust source,
`README.md`, and `docs/v0.1-scope.md` when language behavior changes.

## Design system

The shared design is a compact compiler field guide: paper and graphite
surfaces, a restrained copper action color, and a small citron status accent.
DM Sans is used for interface and reading text; IBM Plex Mono is reserved for
source, commands, filenames, and technical labels. Color tokens are semantic
and support light, dark, and system themes. The selected preference is stored
in local browser storage, while system mode follows the operating-system
setting.

`components/site-primitives.tsx` contains the shared button-link, badge,
callout, section-heading, inline-code, and arrow primitives. The shared shell
and navigation live in `site-header.tsx`, `site-footer.tsx`, and
`navigation-link.tsx`. Interactive behavior stays in small client components
for navigation, theme preference, copy controls, example filtering, and
documentation search. Focus rings and selected navigation states remain visible
in both themes.

The landing page introduces only implemented v0.1 behavior as current. It
separates the process-lifetime managed heap from future memory-control ideas,
shows the class/reference and struct/value distinction, and labels possible
future work as roadmap rather than shipped capability.

## Documentation architecture

Documentation is Markdown under `content/docs/`, organized by reader task:
getting started, language guide, compiler, reference, and project. The Language
Guide has separate v0.1 pages for syntax, types, expressions, control flow,
classes, constructors, methods, structs, lists, imports, and C interoperability.
The registry in `content/docs/navigation.ts` is the single source for page paths,
legacy path aliases, titles, descriptions, sidebar order, breadcrumbs,
active-page links, previous/next navigation, and GitHub edit links. Add a
Markdown file and one registry entry to publish a page; the route is statically
generated from that registry.

To add a page, create `content/docs/<section>/<slug>.md` using the `princi`
fence language for source examples, then add a `page(...)` entry to
`content/docs/navigation.ts` with its section, slug, title, description, and
keywords. Set `file` when its filename differs from the slug-derived default.
The registry drives the sidebar, generated route, breadcrumbs, search, metadata,
and previous/next links; don't add the page separately to those components.

The Compiler Internals section documents modules and stage boundaries against
the Rust source tree. The pipeline and future memory direction use small
server-rendered React diagrams rather than screenshots. Diagram slots in the
Markdown pages and the release-download slot in the installation guide are
expanded by `components/docs/docs-document.tsx`; diagram layout and
narrow-screen behavior live in `compiler-diagrams.module.css`.

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

### Documentation search, navigation, and metadata

`lib/docs-search-index.ts` builds a compact inverted index from the registered
Markdown pages during static generation. The static `/docs/search-index` route
serves that index locally; the browser fetches it only when search is first
opened so every docs page does not carry the full index. It indexes page titles,
descriptions, registry keywords, headings, and page text. The client palette
ranks title/heading matches first and can navigate to either a page or a
matching heading. It does not call a hosted search provider. Press **Ctrl+K** or
**Cmd+K** on a docs page to open the keyboard accessible native dialog; arrow
keys move through results, Enter opens one, and Escape closes it.

The docs shell keeps the navigation registry as its source for breadcrumbs,
active sidebar state, and previous/next links. The outline marks the section
nearest the current scroll position. Mobile uses native disclosure menus for
the docs sidebar and page outline. Heading controls copy a deep link, while code
copy reads the rendered code text so syntax highlighting does not alter the
clipboard contents. A docs loading skeleton and the shared not-found page cover
route transitions and broken links.

Each docs route emits a title in the form `Classes — PrinciPL Documentation`,
its registered description, Open Graph metadata, and Twitter card metadata.
The App Router uses `app/icon.svg` and generated Apple/social image routes. Docs
breadcrumbs add `BreadcrumbList` JSON-LD when the public origin is configured.
There is no analytics provider or tracking code.

The repository does not define a public website hostname. Set
`NEXT_PUBLIC_SITE_URL` to the actual deployed origin (for example, in the
deployment environment) to enable self-referencing canonical links, absolute
URLs in the sitemap, and the sitemap entry in `robots.txt`. Without it, the
site omits canonical URLs and the sitemap contains no entries rather than
publishing a guessed hostname. Next.js uses a local origin only to resolve
social image URLs during local development.

## Validation and deployment

The frontend package exposes `lint`, `build`, and `start` scripts; it has no
separate JavaScript unit-test script. Before deployment, install dependencies
and run:

```powershell
npm install
npm run lint
npm run build
npm run start
```

The website is an independent Next.js App Router project. Deploy `frontend/`
with a normal Node-compatible Next.js host: use `npm run build` for its build
step and `npm run start` for the server. No compiler binary, database,
authentication service, search provider, or analytics account is required.
Set `NEXT_PUBLIC_SITE_URL` to the final public origin in the deployment
environment so canonical URLs, sitemap entries, robots metadata, and absolute
social metadata point to the deployed site. The search index is generated from
Markdown at build time and served by the local static route; it does not need a
separate backend.

`cargo test` from the repository root tests the Princi compiler and its native
Windows integration behavior. It does not execute frontend JavaScript tests or
automatically compile Markdown snippets; when editing documentation examples,
check them with the repository's Windows compiler and inspect affected routes
at desktop and mobile widths.

The former `/installation`, `/language`, `/architecture`, and `/roadmap` URLs
redirect to the matching canonical documentation pages so content does not
need to be maintained twice. Keep claims aligned with the root `README.md` and
`docs/v0.1-scope.md`.
