---
name: capturing-instructions
description: Use when the user gives a correction, preference or convention meant for future sessions ("always", "never", "from now on", or a remark they have made before), when they walk you through a multi-step procedure, when you catch yourself repeating a workaround from an earlier session, when a repo skill, AGENTS.md line, hook or script turned out wrong, or when creating or editing a skill, hook or AGENTS.md instruction in this repository.
---

# Capturing instructions

Private memory holds what's true for this user or machine. Project knowledge belongs in the repo, where
every machine, session and subagent gets it. When something should outlive the session, pick its
home from the table and **suggest it in one line at the end of your reply**: "This could be a
`<name>` skill / an AGENTS.md line / a hook: <what it would hold>. Want me to add it?" Only write it
once the user agrees. If an existing skill or AGENTS.md line already covers the topic, suggest
editing that instead.

## Where it goes

| The instruction is... | Home |
|---|---|
| Checkable by a tool (formatting, a lint, a file pattern) | A hook in `.claude/settings.json`, or a lint in `Cargo.toml` `[workspace.lints]`, `clippy.toml` or `rustfmt.toml`. Wording alone gets ignored: stable `cargo fmt` passes while silently skipping `imports_granularity`. |
| Needed while working anywhere in the repo | Root `AGENTS.md`, one line |
| Needed while working in one area | That area's `AGENTS.md` (`runtime/`, `asset/`, `editor/`, `tools/`, or a crate's own). It loads when a file under its directory is read. Never create a `CLAUDE.md`: it turns off every `AGENTS.md`. |
| A procedure with steps, a checklist run at a fixed moment, or reference material | `.claude/skills/<name>/SKILL.md` |
| A procedure only the user may start (it acts outside the repo, or costs a build) | A skill with `disable-model-invocation: true` |
| Only true for this user or machine (paths on this disk, personal preferences) | Memory |

A skill's body loads only when it's invoked, and after compaction only its first 5,000 tokens come
back. A rule the agent must follow while typing can't live in a skill alone: put a one-line trigger
in `AGENTS.md` ("before reporting Rust changes as done, run `rust-review`"), with the detail in the
skill.

## Writing a skill

- `name`: lowercase with hyphens. `description`: third person, starting "Use when...". It lists
  **only triggering situations and the error texts or symptoms an agent would search for**, never
  the steps: agents follow a summarized workflow instead of reading the skill.
- `paths:` (a YAML list of globs) when it only applies to some files.
- The body is a recipe: what to do, in order, with the one example that shows the pattern. Name the
  wrong result next to the right one when agents overshoot (see `rust-review`'s "Too far" list).
  Stay under ~500 words; put scripts and long reference in files next to `SKILL.md`, invoked
  through their interpreter.
- Ground it in evidence: the user's actual remarks, or a failure you reproduced. Verify every
  command and path it names.
- Leave one check behind that fails if the skill is wrong: run its script, or give a fresh subagent
  a task that should trigger it without naming the skill, and read what it did.
- When it replaces a memory, say which memory can go once the change is committed.
