---
name: svelte-rust-clean-code
description: Use when reviewing, refactoring, or writing Svelte, SvelteKit, Rust, or mixed Svelte/Rust code with a focus on clean code, maintainability, correctness, and practical engineering hygiene.
---

# Svelte Rust Clean Code

## Goal

Help Codex produce and review Svelte and Rust code that is easy to read, hard to misuse, and simple to change. Prefer direct, local improvements over broad rewrites unless the user explicitly asks for a larger refactor.

## Trigger

Use this skill when the task mentions:

- Clean code, maintainability, readability, refactoring, code review, or best practices.
- Svelte, SvelteKit, Svelte stores, components, actions, forms, routes, or frontend state.
- Rust, Cargo, traits, errors, ownership, lifetimes, async Rust, CLI, services, or Tauri backends.
- Mixed Svelte/Rust apps, including SvelteKit frontends with Rust services or Tauri apps.

## General Clean Code Rules

- Read the surrounding code first and follow existing project conventions unless they are clearly harmful.
- Keep changes small, behavior-preserving, and covered by targeted validation.
- Prefer names that describe domain intent over implementation mechanics.
- Avoid clever control flow. Make the common path obvious and handle edge cases explicitly.
- Remove duplication only when the repeated logic has the same reason to change.
- Keep public interfaces narrow. Do not expose data or helpers just because they are convenient.
- Avoid adding abstractions before there is a real second use case or a clear complexity reduction.
- Preserve user-facing behavior unless the task explicitly asks for a behavior change.

## Svelte Guidance

- Keep components focused on one rendering responsibility.
- Move reusable logic into plain TypeScript modules, stores, actions, or derived state when it improves clarity.
- Prefer declarative markup over imperative DOM manipulation.
- Use stores for shared state, not for every local value. Keep local UI state local.
- Derive values instead of syncing duplicate state through reactive statements.
- Keep `$:` statements short and side-effect-light. Avoid hidden write chains across many reactive blocks.
- Prefer explicit props and events over reaching into module-level mutable state.
- Use semantic HTML first, then ARIA only where semantics are insufficient.
- Keep event handlers named by user intent, such as `submitTask` or `toggleFilter`.
- Avoid large inline expressions in markup. Extract them when they obscure the UI structure.
- In SvelteKit, keep route loading and server actions focused on data boundaries, validation, and authorization.
- Do not put secret or trusted logic in client-only code.

## Rust Guidance

- Model invalid states out of the type system when it keeps callers simpler.
- Prefer clear ownership over unnecessary cloning. Clone intentionally at boundaries.
- Use `Result` for recoverable failures and reserve `panic!` for invariant violations or tests.
- Preserve error context with typed errors or contextual messages.
- Keep functions small enough that ownership, error paths, and side effects are easy to audit.
- Prefer iterators when they improve clarity, but use simple loops when they make mutation or branching easier to read.
- Avoid `unwrap` and `expect` in production code unless the invariant is obvious and documented by context.
- Keep trait bounds close to the function or type that needs them.
- Use newtypes when primitive values carry domain meaning or validation.
- Keep async boundaries explicit. Avoid holding locks or borrows across `.await` unless deliberately designed.
- Keep modules organized around domain concepts, not only technical layers.

## Review Checklist

When reviewing code, report findings before summary. Check:

- Does each component, function, route, or module have one clear responsibility?
- Are names specific enough for future maintainers to understand intent?
- Is state duplicated or synchronized manually when it could be derived?
- Are errors handled at the right layer with enough context?
- Are edge cases visible and tested?
- Are user inputs validated before crossing trust boundaries?
- Are abstractions helping current code, or only preparing for imagined future needs?
- Are tests focused on behavior rather than implementation details?

## Refactoring Approach

1. Identify the smallest change that improves readability or correctness.
2. Preserve existing behavior and public APIs unless a cleaner API is the explicit goal.
3. Separate mechanical cleanup from semantic changes when possible.
4. Add or update tests when the refactor touches behavior, shared utilities, or error handling.
5. Run the repo's existing formatter, linter, and targeted tests when available.

## Output Style

- Be concrete and cite files and lines when reviewing.
- For implementation tasks, make the change directly and summarize what changed.
- For review tasks, lead with risks and fixes, ordered by severity.
- Avoid generic clean code advice that is not grounded in the code being discussed.
