---
name: commit
description: Review current git changes, validate them, and create a clean commit
disable-model-invocation: true
---

Create a git commit for the current working tree.

User context:

- Optional instruction from user: $ARGUMENTS

Follow this workflow exactly:

1. Inspect the current repository state.
   - Run: `git status --short`
   - Run: `git diff --stat`
   - Run: `git diff --cached --stat`
   - Review the actual diffs for both staged and unstaged changes.

2. Understand the change before committing.
   - Summarize what changed in 3-7 bullets.
   - Identify risky or incomplete edits.
   - If there are unrelated changes mixed together, STOP and explain what should be split.

3. Validate quality before committing.
   - Run the smallest relevant verification for the project:
     - targeted tests if obvious
     - lint/typecheck if relevant
     - otherwise explain why no automated verification was run
   - If verification fails, STOP and summarize failures.

4. Prepare the commit contents.
   - If nothing is staged, stage only the files that belong to the change.
   - Do not stage unrelated files.
   - Prefer the smallest coherent commit.

5. Write the commit message.
   - Use Conventional Commit style when it fits:
     - feat:
     - fix:
     - refactor:
     - docs:
     - test:
     - chore:
   - Format:
     - first line: concise subject under 72 chars
     - optional body: why this change was needed and any important tradeoffs
   - If the user gave commit guidance in $ARGUMENTS, incorporate it.

6. Before committing, show:
   - files to be committed
   - final commit message
   - verification run and result

7. Create the commit.

8. After committing, report:
   - commit hash
   - commit title
   - whether any changes remain unstaged or uncommitted

Rules:

- Never commit broken or unrelated work.
- Never use `git add .` unless every modified file clearly belongs to this change.
- Prefer caution over speed.
