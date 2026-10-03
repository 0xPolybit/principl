export type DocsPage = {
  slug: string;
  aliases?: string[];
  title: string;
  description: string;
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
  options: { file?: string; aliases?: string[] } = {},
): DocsPage {
  const file = options.file ?? (slug || "getting-started/introduction");
  return {
    section,
    slug,
    title,
    description,
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
      page("Language guide", "language/control-flow", "Control flow", "Conditions, loops, ranges, and returns."),
      page("Language guide", "language/functions", "Functions", "Parameters, return types, calls, and recursion."),
      page("Language guide", "language/strings", "Strings", "String literals, output, equality, and concatenation."),
      page("Language guide", "language/classes", "Classes", "Reference-oriented classes, constructors, fields, and methods."),
      page("Language guide", "language/structs", "Structs", "Field-only structs with value-copy semantics."),
      page("Language guide", "language/lists", "Lists", "The built-in List<T> collection and its checked operations."),
      page("Language guide", "language/imports", "Imports", "The fixed built-in io and math module registry."),
      page("Language guide", "language/c-ffi", "C interoperability", "The restricted Windows x64 C ABI boundary."),
    ],
  },
  {
    title: "Compiler",
    pages: [
      page("Compiler", "compiler/architecture", "Architecture", "The compiler modules and their one-way responsibilities."),
      page("Compiler", "compiler/compilation-pipeline", "Compilation pipeline", "Follow source through verified LLVM IR to a Windows executable."),
      page("Compiler", "compiler/diagnostics", "Diagnostics", "Understand source errors and toolchain failures."),
      page("Compiler", "compiler/managed-memory", "Managed memory", "The process-lifetime allocator used in v0.1."),
      page("Compiler", "compiler/windows-toolchain", "Windows toolchain", "Install and validate LLVM/Clang and MinGW-w64."),
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
