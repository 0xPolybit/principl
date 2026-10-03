# PrinciPL website product record

<!-- impeccable:product-schema 1 -->

## Platform

web

## Stack

Next.js App Router, TypeScript, React, Tailwind CSS, ESLint. The user selected
this stack for the standalone `/frontend` website.

## Users

Developers evaluating and trying Princi. The site should help them understand
the current language boundary and reach a first local build.

## Product Purpose

Introduce the Princi programming language and provide a reliable guide to its
v0.1 syntax, compiler architecture, Windows setup, and examples.

## Positioning

Princi v0.1 is a statically typed language compiled by a Rust compiler to
Windows x86-64 native executables. The language and compiler scope are still
early and intentionally limited.

## Capabilities and Constraints

- The compiler accepts equivalent `.prnc` and `.princi` source files.
- The sole v0.1 compiler command is `princi build <source-file>` with an
  optional `-o <path>` output.
- The only native target is Windows x86-64.
- Feature claims must agree with `../README.md` and
  `../docs/v0.1-scope.md`; planned features must be labeled as deferred.
- The frontend is a separate Next.js application under `/frontend`; compiler
  behavior and Rust sources are outside its scope.

## Evidence on Hand

The repository README, v0.1 scope document, `examples/modules.prnc`, compiler
source, and automated test fixtures. No customer testimonials, benchmarks,
download statistics, or release binaries are provided.
