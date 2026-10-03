import styles from "./compiler-diagrams.module.css";

const pipelineSteps = [
  [".prnc / .princi", "Interchangeable source file extensions"],
  ["Source", "UTF-8 text with indexed source locations"],
  ["Lexer", "Located tokens"],
  ["Parser", "Syntax analysis with recovery"],
  ["AST", "Span-preserving source tree"],
  ["Module resolution", "Fixed built-in io and math registry"],
  ["Semantic analysis", "Names, scopes, calls, and language rules"],
  ["Typed representation", "AST plus resolved symbols and expression types"],
  ["LLVM IR", "Windows target triple and runtime helpers"],
  ["Windows x86-64 COFF", "Object emitted by LLVM/Clang"],
  ["MinGW-w64 linker", "GCC links the object and Windows C runtime"],
  [".exe", "Native Windows x86-64 executable"],
];

const memoryStages = [
  {
    title: "Managed",
    status: "CURRENT · v0.1",
    description: "Runtime registers heap blocks and releases them together when the program exits.",
    current: true,
  },
  {
    title: "Deterministic / owned",
    status: "FUTURE DIRECTION",
    description: "A possible way to make some value lifetimes explicit and deterministic.",
    current: false,
  },
  {
    title: "Arena",
    status: "FUTURE DIRECTION",
    description: "A possible scoped allocation strategy for groups of short-lived values.",
    current: false,
  },
  {
    title: "Raw / manual",
    status: "FUTURE DIRECTION",
    description: "A possible low-level control layer, not a v0.1 source feature.",
    current: false,
  },
];

export function CompilerPipelineDiagram() {
  return (
    <figure aria-labelledby="pipeline-diagram-caption" className={styles.pipelineFigure}>
      <figcaption className={styles.figureCaption} id="pipeline-diagram-caption">
        <span>Build path</span>
        <span>Princi source to native Windows executable</span>
      </figcaption>
      <ol aria-label="Princi v0.1 compiler pipeline" className={styles.pipelineSteps}>
        {pipelineSteps.map(([title, description], index) => (
          <li className={styles.pipelineStep} key={title}>
            <span aria-hidden="true" className={styles.stepNumber}>{String(index + 1).padStart(2, "0")}</span>
            <span className={styles.stepCopy}>
              <strong>{title}</strong>
              <small>{description}</small>
            </span>
          </li>
        ))}
      </ol>
    </figure>
  );
}

export function MemoryRoadmapDiagram() {
  return (
    <figure aria-labelledby="memory-roadmap-caption" className={styles.memoryFigure}>
      <figcaption className={styles.figureCaption} id="memory-roadmap-caption">
        <span>Roadmap concept</span>
        <span>Possible progression of memory control</span>
      </figcaption>
      <ol aria-label="Future memory model direction, not a v0.1 feature list" className={styles.memoryStages}>
        {memoryStages.map((stage) => (
          <li className={stage.current ? styles.currentStage : styles.futureStage} key={stage.title}>
            <span className={styles.stageStatus}>{stage.status}</span>
            <strong>{stage.title}</strong>
            <small>{stage.description}</small>
          </li>
        ))}
      </ol>
      <p className={styles.roadmapNote}>
        Only <strong>Managed</strong> describes the v0.1 implementation. The remaining stages are concepts for future versions, not available APIs or a committed release schedule.
      </p>
    </figure>
  );
}
