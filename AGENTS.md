# AGENTS

This document defines the non-negotiable development rules for humans and agents working in this repository.
Read this file first.

If multiple docs apply, follow them in this order:

1. [AGENTS.md](./AGENTS.md)
2. [TESTING_GUIDELINES.md](./standards/TESTING_GUIDELINES.md)
3. [ARCHITECTURE_GUIDELINES.md](./standards/ARCHITECTURE_GUIDELINES.md)
4. [DESIGN_SYSTEM_GUIDELINES.md](./standards/DESIGN_SYSTEM_GUIDELINES.md)
5. The relevant tech guide: [DART_GUIDELINES.md](./standards/DART_GUIDELINES.md), [FLUTTER_GUIDELINES.md](./standards/FLUTTER_GUIDELINES.md), [BLOC_GUIDELINES.md](./standards/BLOC_GUIDELINES.md)

## Mission

Build a Flutter application with the highest practical standards for correctness, maintainability, platform reliability, and design clarity.

The repository standard is simple:

- Tests are the specification.
- When in doubt, add another test.
- Strong Dart models are a safety layer, not a convenience.
- Clean architecture and explicit state transitions are mandatory.
- Small files, small functions, and single-purpose modules are mandatory.
- Platform and plugin integrations stay behind ports, repositories, or adapters.
- The UI must converge toward one unified Material-based design system and one standardized component inventory.

## Non-Negotiable Workflow

Every change follows this order:

1. Define the behavior to change.
2. Add a failing automated test at the highest practical boundary for that behavior.
3. Add failing unit, bloc, widget, or integration tests for domain, application, adapter, and UI behavior as needed.
4. Implement the minimum production code required to make the tests pass.
5. Refactor only after all tests are green.
6. Run the relevant verification commands.

If you touch production code before writing a failing test, stop and correct the process.

## Testing Contract

- Every bug fix starts with a failing regression test.
- Every new business rule must have unit or bloc coverage at the right boundary.
- Every user-visible flow must be protected by widget or integration coverage at the highest practical automated boundary.
- If a behavior depends on real device capabilities that cannot be represented faithfully in automated tests, add the best available automated coverage and record the manual verification performed.
- Tests live outside production files.
- One test file covers one behavior or one scenario.
- Shared helpers live in dedicated helper modules, never inside unrelated tests.
- Snapshot-only and golden-only tests are forbidden.
- Accessibility, semantics, and visible-contract assertions are mandatory for interactive UI.

### Required UI States

Whenever applicable, tests must cover:

- happy path
- loading
- empty
- error
- disabled or permission-restricted states

## Quality Gates

These limits apply unless a stricter local rule exists:

- `100%` line coverage for touched pure domain logic, parsers, policies, state machines, and deterministic engine logic
- `95%` line coverage for touched blocs, cubits, adapters, bridges, routing helpers, and shared widgets
- `0` coverage regressions allowed in touched modules
- production files must stay at or below `200` lines
- test files must stay at or below `100` lines
- functions and methods must stay at or below `20` logical lines
- functions and methods must accept at most `4` parameters before introducing a typed object
- cyclomatic complexity must stay at or below `5`
- cognitive complexity must stay at or below `10`
- warnings are treated as errors

If a target cannot meet these gates cleanly, refactor the design. Exceptions require explicit written justification in the task or PR.

## Command Contract

Prefer `make` targets when they exist. Otherwise use the direct Flutter or Dart command.

The minimum verification surface for this repository is:

- `flutter analyze`
- `flutter test`
- `make test-coverage`
- `make coverage-summary`
- `make wake-word-unit-tests` when touching the wake-word engine
- `make wake-word-tests` when touching dataset-driven wake-word behavior

If a repeated workflow has no stable command, add one before the repository accumulates ad hoc variants.

## Architecture Contract

- Use clean architecture.
- Dependencies point inward.
- Feature code is organized around `domain`, `data`, and `presentation`.
- Domain code is framework-free, UI-free, and IO-free.
- Presentation code renders state and emits intent. It does not own business rules.
- Data code translates between plugins, storage, network, native APIs, and domain-safe models.
- `bridges`, `routing`, `navigation`, and `di` coordinate features and app bootstrap, but do not become dumping grounds for product logic.
- Platform and plugin details stay behind adapters, repositories, or ports.

## Design Contract

- Prefer strong types, value objects, enums, and sealed states over primitive-heavy APIs.
- No stringly typed identifiers when a domain type can exist.
- No unvalidated plugin payloads, JSON maps, or route arguments flowing inward.
- Keep modules cohesive and small.
- Each file should have one reason to change.
- Each function should do one thing at one level of abstraction.
- Avoid hidden state and implicit dependencies.
- Prefer constructor or parameter injection over global lookups. Use the registry only at composition roots when necessary.

## Design System Contract

- Build and preserve one unified design system across the app.
- Prefer existing shared widgets and themed Material primitives before creating new visual patterns.
- Shared UI primitives must come from the standardized component inventory defined in [DESIGN_SYSTEM_GUIDELINES.md](./standards/DESIGN_SYSTEM_GUIDELINES.md).
- If a new component primitive is truly needed, add it deliberately to the inventory instead of introducing an ad hoc feature-local variant.
- Visual tokens, spacing, typography, shape, states, and interaction patterns must stay consistent across screens.
- Duplicate widgets with slightly different names or styles are forbidden.

## Definition of Done

A task is done only when:

- the behavior is specified by tests first
- all relevant tests are green
- the relevant verification commands pass in the current branch state
- the change keeps the architecture boundaries intact
- manual verification is documented for any device-only behavior that could not be automated
- the final report lists the tests added or changed and the command results

## References

- [AI_BOOTSTRAP_PROMPT.md](./standards/AI_BOOTSTRAP_PROMPT.md)
- [ARCHITECTURE_GUIDELINES.md](./standards/ARCHITECTURE_GUIDELINES.md)
- [DESIGN_SYSTEM_GUIDELINES.md](./standards/DESIGN_SYSTEM_GUIDELINES.md)
- [TESTING_GUIDELINES.md](./standards/TESTING_GUIDELINES.md)
- [DART_GUIDELINES.md](./standards/DART_GUIDELINES.md)
- [FLUTTER_GUIDELINES.md](./standards/FLUTTER_GUIDELINES.md)
- [BLOC_GUIDELINES.md](./standards/BLOC_GUIDELINES.md)
