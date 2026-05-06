---
name: svelte-rust-clean-code
description: Use when reviewing, refactoring, designing, or implementing Svelte, SvelteKit, Rust, or mixed Svelte/Rust systems with a focus on clean code, maintainability, correctness, full-stack architecture, and rigorous validation.
---

# Svelte Rust Clean Code

## Goal

Help Codex produce and review Svelte and Rust code that is easy to read, hard to misuse, simple to change, and strong enough for difficult production systems. Prefer direct, local improvements over broad rewrites unless the user explicitly asks for a larger refactor.

For large or ambiguous work, break the problem into smaller systems, define contracts between them, implement incrementally, and verify each layer before moving on. The operating principle is: excellent engineering makes hard systems understandable without making simple things unnecessarily complex.

## Trigger

Use this skill when the task mentions:

- Clean code, maintainability, readability, refactoring, code review, or best practices.
- Svelte, SvelteKit, Svelte stores, components, actions, forms, routes, or frontend state.
- Rust, Cargo, traits, errors, ownership, lifetimes, async Rust, CLI, services, or Tauri backends.
- Mixed Svelte/Rust apps, including SvelteKit frontends with Rust services or Tauri apps.
- Large full-stack features, architecture, system design, notifications, WebSocket flows, background jobs, integration work, or production hardening.

## General Clean Code Rules

- Read the surrounding code first and follow existing project conventions unless they are clearly harmful.
- Keep changes small, behavior-preserving, and covered by targeted validation.
- Prefer names that describe domain intent over implementation mechanics.
- Avoid clever control flow. Make the common path obvious and handle edge cases explicitly.
- Remove duplication only when the repeated logic has the same reason to change.
- Keep public interfaces narrow. Do not expose data or helpers just because they are convenient.
- Avoid adding abstractions before there is a real second use case or a clear complexity reduction.
- Preserve user-facing behavior unless the task explicitly asks for a behavior change.
- Prefer the simplest design that satisfies the real requirements, supports testing, and gives future maintainers clear extension points.
- Treat complexity as a cost. Add it only when it buys correctness, performance, reliability, security, or meaningful developer ergonomics.

## Large-System Engineering

When the user asks for a difficult system, do not jump directly into scattered code edits. First create a compact implementation map:

- Define the user-visible workflow and the system guarantees.
- Split the feature into bounded subsystems with clear responsibilities.
- Identify data models, APIs, events, background jobs, permissions, observability, and failure modes.
- Decide what must be synchronous, asynchronous, real-time, persisted, retried, rate-limited, or idempotent.
- Implement the smallest complete vertical slice first, then expand subsystem by subsystem.
- Keep contracts explicit between frontend, backend, storage, and real-time channels.
- Prefer boring, proven architecture over novelty unless the repository already uses a stronger pattern.

For full-stack work, consider these subsystems when relevant:

- Domain model and persistence.
- API contracts, validation, authentication, authorization, and rate limits.
- Real-time transport such as WebSocket, Server-Sent Events, polling, or push notifications.
- Background workers, queues, retries, idempotency keys, and dead-letter handling.
- Frontend state, optimistic updates, loading states, error states, and offline or reconnect behavior.
- Audit logs, metrics, tracing, structured logs, and operational visibility.
- Migration strategy, backwards compatibility, and rollout or rollback plan.
- Tests at unit, integration, contract, component, and end-to-end levels.

## Notification Systems

For notification features, design the complete lifecycle instead of only the UI:

- Event source: what domain event creates the notification and whether it is transactional.
- Recipient resolution: who should receive it, including roles, permissions, mute settings, and organization boundaries.
- Persistence: notification records, read/unread state, deduplication, expiration, and audit needs.
- Delivery channels: in-app feed, WebSocket, Server-Sent Events, email, push, desktop, webhook, or polling fallback.
- Real-time behavior: connection auth, reconnect, heartbeats, missed-event recovery, ordering, and duplicate suppression.
- UX behavior: unread badges, grouping, filters, mark-as-read, bulk actions, toasts, deep links, and accessible announcements.
- Reliability: retries, idempotency, backpressure, rate limiting, dead letters, and safe failure when a channel is down.
- Privacy and security: avoid leaking event contents to unauthorized clients or browser-visible code.
- Testing: event creation, permission filtering, delivery fanout, reconnect recovery, dedupe, read state, and UI rendering.

Choose WebSocket when the app needs bidirectional low-latency updates. Choose Server-Sent Events when one-way server updates are enough. Use polling as a simple fallback when real-time guarantees are not required or infrastructure is constrained.

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

## Delivery Discipline

For implementation tasks, keep working until the requested behavior is actually handled or a real blocker is reached.

- Do not stop at a plan when code changes are feasible.
- After each implementation slice, run the most relevant fast checks before continuing.
- Repeat fix-test cycles until the touched behavior passes or the remaining failure is clearly unrelated.
- Test both success paths and important failure paths.
- Verify frontend behavior in the browser when a UI change needs visual or interaction confidence.
- Verify backend behavior with targeted unit, integration, or command-level tests when available.
- If a test cannot be run, state exactly why and what risk remains.
- Keep the final summary focused on what changed, what was validated, and what still needs human input.

## Output Style

- Be concrete and cite files and lines when reviewing.
- For implementation tasks, make the change directly and summarize what changed.
- For review tasks, lead with risks and fixes, ordered by severity.
- Avoid generic clean code advice that is not grounded in the code being discussed.
