#!/usr/bin/env bash
# PostToolUse hook: formats every Rust file Claude edits or writes with nightly rustfmt. Stable rustfmt skips the
# unstable options in rustfmt.toml (imports_granularity) with only a warning, so it can't be trusted to enforce them.
input=$(cat)
# Plain bash instead of jq. Inside a written file's content the key's quotes are escaped, so only tool_input's own
# "file_path" matches.
re='"file_path"[[:space:]]*:[[:space:]]*"([^"]*\.rs)"'
[[ $input =~ $re ]] || exit 0
file=${BASH_REMATCH[1]}
# Only this repository's own code: not scratch files outside it, not the vendored puffin_egui fork.
git -C "$(dirname "$file")" rev-parse --show-toplevel >/dev/null 2>&1 || exit 0
[[ $file == */wutengine_puffin_egui/* ]] && exit 0

# skip_children: format only the edited file, not the out-of-line modules it declares.
out=$(rustfmt +nightly --edition 2024 --config skip_children=true "$file" 2>&1) || {
    printf 'formatting failed on %s:\n%s\n' "$file" "$out" >&2
    exit 2
}
