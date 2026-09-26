---
name: instruction-audit
description: Audit recent Claude Code sessions for corrections the user keeps making and errors agents keep hitting, and propose changes to this repository's instructions, skills, rules and hooks.
argument-hint: "[since YYYY-MM-DD, default 30 days ago]"
disable-model-invocation: true
---

# Instruction audit

Find what still goes wrong between the user and agents, and propose fixes. **Change nothing until
the user picks** from your proposals.

## 1. Extract

```
python3 "${CLAUDE_SKILL_DIR}/extract_sessions.py" <scratchpad>/audit --since <date>
```

`<date>` is `$ARGUMENTS`, or 30 days ago. It covers the main checkout's sessions and every
worktree's, and writes `user_messages.txt` (what the user typed, per session, with `[COMPACTION]`
markers) and `tool_errors.txt` (tool errors grouped and counted). Both files are large: have
subagents read them, and keep only their conclusions.

## 2. Find the recurring problems

- **Corrections**: the user's messages that redo or undo agent work ("don't", "again", "I told
  you", "why did you", "instead", "remove", "revert", "simpler", "are you sure"). Group them by
  cause and count the sessions each appears in. Also check git history for commits where the user
  rewrote agent code without saying so in chat (`git log --author`, cleanup/removal commits).
- **Procedures** the user walked an agent through more than once.
- **Tool errors** that repeat across sessions (`tool_errors.txt`): each is a missing procedure or a
  wrong assumption.
- **Where it happened**: before or after a `[COMPACTION]` marker, and in subagents vs the main
  session.

## 3. Diagnose each against the current instructions

Read every `AGENTS.md` (`git ls-files '*AGENTS.md'`), `.claude/skills/`, `.claude/settings.json` hooks, and
the user's memory folder. For each problem, say which case it is:

| Case | Fix |
|---|---|
| A rule exists and was ignored | Enforce it: a hook, a lint, CI, or a checklist skill run at a fixed moment |
| A rule exists and agents overshoot it | Add the counter-example ("Too far") next to it |
| No rule, or the rule is only in private memory | Add one where the `capturing-instructions` skill says |
| A procedure keeps being rebuilt | A skill, with a script if the steps are mechanical |
| An instruction or memory is stale or wrong | Fix or delete it |

Also check the other direction: skills that never triggered, rules nobody needed, and instructions
contradicted by what the user asked for since.

## 4. Report

Give a numbered list, most sessions first. For each item: the problem, the evidence (short quotes,
session IDs), which case it is, the proposed change, and its size. Then stop and let the user pick.
