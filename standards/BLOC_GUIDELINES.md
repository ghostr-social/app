# BLoC Guidelines

Read [AGENTS.md](../AGENTS.md) first. This file narrows the root contract for event-driven state management.

## State Management Choice

- Use `Bloc` for behavior-rich flows, cross-widget coordination, or explicit event-driven workflows.
- Use `Cubit` for narrow UI state that does not need a public event protocol.
- If state can stay as simple local widget state without leaking behavior or duplication, keep it local.

## Design Rules

- Events represent user or system intent, not widget implementation details.
- States model real business and UI states explicitly: loading, idle, empty, success, error, disabled, permission-restricted, and interrupted when applicable.
- Keep every emitted state valid on its own. Do not rely on implicit combinations of booleans.
- Delegate business rules to use cases, domain services, policies, or repositories. Event handlers should orchestrate, not decide the business.
- Inject dependencies through constructors. Widgets should not reach through a bloc into repositories or plugins.
- Keep bloc constructors predictable. If subscriptions or bootstrap work start immediately, they must be intentional and directly tested.

## Code Shape

- Production files stay at or below `200` lines.
- Test files stay at or below `100` lines.
- Event handlers stay at or below `20` logical lines where practical.
- One handler should own one branch of behavior at one level of abstraction.
- Avoid large monolithic blocs that coordinate unrelated concerns. Split by workflow when the state space stops being cohesive.

## UI Contract

- Widgets emit intent to blocs or cubits and render the exposed state.
- Use `BlocSelector` or focused `BlocBuilder` scopes to keep rebuilds narrow.
- Use `BlocListener` for one-off UI reactions such as snack bars, navigation, or dialogs.
- Do not call repositories, plugins, or HTTP clients directly from widgets.
- Do not let widgets reconstruct business rules by combining raw bloc fields ad hoc.

## Testing

- Use `bloc_test` for event-to-state contracts.
- Keep one behavior or one scenario per test file.
- Assert the exact state sequence when the order matters.
- Cover every event branch, retry branch, debounce path, and timer-driven branch even if coverage tooling only reports line coverage.
- Use `fake_async`, fake clocks, or deterministic schedulers for time-based behavior.
- Mock only true boundaries. Prefer real value objects and in-memory fakes for everything else.

## What To Avoid

- repository or plugin calls from widgets
- blocs that own unrelated features
- hidden side effects in state getters
- states that require reading private fields to understand validity
- boolean flag combinations that should be enums or sealed states
- untested subscriptions, retries, debounces, and cancellation paths
