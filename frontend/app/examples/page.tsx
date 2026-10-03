import { DocShell } from "@/components/doc-shell";
import { ExampleGallery } from "@/components/examples/example-gallery";
import { princiExamples } from "@/content/examples";
import { pageTitle } from "@/lib/site";
import { publicPageMetadata } from "@/lib/site-metadata";

export const metadata = publicPageMetadata(
  pageTitle("Examples"),
  "Browse and copy Princi v0.1 programs for functions, control flow, classes, structs, lists, imports, and the limited C FFI.",
  "/examples",
);

export default function ExamplesPage() {
  return (
    <DocShell
      description="Small, complete programs that use the current v0.1 compiler. Search by concept, copy a source file, and follow the language guide for more detail."
      note="Princi source gallery"
      title="Examples"
    >
      <div className="example-build-note">
        <p>
          Save a card using its shown <code>.prnc</code> or <code>.princi</code> filename, then
          build it with the Windows x86-64 compiler:
        </p>
        <code className="example-build-command">princi build hello.prnc</code>
        <span><code>.prnc</code> and <code>.princi</code> are interchangeable aliases for the same source language.</span>
        <span>Each output panel is static expected output; the website does not run Princi code.</span>
      </div>
      <ExampleGallery examples={princiExamples} />
    </DocShell>
  );
}
