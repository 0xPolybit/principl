The repository contains two projects: the Rust compiler at the root and the separate Next.js website in `frontend/`. Compiler behavior and frontend documentation have separate test/build commands.

## Compiler

~~~powershell
cargo test
cargo build
cargo build --release
~~~

Windows native integration tests need LLVM/Clang and x86-64 MinGW-w64 GCC. Set `PRINCI_REQUIRE_NATIVE_TESTS=1` when native executable checks must fail instead of skipping on a machine without that toolchain.

## Website

~~~powershell
cd frontend
npm ci
npm run lint
npm run build
~~~

Documentation Markdown lives in `frontend/content/docs/`. Add page metadata and its file path to the single registry in `frontend/content/docs/navigation.ts`; the registry supplies sidebar order, routes, breadcrumbs, edit links, and previous/next links. Keep language claims aligned with the root README and `docs/v0.1-scope.md`.

No additional contribution workflow, release schedule, or code-of-conduct policy is defined by this guide.
