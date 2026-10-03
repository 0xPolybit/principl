import type { ReactNode } from "react";

const keywords = new Set([
  "fn",
  "return",
  "let",
  "var",
  "if",
  "else",
  "while",
  "for",
  "in",
  "class",
  "struct",
  "init",
  "self",
  "import",
  "extern",
]);

const types = new Set([
  "Int",
  "Float",
  "Bool",
  "String",
  "Void",
  "List",
  "Int32",
  "Int64",
  "Float64",
]);

type HighlightToken = { text: string; kind?: string };

function identifierAt(source: string, offset: number) {
  return /^[\p{L}_][\p{L}\p{N}_]*/u.exec(source.slice(offset))?.[0];
}

function numberAt(source: string, offset: number) {
  return /^(?:\d+\.\d+(?:[eE][+-]?\d+)?|\d+[eE][+-]?\d+|\d+)/.exec(
    source.slice(offset),
  )?.[0];
}

function tokenize(source: string): HighlightToken[] {
  const tokens: HighlightToken[] = [];
  let offset = 0;

  while (offset < source.length) {
    const start = offset;
    const character = source[offset];

    if (/\s/.test(character)) {
      while (offset < source.length && /\s/.test(source[offset])) offset += 1;
      tokens.push({ text: source.slice(start, offset) });
      continue;
    }

    if (source.startsWith("//", offset)) {
      const end = source.indexOf("\n", offset);
      offset = end === -1 ? source.length : end;
      tokens.push({ text: source.slice(start, offset), kind: "comment" });
      continue;
    }

    if (character === '"') {
      offset += 1;
      while (offset < source.length && source[offset] !== "\n") {
        if (source[offset] === "\\") {
          offset = Math.min(source.length, offset + 2);
        } else if (source[offset++] === '"') {
          break;
        }
      }
      tokens.push({ text: source.slice(start, offset), kind: "string" });
      continue;
    }

    const number = numberAt(source, offset);
    if (number) {
      offset += number.length;
      tokens.push({ text: number, kind: "number" });
      continue;
    }

    const identifier = identifierAt(source, offset);
    if (identifier) {
      offset += identifier.length;
      const next = source.slice(offset).match(/^\s*/)?.[0].length ?? 0;
      const nextCharacter = source[offset + next];
      const kind = identifier === "true" || identifier === "false"
        ? "boolean"
        : keywords.has(identifier)
          ? "keyword"
          : types.has(identifier)
            ? "type"
            : nextCharacter === "("
              ? "function"
              : undefined;
      tokens.push({ text: identifier, kind });
      continue;
    }

    const operator = ["->", "==", "!=", "<=", ">=", "&&", "||", "+=", "-=", "*=", ".."].find(
      (candidate) => source.startsWith(candidate, offset),
    );
    if (operator) {
      offset += operator.length;
      tokens.push({ text: operator, kind: "operator" });
      continue;
    }

    offset += 1;
    tokens.push({
      text: character,
      kind: "+-*/%=<>!(){}[],.:;".includes(character)
        ? "operator"
        : undefined,
    });
  }

  return tokens;
}

export function PrinciCode({ source }: { source: string }) {
  const nodes: ReactNode[] = [];

  for (const [index, token] of tokenize(source).entries()) {
    nodes.push(
      token.kind ? (
        <span className={`princi-token-${token.kind}`} key={index}>
          {token.text}
        </span>
      ) : (
        <span key={index}>{token.text}</span>
      ),
    );
  }

  return (
    <pre className="princi-code-window">
      <code>{nodes}</code>
    </pre>
  );
}
