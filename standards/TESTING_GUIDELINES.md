# Testing Guidelines

Read [AGENTS.md](../AGENTS.md) first. This file narrows the root contract for the testing process and quality gates.

Tests are the executable specification of this repository.

## Required Development Loop

Always work Red -> Green -> Refactor.

For any behavior change:

1. Add or update the failing highest-boundary automated test for the user-visible contract.
2. Add or update failing unit, bloc, or adapter tests for the smallest business rules involved.
3. Add or update integration coverage when the flow crosses routes, features, or app bootstrap boundaries.
4. Implement the smallest production change that makes the tests pass.
5. Refactor while the full suite remains green.
6. Run the relevant verification commands.

## Test Taxonomy

Use the smallest useful test first, but never skip the user contract:

- `flutter test` unit tests: pure Dart logic, value objects, parsers, mappers, policies, use cases, deterministic engine logic
- `bloc_test`: blocs and cubits, event handling, state transitions, retries, debounces, and orchestration seams
- widget tests: screens, widgets, route wiring, semantics, visible UI contracts, loading and error states
- integration tests: cross-route journeys, startup flows, multi-feature orchestration, and flows that need a fuller app harness
- manual device verification: permissions, Bluetooth or audio routing, background execution, wake-word capture, and other OS-driven behavior that cannot be modeled faithfully in automated tests

## Highest-Boundary Contract

- Cover user-visible behavior at the highest practical automated boundary.
- When a widget test can represent the contract faithfully, prefer it.
- When a route-spanning or multi-feature flow needs a fuller harness, add an integration test.
- When real hardware or OS behavior is essential, add the best automated coverage available and record the manual verification steps and outcome.

## Widget And Integration Rules

- Read like user or business scenarios.
- Use semantics, visible text, enabled state, and route outcomes as assertions whenever possible.
- Assert the user can complete the behavior, not just that widgets exist.
- Cover success, validation, error, loading, and permission paths when relevant.
- Avoid brittle tree-shape assertions unless there is no contract-level alternative.

## Unit And Bloc Rules

- One test file covers one behavior or one scenario.
- Prefer a single top-level scenario per file.
- Test files stay at or below `100` lines.
- Tests live outside production files.
- Shared builders, factories, and helpers belong in dedicated test support modules.
- Mock only true boundaries. Prefer real value objects and in-memory fakes for everything else.
- Do not assert private implementation details when a public contract can be asserted.
- Snapshot-only and golden-only tests are forbidden.

## Coverage Policy

The release pipeline has one blocking Dart coverage floor:

- every measured Dart source file must have at least `80%` line coverage
- every executable Dart source must be represented; an omitted source is treated as uncovered
- coverage targets above `80%` and coverage-regression reports are advisory and must not block the pipeline

Important domain and state-transition behavior should still be tested thoroughly.
Use focused tests to protect behavior instead of raising the blocking threshold.

## Flutter-Specific Rules

- Build the smallest possible harness around the behavior under test.
- Prefer deterministic fake time, fake repositories, and fake platform adapters over sleeping or waiting on real timers.
- Keep plugin and MethodChannel mocking at the adapter boundary whenever possible.
- Use semantics and user-observable outcomes for verification, not incidental widget tree structure.

## Stable Commands

- `flutter analyze`
- `flutter test`
- `make test-coverage`
- `make coverage-summary`
- `make wake-word-unit-tests`
- `make wake-word-tests`
