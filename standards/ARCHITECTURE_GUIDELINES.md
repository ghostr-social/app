# Architecture Guidelines

Read [AGENTS.md](../AGENTS.md) first. This file narrows the root contract for architecture and boundaries.

This repository uses clean architecture for Flutter features, shared app services, and platform integrations.

## Dependency Rule

Dependencies always point inward:

- `domain` depends on nothing from Flutter, plugins, network, storage, or platform frameworks
- `application` behavior may live inside `domain/use_cases` or dedicated orchestration modules, but it still depends only on domain contracts and inward primitives
- `data` depends on domain contracts and owns translation to external systems
- `presentation` depends on domain or application contracts and may use Flutter, Bloc, routing, and UI concerns
- `bridges`, `routing`, `navigation`, and `di` coordinate features and app startup, but never become inward dependencies of domain logic
- `platform` wraps MethodChannels, native services, plugin glue, and OS-specific details that do not leak inward

Frameworks, plugins, native APIs, storage clients, and HTTP clients belong at the edge.

## Feature Shape

Feature code is organized around these responsibilities:

- `domain`: entities, value objects, policies, state machines, repository contracts, use cases, domain errors
- `data`: repository implementations, sources, DTO mappers, plugin adapters, storage adapters, network adapters
- `presentation`: blocs, cubits, screens, widgets, selectors, and presentation-only logic

Rules:

- Domain code must be pure and deterministic.
- Presentation code renders state and emits intent. It does not own business decisions.
- Data code translates plugin, storage, network, and native payloads into domain-safe types.
- Blocs and cubits orchestrate use cases. They should not become direct owners of plugin, storage, or HTTP details.

## Application Shape

Cross-cutting app code follows these responsibilities:

- `core`: cross-cutting primitives such as `Result`, `AppError`, logging, ports, and small platform-neutral helpers
- `shared/widgets` and `ui`: reusable UI primitives and shared visual building blocks
- `routing` and `navigation`: route names, route factories, route arguments, and app navigation contracts
- `bridges`: coordination between features or subsystems after bootstrap, such as wake-word and assistant orchestration
- `platform`: native channel wrappers and plugin-facing platform APIs
- `di`: composition root and dependency registration

## Modeling Rules

- Model identifiers as dedicated types.
- Model constrained strings, numeric ranges, confidence thresholds, and status values as strong types.
- Encode invalid states out of existence whenever possible.
- Prefer explicit objects over primitive bundles.
- Prefer enums or sealed states over boolean combinations when more than two states matter.
- Validate invariants at construction time or at the boundary where raw data enters the app.

## Boundary Rules

- Parsing, decoding, and validation happen at the edge.
- Domain objects should never receive unchecked JSON, plugin payloads, route arguments, or storage maps.
- Time, randomness, IO, and platform details must be abstracted behind ports, repositories, or adapters.
- Mapping code stays in `data`, `platform`, or other adapter layers. Do not leak DTOs or plugin models inward.
- `BuildContext`, `MethodChannel`, `AssetBundle`, `SharedPreferences`, HTTP responses, and plugin-specific classes do not belong in domain code.

## File And Module Rules

- One file, one cohesive responsibility.
- Production files must stay at or below `200` lines.
- Tests stay outside production files and at or below `100` lines per file.
- Public modules should expose narrow, intention-revealing APIs.
- Avoid generic helpers and barrel files that hide ownership and dependency direction.

## What Must Never Happen

- business rules in widgets
- business rules in route builders or bootstrap glue
- plugin or transport models leaking into domain code
- unchecked maps or DTOs reused as domain entities
- cross-layer imports that point outward
- shared utility folders that become dumping grounds
