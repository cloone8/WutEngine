# WutEngine design

What WutEngine is meant to be and how it works: the design goals and the decisions that shape the engine's
architecture, each with a short reason. This book is the source of truth for the design. It describes the target, not
necessarily what is built yet.

Decisions are a first draft, API shapes especially. When building something shows that a decision doesn't work, the
decision is reopened and this book is changed with it. What is still undecided is listed in [Open questions](open.md);
features that plug into the design later without changing it are listed in [Later](later.md). [Planning](planning.md) is
the order to build it in.

Build it with `mdbook build`, or read it with `mdbook serve --open`, in the folder with `book.toml`.

NOTE: Contrary to the actual engine and the engine documentation, this document was mostly AI written during a
long series of design sessions. They'll probably be AI modified too, to be kept up to date.
