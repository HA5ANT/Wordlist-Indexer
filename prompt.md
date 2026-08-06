# `wl` — Phase 3, 4, 5

The app currently has a working core: indexing, exact/fuzzy search, listing,
config, stats, duplicates, verification, and an installer. The three phases
below build on top of that. Read the existing codebase first and make
implementation decisions that fit its current patterns — the descriptions
below are the goals and our suggested direction, not a spec to copy
literally.

Build in order: Phase 3 first, then 4, then 5 — each one depends on what the
previous phase produces.

---

# Phase 3 — Tags & Safe Schema Evolution

## What we want

Every indexed wordlist should be automatically categorized with tags based
on where it lives (its path/repo structure), so it can later be filtered by
purpose — "give me web wordlists," "give me SQLi payloads," etc. A single
wordlist can and should carry multiple tags where it makes sense (something
in a Fuzzing/XSS folder is both a fuzzing list and an XSS list).

Here's the taxonomy we want reflected — treat this table as the target
behavior, not literal code:

| Tag | Also implies | Applies when the path/filename indicates... |
|---|---|---|
| `discovery` | — | anything under Discovery |
| `webcontent` | `web`, `discovery` | web content discovery lists |
| `webshells` | `web` | web shell payloads |
| `dns` | `discovery` | DNS enumeration lists |
| `infra` | `discovery` | infrastructure discovery |
| `snmp` | `infra` | SNMP-related |
| `mainframe` | `infra` | mainframe-related |
| `iot` | `infra` | IoT/router/camera related |
| `ports` | `infra` | port lists |
| `fuzzing` | — | anything under Fuzzing |
| `xss` | `fuzzing`, `injection` | XSS-related |
| `sqli` | `fuzzing`, `injection` | SQL injection-related |
| `lfi` / `ssrf` / `ssti` / `xxe` / `cmdi` | `fuzzing`, `injection` | respective injection classes |
| `passwords` | — | anything under Passwords |
| `creds` | `passwords` | common credentials |
| `leaked` | `passwords` | leaked database dumps |
| `permutations` | `passwords` | password permutation generators |
| `usernames` | — | username lists |
| `patterns` | — | secret/regex pattern matching lists |
| `payloads` | — | generic payloads without a more specific match above |
| `ai` | — | LLM/AI testing lists |

On top of automatic tags, users should be able to manually add, remove, and
view tags on any entry — and manual tags need to survive re-indexing (an
update pass should never silently strip a tag someone added by hand).

We also want a filter that works consistently across the tool wherever it
makes sense to narrow results by category — searching, listing, and (later,
in Phase 5) browsing should all be able to say "only show me entries tagged
X" and combine multiple tags with OR logic.

Separately, and just as important: **schema changes must never be
destructive to a user's existing data going forward.** Right now an update
risks wiping someone's index or config. That has to stop being possible,
permanently — not just for this change, but for any future schema change
too.

## Suggested approach

- A comma-separated tags field on each entry is a reasonable default, but if
  a normalized structure fits the existing schema better, use your
  judgment.
- For schema safety, track some notion of schema version inside the
  database itself, and apply changes incrementally and additively based on
  it — never drop or recreate tables that hold user data. Back up the
  database file before altering it, so a failed update is always
  recoverable.
- The installer shouldn't be the thing responsible for migrating data
  anymore — it should just build and install the binary. Let the
  application itself detect and safely bring its own database up to date
  whenever it runs, so this is automatic and doesn't depend on the install
  script doing the right thing.

## What "done" looks like

- Updating from the current version to this one preserves an existing
  user's indexed data and config without any manual intervention
- A manually added tag is still present after the next index/update pass
- Tags applied to a representative sample of real SecLists-style paths
  match the intended taxonomy above
- A tag filter narrows results correctly when used standalone or combined
  with other tags

---

# Phase 4 — Search UX & Flag Safety

## What we want

**Stop showing full paths when a search or lookup returns more than one
result.** Instead, show something scannable — name, how many entries/lines
it has, its size — and identify each result by its actual, permanent
database identity (not a position in the current result list, which can
shift between queries). That identity should be usable later to fetch the
exact path for that one specific entry, on demand, via a dedicated lookup —
something like resolving one or more of those identities straight to their
paths, cleanly, for scripting.

**Flags need to behave predictably together, with no undefined
combinations.** Output-format flags shouldn't be combinable in ways that
produce ambiguous results (e.g. asking for two different output formats at
once should be rejected, not silently resolved by whichever one happens to
run first). Flags that narrow results by repo, extension, or tag should all
accept multiple comma-separated values consistently — not have one accept
lists and another not.

**Nothing ambiguous should ever leak into a script.** If a lookup could
resolve to more than one file, and the caller hasn't explicitly asked for
"just give me your best guess," and we're not in a live terminal where a
human can actually read a disambiguation prompt — the tool should refuse
and fail clearly rather than dump multiple lines of output into something
like a command substitution.

**Scripts need to be able to tell failures apart.** Not found, used wrong,
broke internally, and "this was ambiguous and needs to be resolved" are all
different situations and should be distinguishable by exit code, not just
"it exited non-zero." A "be quiet" flag should suppress explanatory
messages only — it should never change what exit code comes back.

## Suggested approach

- Use the entry's existing database identity as the number shown in
  multi-result output — don't invent a separate scheme.
- For line/entry counts, avoid reading files fully into memory or
  line-by-line for large wordlists; a fast byte-level scan is preferable.
  Skip this for compressed entries.
- Rust's standard library has a way to detect whether stdout is a real
  terminal versus being piped/captured — use that to decide whether to show
  an interactive disambiguation view or fail loud.
- We'd suggest something like: 0 for success, 1 for not found, 2 for
  invalid flag usage, 3 for internal errors, 4 for "ambiguous, needs
  resolving" — but pick whatever set makes sense as long as it's consistent
  and documented.

## What "done" looks like

- A query with multiple matches shows a compact, scannable result set keyed
  by stable identities, not raw paths
- Resolving a specific result to its path works reliably and prints nothing
  but that path
- Two conflicting output-format flags together fail clearly instead of
  silently picking one
- Filtering by multiple repos, extensions, or tags at once works the same
  way across all of them
- An ambiguous, non-interactive lookup fails loudly instead of producing
  multi-line output that could corrupt a downstream command
- Different failure situations (not found / bad usage / ambiguous /
  internal error) return different, documented exit codes

---

# Phase 5 — Interactive Browse

## What we want

A full-screen interactive mode where we can fuzzy-search across the entire
collection live, see enough context to recognize the right file (size,
location, maybe a preview of metadata), and select one with the keyboard.
Selecting should end the interactive session and print exactly one clean
path — nothing else — so it can be used directly in place of a manual
lookup, the same way our other single-result outputs already work for
scripting.

It should support the same kind of tag-based pre-filtering as the rest of
the tool, so we can jump straight into browsing just the web-related lists,
for example.

Canceling out of it (without selecting anything) should produce no output
and a clear, consistent "no result" signal — same idea as any other command
that comes up empty.

## Suggested approach

- A terminal UI crate suited to this kind of live, keyboard-driven
  interface would be a good fit — reuse the fuzzy-matching logic that
  search already uses rather than building a second implementation of it.
- Whatever you build needs to reliably restore the user's terminal to a
  normal state on exit, including if something goes wrong mid-render — a
  broken terminal after a crash is a bad experience.
- Nothing from the interactive view itself should touch normal output —
  only the final selected path, printed after the interactive view has
  fully closed.

## What "done" looks like

- Selecting an entry produces exactly one line of output: its path
- Canceling produces no output and the appropriate "nothing selected"
  exit behavior
- Pre-filtering by tag correctly narrows what shows up before any typing
  happens

---

# Not part of this round

- Redesigning how the existing table-style listing renders (columns,
  wrapping, etc.) — not planned right now, browsing interactively covers
  that need instead
- Copying a result to the clipboard — an idea we've floated, not something
  to build yet
- Any kind of quick-filter chips inside the interactive browser beyond
  what's needed for the tag pre-filter above — nice to have if it falls out
  naturally, not a requirement
