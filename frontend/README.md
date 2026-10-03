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
| `components/` | Reusable navigation, article, code sample, and copy controls |
| `content/` | Compiler-grounded feature summaries and example source strings |
| `lib/` | Site navigation and shared helpers |
| `public/` | Public static assets |
| `styles/` | Shared design tokens |
| `briefs/` | Internal page briefs used during site design |
| `licenses/` | Third-party font license texts |

The landing page and guide pages are Server Components. The responsive menu
and code-copy control are small Client Components because they use browser
interaction. Newsreader, DM Sans, and IBM Plex Mono are bundled locally from
Fontsource packages; their SIL Open Font License texts are in `licenses/`.
