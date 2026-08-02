# Design System Guidelines

Read [AGENTS.md](../AGENTS.md) first. This file narrows the root contract for the shared UI language and component inventory.

This repository must converge toward one unified design system.

The goal is not "nice-looking screens". The goal is a coherent, reusable, and testable UI language with one standardized inventory of shared widgets and themed Material primitives.

## Core Rules

- Prefer extending the shared design system over inventing feature-local UI.
- Reuse an existing shared widget or themed Material primitive before creating a new one.
- If a new primitive is necessary, add it intentionally to `lib/shared/widgets` or `lib/ui`.
- Do not create near-duplicate widgets that differ only by minor styling or naming.
- Shared primitives must preserve accessibility, testability, and visual consistency.

## Standardized Component Inventory

The app should standardize around a shared inventory like this:

- app shell: `Scaffold`, `AppBar`, `Drawer`, `SafeArea`
- surfaces: themed `Card`, `ListTile`, and shared containers such as `GlassContainer`
- feedback: `SnackBar`, loading panels, empty panels, error panels, and status widgets
- actions and inputs: themed Material buttons, `IconButton`, `PopupMenuButton`, `TextField`, `SwitchListTile`, `Slider`
- audio-aware shared UI: `AudioReactiveBackground`, `AudioRouteIndicator`, and any promoted shared audio chrome
- navigation: `AppRouter`, `RouteBuilder`, `RouteRegistry`, and consistent route affordances

This list is the default shared vocabulary. Expand it carefully and only when an existing primitive cannot express the need cleanly.

## Component Admission Rule

Before adding a new shared component, verify:

- no current inventory component can represent the behavior cleanly
- the component solves a repeated UI problem, not a one-off screen quirk
- the API is strongly typed and intention-revealing
- semantics and accessibility are explicit
- widget tests cover the component contract
- affected flows remain protected by widget or integration tests

## Styling Rules

- Shared components must use the same tokens for color, spacing, typography, shape, border, and motion.
- Route visual decisions through `ThemeData`, `ColorScheme`, `TextTheme`, shared constants, or `ThemeExtension`s.
- Variants belong in the shared widget API, not in feature-local style forks.
- Feature code may compose shared widgets, but must not silently fork their visual language.
- If styling pressure keeps escaping the shared system, improve the system instead of layering hacks.

## Testing Rules

- Every shared component must have behavior tests.
- Stateful shared components must cover loading, empty, error, disabled, permission, and interactive states when applicable.
- Accessibility and semantics are part of the component contract.
- Snapshot-only and golden-only tests are forbidden.

## What To Avoid

- feature-specific clones of shared widgets
- one-off button, dialog, panel, or settings row implementations that should be shared
- inconsistent spacing and typography between screens
- bypassing theme tokens with scattered hard-coded values
- variant explosion caused by unclear component boundaries
