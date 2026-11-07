# Coding from regular chats through n8n

The private [Sandbox Coding Gateway](https://n8n.pchrisoc.com/workflow/lO6EIO8ZNqcmQyMY)
is available through the connected n8n app in the owner's personal project. It is an
on-demand tool for the existing sandbox VM. It has no public chat endpoint or schedule.

In a regular chat, select **n8n** and ask:

> Use Sandbox Coding Gateway to work on SimplyPDF. Start with help and doctor,
> prepare a fresh task checkout, read its repository instructions, implement this task,
> and run the relevant verification. Report the final job result and any remaining gaps.

## Tool sequence

1. Find the workflow by its name and read its details. Execute it in **manual** mode.
   Supply chat input as a JSON string, for example `{"action":"help"}` or
   `{"action":"doctor"}`. Do not edit an existing SSH workflow's command to run a task.
   The first response is an n8n execution ID. Retrieve that execution with `includeData:
   true` and `nodeNames: ["Return Explicit Result"]` to read the result without loading
   every node's data. If the execution is still running, wait and retrieve it again.
2. Send `{"action":"prepare","repo":"pchrisoc/simplypdf"}`. Keep the returned session ID,
   branch, base SHA, and package names. Every task receives its own checkout and target
   directory; existing projects and unsaved work are preserved. Preparation fetches the
   current remote revision and provisions SimplyPDF's pinned font and translation inputs.
3. Use `read` to inspect `AGENTS.md`, `CLAUDE.md`, the task acceptance criteria, and relevant
   source. `search` uses ripgrep and returns bounded excerpts. Default reads return 120
   lines with a SHA-256 of the full file; request another line range when necessary.
4. Prefer `edit` for small changes. Supply `path`, `old_text`, `new_text`, and the
   `expected_sha256` returned by `read`. The old text must match exactly once. `write`
   also checks the expected SHA before replacing an existing file. Read again after a
   stale-edit refusal; never bypass the check by changing a shared checkout.
5. Use `run` with an argument array, for example
   `{"action":"run","session":"<returned session>","argv":["git","status","--short"]}`.
   Commands run in that checkout with the managed environment, without shell interpolation.
   Do not pass passwords, tokens, or other credentials as command arguments.
6. Use `verify` with exact package names from preparation, for example
   `{"action":"verify","session":"<returned session>","crates":["simplypdf-measure"],"wasm":true,"timeout_seconds":1800}`.
   It runs formatting, scoped strict Clippy, scoped tests, layering and asset checks, then
   the requested scoped WASM check. Select all relevant packages; it does not claim that
   unrelated crates, platforms, or UI workflows were verified.
7. `run` and `verify` return a job ID immediately. Poll
   `{"action":"job","job":"<returned job>"}` until it finishes. Only `status: "done"`
   **and** `ok: true` mean the commands passed. A successful n8n execution, queued/running
   status, or a job ID alone is not test evidence. Failures include an actionable error
   and the log tail. Fix the first real error and submit a fresh verification.

The gateway serializes its jobs on this two-CPU VM, uses one Cargo build job and one test
thread, disables incremental build output, and stores temporary files on the disk-backed
coding volume rather than the small `/tmp` filesystem. Reads remain available while a
job runs; edits are refused until that session's job finishes. Timeouts stop only the
owned command process group. No commit, push, merge, or agent run happens implicitly.
Verification also refuses to pass if the source fingerprint changes while checks run.

## Coverage and maintenance

Repository rules still apply: add meaningful regression coverage, inspect page renders
for document changes, verify the real interface for UI changes, and record task-specific
acceptance evidence. The helper reduces environment and orchestration mistakes; it does
not guarantee correct generated code or replace code review and platform testing.

The [Sandbox Coding Doctor](https://n8n.pchrisoc.com/workflow/VSAK0s8Yu3wtTBri) is a
separate read-only readiness check. The helper and its tests are installed on the VM
under `~/.local/share/sandbox-coding/`. `env.sh` there supplies the same PATH, temporary
directory, and resource limits for manual SSH commands. Existing SSH workflows and the
experimental local model scheduler are separate; they do not automatically inherit
gateway isolation or verification.

Validated on 2026-11-06: ten helper regressions on macOS and on the Debian VM; n8n
request/response tests including a simulated SSH failure; real VM readiness and task
preparation from merged main. Gateway job `job-d6363c881dda` finished with `status:
"done"` and `ok: true`: formatting, strict all-target Clippy, 116 filter/property/doc
tests, layering, assets, and scoped WASM checking passed on source `87851d8c`. The
source fingerprint was unchanged throughout verification.
