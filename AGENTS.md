# Repository Instructions

Use super ultra thinking for engineering work. Solve issues completely, with enough context gathering, validation, and clear tradeoff handling for the actual risk of the change.

## Always-On Svelte/Rust Clean Code

For any task that touches Svelte, SvelteKit, TypeScript frontend code, Rust backend code, shared frontend/backend contracts, code review, refactoring, maintainability, correctness, or bug fixing, always use the local `svelte-rust-clean-code` skill even when the user does not mention it.

Before reviewing or editing related code:

1. Read `plugins/svelte-rust-clean-code/skills/svelte-rust-clean-code/SKILL.md`.
2. Apply its clean-code rules, Svelte guidance, Rust guidance, review checklist, and output style.
3. Follow existing project conventions unless they are clearly harmful.
4. Prefer the smallest behavior-preserving change that solves the issue.
5. Validate with targeted commands when the change touches behavior, types, tests, formatting, or build-sensitive code.

## Review Default

When the user asks for a review, audit from a correctness and maintainability stance:

- Report findings first, ordered by severity.
- Cite concrete file and line references.
- Explain why each issue matters and what fix is appropriate.
- Avoid generic advice that is not grounded in the code.
- Mention residual test or validation gaps.

## Implementation Default

When the user asks for a fix or feature:

- Read surrounding code before editing.
- Keep state local unless shared state is needed.
- Derive values instead of manually synchronizing duplicate state.
- Keep Svelte reactive statements short and side-effect-light.
- Keep Rust errors explicit and contextual.
- Avoid broad rewrites, speculative abstractions, and unrelated formatting churn.
- Add or update focused tests when behavior, shared helpers, or error handling changes.

## Validation Default

Use the narrowest useful validation first:

- For Svelte/TypeScript changes, prefer existing typecheck, lint, unit tests, or targeted test commands from `package.json`.
- For Rust changes, prefer `cargo test` or targeted package/module tests in the affected crate.
- If validation cannot run, state exactly why and what remains unverified.

## Security And Trust Boundaries

- Do not put secrets or trusted authorization logic in client-only code.
- Treat auth, cookies, localStorage, API response parsing, and server/client boundaries as high-risk areas.
- Prefer backend-owned secure cookies for auth tokens when possible.
- Validate user input before it crosses trust boundaries.

