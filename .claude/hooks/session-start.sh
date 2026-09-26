#!/usr/bin/env bash
# SessionStart hook (startup, resume, clear and after compaction): points the agent at the branch's state documents,
# which nothing else mentions after a compaction or on another machine, and lists the tools the repo's hooks and
# skills need but this machine lacks.
input=$(cat)
re='"cwd"[[:space:]]*:[[:space:]]*"([^"]*)"'
[[ $input =~ $re ]] && dir=${BASH_REMATCH[1]}
root=$(git -C "${dir:-$CLAUDE_PROJECT_DIR}" rev-parse --show-toplevel 2>/dev/null) || exit 0

# State documents are Markdown files at the worktree root that git neither tracks nor ignores (see the handoff skill),
# minus instruction files, plus PENDING.md even if it was committed on a WIP commit.
docs=$(git -C "$root" ls-files --others --exclude-standard -- '*.md' | grep -v -e / -e '^AGENTS' -e '^CLAUDE')
[[ -f $root/PENDING.md ]] && docs=$(printf '%s\n%s' "$docs" PENDING.md | sort -u)
docs=$(sed '/^$/d' <<<"$docs")
if [[ -n $docs ]]; then
    echo "State documents at the root of $root, left there by an earlier session:"
    sed 's/^/- /' <<<"$docs"
    echo "Read them before continuing this work, and keep them current as the handoff skill describes."
fi

missing=()
rustup run nightly rustfmt --version >/dev/null 2>&1 ||
    missing+=('nightly rustfmt (`rustup toolchain install nightly --profile minimal --component rustfmt`): the format hook, `cargo +nightly fmt`')
command -v python3 >/dev/null ||
    missing+=('Python 3: the instruction-audit skill')
if ((${#missing[@]})); then
    echo "Tools this repository's hooks and skills use are missing on this machine:"
    printf -- '- %s\n' "${missing[@]}"
    echo "Tell the user about a missing tool as soon as a task needs it, with how to install it."
fi
exit 0
