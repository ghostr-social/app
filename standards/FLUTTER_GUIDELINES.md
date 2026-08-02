# Flutter Guidelines

Read [AGENTS.md](../AGENTS.md) first. This file narrows the root contract for Flutter UI and widget composition.

## Widget Design

- Widgets are primarily presentational and compositional.
- Business rules belong in domain code, use cases, policies, blocs, or cubits, not in widget build methods.
- Screens wire intents, routing, and composition. Leaf widgets stay narrow and reusable.
- Prefer composition over deeply nested conditional trees.
- Keep widget inputs explicit and intention-revealing.
- Prefer shared widgets and themed Material primitives before introducing feature-local visual patterns.

## State Rules

- Keep state local only when it is truly local and short-lived.
- Do not store derivable state.
- Use bloc or cubit state for workflow state, async state, and cross-widget coordination.
- Represent loading, empty, success, error, disabled, and permission states directly in types.
- Keep plugin and repository calls out of `build`.

## UI And Theming Rules

- Route visual decisions through `ThemeData`, `ColorScheme`, `TextTheme`, shared constants, or `ThemeExtension`s.
- Avoid hard-coded colors, spacing, radii, and text styles in feature widgets when a shared token can express the intent.
- Preserve semantics, focus behavior, and tap targets.
- Use `Semantics`, labels, tooltips, and visible text that reflect the real UI contract.
- Keep navigation contracts inside `routing` or `navigation`, not scattered through feature widgets.

## Code Shape

- Production files stay at or below `200` lines.
- Test files stay at or below `100` lines.
- Build methods and widget callbacks stay at or below `20` logical lines where practical.
- Large screens must be decomposed into small widgets before they become logic containers.

## Testing

- Use widget tests for screens, widgets, route wiring, and visible UI contracts.
- Assert on semantics, visible text, enabled state, navigation, and rendered behavior rather than implementation details.
- Golden tests are optional support, never the only acceptance criterion.
- If a widget has loading, empty, error, disabled, or permission states, test each state.
- Prefer minimal test harnesses over booting the entire app when the behavior can be isolated.

## What To Avoid

- data fetching or plugin calls in presentation-only leaf widgets
- business logic in `build` methods
- giant screen widgets that own too many responsibilities
- ad hoc style forks that bypass the shared theme
- brittle tests that depend on incidental widget tree structure
