---
name: handoff
description: Use when the user asks for a handoff, or asks to prepare the work so it can be continued on another machine, in another session or by another agent without your memories.
argument-hint: "[file name, default PENDING.md]"
---

# Handoff

Write a handoff document now, then keep it current for the rest of the session **without asking
permission**. It's for an agent on another machine: no memories, no conversation, no scratchpad,
no `.claude/notes/`. Anything that reader needs goes in the document.

## Where

`$ARGUMENTS` if given, else `PENDING.md`, at the root of the worktree you're working in (the
repository root if there's no worktree). Git must not ignore it (`git check-ignore -v <file>`
prints nothing). **Never commit it**; the user decides. If a hook staged it, `git reset <file>`.
If one already exists, read it and rewrite it; don't start a second one.

## What it contains, in this order

1. **Title** naming the task, then: branch, worktree, base (`git merge-base HEAD master`), and when
   it was last updated.
2. **State in one paragraph:** what's done, what isn't, and the single next step.
3. **Next steps**, ordered, each concrete: the file, command or check, and what result means what.
4. **Decisions**: the ones taken, with the user's reason; and the open ones only the user can make.
5. **Uncommitted work**: what the diff against the base contains and which parts are unfinished.
6. **Findings**: facts with `path:line @ <short-sha>` (`@ <sha>+` for uncommitted lines),
   hypotheses ruled out and why, and measurements. Keep code excerpts short.
7. **How to verify**: build and test commands, what needs a window or GPU, and the expected
   result.
8. **Context the reader lacks**: working agreements from this session (e.g. "the user wants the reformat
   kept separate from the feature"), relevant facts from your memories, and anything outside the repo: logs,
   installs or files on this machine, marked as such, since they may not exist over there.

## Keeping it current

Update it in the same turn whenever something changes it: a finding, a fix, a decision, a
disproved claim, a finished or reordered step. Rewrite it in place, so it always reads as the
current state, not as a log. Delete what's stale or settled: the reader acts on everything still
in it. After an update, tell the user in one line that the handoff was refreshed and left
uncommitted.

When the user says the work is done, or asks for it, delete the document.
