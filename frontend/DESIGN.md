# PrinciPL website design direction

The visual world is a compact compiler field guide: the legibility and exact
labels of a toolchain interface, with the information hierarchy of good
technical documentation. It should feel at home beside systems-language and
compiler projects without borrowing their brand patterns. The site helps
developers evaluate and try Princi; every language claim remains tied to the
repository's documented v0.1 behavior.

## Visual system

- A quiet graphite-and-paper palette keeps source, commands, and scope limits
  easy to scan. Burnt copper marks important actions; a restrained citron
  signal is reserved for small status details.
- DM Sans handles interface and reading text. IBM Plex Mono is reserved for
  code, command lines, filenames, and compact technical labels. Both are
  bundled locally.
- A compact top navigation, ruled content sections, text-led feature cards,
  and small build/source panels keep pages relatively dense without losing a
  clear reading measure.
- Light, dark, and system themes use semantic color tokens. System follows the
  operating-system preference; a chosen setting is stored locally.
- Focus rings, selected navigation, tab state, and control labels stay visible
  in both themes. Mobile navigation remains in the keyboard order and closes
  with Escape.
- No decorative gradients, fake product capabilities, or external font
  requests. Code and examples describe only features present in the v0.1
  scope.

## Shared shell and components

The global shell provides Home, Docs, Learn, Examples, GitHub, a v0.1 marker,
the theme selector, responsive menu, and a footer with compiler-target details.
Reusable components include button links, badges, cards, callouts, section
headings, feature grids, inline code, navigation links, code samples, copy
controls, and keyboard-operable tabs. The pages remain Server Components;
client code is limited to the navigation menu, theme preference, copy
control, and tabs.
