export type DocsPage = {
  slug: string;
  aliases?: string[];
  title: string;
  description: string;
  keywords: string[];
  section: string;
  file: string;
};

export type DocsSection = {
  title: string;
  pages: DocsPage[];
};

function page(
  section: string,
  slug: string,
  title: string,
  description: string,
  options: { file?: string; aliases?: string[]; keywords?: string[] } = {},
): DocsPage {
  const file = options.file ?? (slug || "getting-started/introduction");
  return {
    section,
    slug,
    title,
    description,
    keywords: options.keywords ?? [title, section, ...slug.split("/").filter(Boolean)],
    file: file + ".md",
    aliases: options.aliases,
  };
}

export const docsSections: DocsSection[] = [
  {
    title: "Getting started",
    pages: [
      page("Getting started", "", "Introduction", "What Princi is, what v0.1 supports, and where to go next."),
      page("Getting started", "getting-started", "Getting started", "Set up Princi, build the compiler, and compile your first Windows program.", { file: "getting-started/index" }),
      page("Getting started", "installation", "Installation", "Set up Rust, LLVM/Clang, MinGW-w64, and the Princi compiler.", { file: "getting-started/installation", aliases: ["getting-started/installation"] }),
      page("Getting started", "hello-world", "Hello, world", "Build and run your first Princi program.", { file: "getting-started/hello-world", aliases: ["getting-started/hello-world"] }),
      page("Getting started", "compiling", "Compiling programs", "Use the v0.1 build command and choose an executable output path.", { file: "getting-started/compiling-programs", aliases: ["getting-started/compiling-programs"] }),
    ],
  },
  {
    title: "Language guide",
    pages: [
      page("Language guide", "language/syntax", "Syntax", "Declarations, blocks, expressions, and statement boundaries."),
      page("Language guide", "language/variables", "Variables", "Typed and inferred locals, mutability, and initialization."),
      page("Language guide", "language/primitive-types", "Primitive types", "The Int, Float, Bool, String, and Void types."),
      page("Language guide", "language/operators", "Operators", "The operators and conversion rules in v0.1."),
      page("Language guide", "language/strings", "Strings", "UTF-8 string values, escapes, concatenation, equality, and output."),
      page("Language guide", "language/functions", "Functions", "Parameters, return types, calls, and recursion."),
      page("Language guide", "language/control-flow", "Control flow", "Conditions, branches, and returns."),
      page("Language guide", "language/loops", "Loops", "While loops and end-exclusive integer range loops."),
      page("Language guide", "language/classes", "Classes", "Reference-oriented class types and instance layout."),
      page("Language guide", "language/constructors", "Constructors", "Class init constructors and positional or named construction."),
      page("Language guide", "language/methods", "Methods", "Instance methods, self, field access, and static dispatch."),
      page("Language guide", "language/structs", "Structs", "Field-only structs with value-copy semantics."),
      page("Language guide", "language/lists", "Lists", "The built-in List<T> collection and its checked operations.", { keywords: ["List<T>", "collection", "indexing", "bounds checks", "add", "length"] }),
      page("Language guide", "language/imports", "Imports", "The fixed built-in io and math module registry."),
      page("Language guide", "language/c-ffi", "C interoperability", "The restricted Windows x64 C ABI boundary.", { keywords: ["FFI", "foreign function interface", "extern C", "native ABI"] }),
    ],
  },
  {
    title: "Compiler Internals",
    pages: [
      page("Compiler Internals", "compiler/architecture", "Compiler Overview", "The Rust compiler modules and how data moves between them."),
      page("Compiler Internals", "compiler/compilation-pipeline", "Compilation Pipeline", "Follow source through LLVM IR to a Windows executable."),
      page("Compiler Internals", "compiler/lexer-parser", "Lexer & Parser", "How source characters become located tokens and syntax."),
      page("Compiler Internals", "compiler/ast", "AST", "The source-oriented syntax tree and its preserved spans."),
      page("Compiler Internals", "compiler/semantic-analysis", "Semantic Analysis", "Name resolution, scopes, and language-rule checks."),
      page("Compiler Internals", "compiler/type-system", "Type System", "Source and compiler-internal type representations."),
      page("Compiler Internals", "compiler/llvm-backend", "LLVM Backend", "Lowering the typed program into verified target-aware LLVM IR."),
      page("Compiler Internals", "compiler/windows-toolchain", "Windows Linking", "Emit Windows COFF with Clang and link with MinGW-w64 GCC."),
      page("Compiler Internals", "compiler/runtime", "Runtime", "Built-ins, process startup, and platform runtime helpers."),
      page("Compiler Internals", "compiler/managed-memory", "Managed Memory", "The current process-lifetime allocator and future direction."),
      page("Compiler Internals", "compiler/diagnostics", "Diagnostics", "Source diagnostics, stable codes, and toolchain failures."),
      page("Compiler Internals", "compiler/c-ffi-boundary", "C FFI Boundary", "How restricted C ABI types cross into Princi code."),
    ],
  },
  {
    title: "Reference",
    pages: [
      page("Reference", "reference/lexical-syntax", "Lexical syntax", "Tokens, comments, literals, and operators recognized by the lexer."),
      page("Reference", "reference/built-in-types", "Built-in types", "Source types and the supported List<T> specialization."),
      page("Reference", "reference/built-in-functions", "Built-in functions", "Primitive output built-ins and their accepted values."),
      page("Reference", "reference/cli", "Command-line interface", "The complete user-facing v0.1 CLI."),
      page("Reference", "reference/file-extensions", "File extensions", "The interchangeable .prnc and .princi suffixes."),
      page("Reference", "reference/limitations", "Limitations", "A concise list of what v0.1 does and does not compile."),
    ],
  },
  {
    title: "Project",
    pages: [
      page("Project", "project/roadmap", "Roadmap", "Areas intentionally deferred beyond v0.1."),
      page("Project", "project/contributing", "Contributing", "Build, test, and change the compiler and website."),
    ],
  },
];

export const docsPages = docsSections.flatMap((section) => section.pages);

export function docsHref(doc: DocsPage) {
  return doc.slug ? "/docs/" + doc.slug : "/docs";
}

export function findDocsPage(slug: string) {
  return docsPages.find((doc) => doc.slug === slug || doc.aliases?.includes(slug));
}

export function findDocsSection(doc: DocsPage) {
  return docsSections.find((section) => section.title === doc.section);
}

export function docsNeighbors(doc: DocsPage) {
  const index = docsPages.findIndex((entry) => entry.slug === doc.slug);
  return {
    previous: index > 0 ? docsPages[index - 1] : undefined,
    next: index >= 0 && index < docsPages.length - 1 ? docsPages[index + 1] : undefined,
  };
}

export function docsRouteParams() {
  return docsPages.flatMap((doc) => [doc.slug, ...(doc.aliases ?? [])])
    .filter((slug) => slug.length > 0)
    .map((slug) => ({ slug: slug.split("/") }));
}
