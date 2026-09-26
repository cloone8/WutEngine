"""Extracts what the user typed, and the tool errors that repeat, from Claude Code session transcripts.

Usage: python3 extract_sessions.py <out_dir> [--since YYYY-MM-DD] [--project <path>]

Writes <out_dir>/user_messages.txt and <out_dir>/tool_errors.txt and prints a summary. Transcripts live in
~/.claude/projects/<mangled path>/*.jsonl, one folder per checkout or worktree path that sessions started in.
"""

import argparse
import collections
import json
import pathlib
import re
import subprocess

parser = argparse.ArgumentParser()
parser.add_argument("out_dir", type=pathlib.Path)
parser.add_argument("--since", default="", help="only sessions active on or after this date")
parser.add_argument("--project", help="checkout path; defaults to this repository's main checkout")
args = parser.parse_args()

if args.project:
    project = args.project
else:
    # The first `git worktree list` entry is the main checkout, also when run from a worktree.
    listing = subprocess.run(["git", "worktree", "list", "--porcelain"], capture_output=True, text=True, check=True)
    project = listing.stdout.splitlines()[0].removeprefix("worktree ")
# Claude Code names the folder after the path with every non-alphanumeric character replaced by `-`.
# Worktree sessions get folders that extend the main checkout's name, so a prefix match finds them too.
prefix = re.sub(r"[^A-Za-z0-9]", "-", str(pathlib.Path(project).resolve()))
projects = pathlib.Path.home() / ".claude" / "projects"
folders = sorted(d for d in projects.iterdir() if d.is_dir() and d.name.lower().startswith(prefix.lower()))


def text_of(content):
    if isinstance(content, str):
        return content
    return "\n".join(p.get("text", "") for p in content if isinstance(p, dict) and p.get("type") == "text")


messages, seen, sessions = [], set(), 0
errors = collections.Counter()
for folder in folders:
    # Subagent transcripts live in subfolders; only the top-level ones hold what the user typed.
    for f in sorted(folder.glob("*.jsonl")):
        lines = f.read_text(encoding="utf-8", errors="replace").splitlines()
        records = []
        for line in lines:
            try:
                records.append(json.loads(line))
            except json.JSONDecodeError:
                continue
        if args.since and not any(r.get("timestamp", "") >= args.since for r in records):
            continue
        sessions += 1
        for i, r in enumerate(records):
            if r.get("type") != "user" or r.get("isSidechain") or r.get("isMeta"):
                continue
            content = r.get("message", {}).get("content")
            for part in content if isinstance(content, list) else []:
                if isinstance(part, dict) and part.get("type") == "tool_result" and part.get("is_error"):
                    lines_ = [x for x in text_of(part.get("content") or "").splitlines() if x.strip()]
                    # A bare "Exit code 1" says nothing; the line after it holds the actual error.
                    if len(lines_) > 1 and re.fullmatch(r"Exit code \d+", lines_[0].strip()):
                        lines_ = lines_[1:]
                    first = lines_[0][:160] if lines_ else "(empty)"
                    # Digits and worktree names vary per occurrence; normalize them so repeats group.
                    first = re.sub(r"worktrees[\\/][\w.-]+", "worktrees/<wt>", re.sub(r"\d+", "N", first))
                    errors[first[:140]] += 1
            t = text_of(content or "").strip()
            if not t or t.startswith(("<command-", "<local-command", "<task-notification>", "Caveat:")):
                continue
            if t.startswith("This session is being continued"):
                t = "[COMPACTION]"
            elif t[:200] in seen:
                continue
            seen.add(t[:200])
            messages.append(f"=== {folder.name} {f.stem[:8]} {i}/{len(records)} {r.get('timestamp', '')[:10]} ===\n{t[:4000]}\n")

args.out_dir.mkdir(parents=True, exist_ok=True)
(args.out_dir / "user_messages.txt").write_text("\n".join(messages), encoding="utf-8")
(args.out_dir / "tool_errors.txt").write_text(
    "\n".join(f"{n:5}  {e}" for e, n in errors.most_common()), encoding="utf-8"
)
print(f"{len(folders)} folders, {sessions} sessions, {len(messages)} user messages, {sum(errors.values())} tool errors")
