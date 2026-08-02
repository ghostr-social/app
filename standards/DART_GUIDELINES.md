# Dart Guidelines

Read [AGENTS.md](../AGENTS.md) first. This file narrows the root contract for Dart modeling and implementation.

## Compiler And Analyzer Rules

- `flutter analyze` must pass with no warnings.
- Keep analyzer and linter configuration strict enough that warnings are treated as errors in practice.
- `dart format --output=none --set-exit-if-changed .` is the expected formatting check.

## Type Design

- Model domain concepts with explicit types, not loose maps or primitive bundles.
- Prefer enums, sealed hierarchies, and dedicated value objects for constrained states and identifiers.
- Parse unknown input at the boundary and convert it into trusted types immediately.
- Avoid `dynamic`. If a boundary shim truly requires it, isolate it and document why.
- Prefer named result objects over tuples or loosely typed maps when meaning matters.
- Use immutable data by default. Favor `final`, `const`, and value equality for stable state.

## Code Shape

- Production files stay at or below `200` lines.
- Test files stay at or below `100` lines.
- Functions stay at or below `20` logical lines.
- Functions accept at most `4` parameters before introducing a typed object.
- Cyclomatic complexity stays at or below `5`.
- Cognitive complexity stays at or below `10`.

## Design Rules

- Keep business rules outside widgets and plugin adapters.
- Prefer pure functions for domain logic, policies, parsers, and mappers.
- Keep side effects in repositories, sources, platform adapters, or orchestration layers.
- Avoid boolean arguments when an enum or options object would be clearer.
- Avoid passing raw `Map<String, dynamic>` values through the domain.
- Export narrow public APIs from each module.
- `BuildContext`, `MethodChannel`, `AssetBundle`, HTTP response models, and plugin types do not belong in domain code.

## Error Handling

- Use explicit result and error types for recoverable failures.
- Convert plugin, network, filesystem, and platform exceptions into app-safe errors at the boundary.
- Do not swallow exceptions silently. Log and translate them intentionally.
- Avoid null-driven control flow when the domain can model the state directly.

## Testing

- Test pure logic without mocks.
- Use in-memory fakes for repositories and ports where practical.
- Use `fake_async` or controllable clocks for time-sensitive behavior.
- Add regression coverage for every bug fix.

## What To Avoid

- `dynamic` in domain or application code
- unvalidated JSON or plugin payloads outside boundary layers
- giant utility files with no clear ownership
- public APIs that expose transport models as domain models
- `BuildContext` leaking into non-UI layers
