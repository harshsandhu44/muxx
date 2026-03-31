---
name: pr
description: Create a pull request from the current branch with a strong title and description
disable-model-invocation: true
allowed-tools: Bash(git *), Bash(gh *)
---

Create a pull request for the current branch.

User context:

- Optional instruction from user: $ARGUMENTS

Follow this workflow exactly:

1. Inspect branch and diff.
   - Run: `git status --short`
   - Run: `git branch --show-current`
   - Run: `git remote -v`
   - Determine the default base branch from git/gh if possible.
   - Run: `git diff --stat origin/<base>...HEAD`
   - Review the actual diff against the base branch.

2. Check commit state.
   - If there are uncommitted changes, STOP and tell the user to run `/commit` first.
   - If the branch is not pushed, push it first.

3. Understand the PR.
   - Summarize:
     - what changed
     - why it changed
     - risks / migration notes
     - test coverage / manual verification

4. Draft PR metadata.
   - Title:
     - concise
     - outcome-focused
     - no vague junk like "updates" or "fixes stuff"
   - Body sections:
     - Summary
     - What changed
     - How to test
     - Risks / Notes
   - If $ARGUMENTS contains extra context, incorporate it.

5. Show the proposed PR title and body before creating it.

6. Create the PR with GitHub CLI.
   - Prefer: `gh pr create --fill` only if the autofill is good enough
   - Otherwise provide explicit `--title` and `--body`
   - Target the correct base branch
   - Assign the PR to the user who created it
   - Add proper label for the type of change (e.g. bug, feature, docs)

7. After creation, report:
   - PR URL
   - PR title
   - base branch
   - head branch

Rules:

- Do not create a PR with uncommitted local changes.
- Do not guess the base branch blindly if it can be detected.
- Prefer explicit, reviewer-friendly descriptions over short lazy ones.
