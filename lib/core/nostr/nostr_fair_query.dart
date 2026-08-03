import 'dart:async';

import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/nostr/nostr_event_client.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';

const maxNostrFiltersPerRequest = 20;
const maxNostrBatchesPerFamily = 4;
const maxNostrTargetsPerFamily =
    maxNostrFiltersPerRequest * maxNostrBatchesPerFamily;
const nostrHydrationDeadline = Duration(seconds: 10);

typedef NostrTargetQueryBuilder<T> = NostrEventQuery Function(T target);
typedef NostrElapsedClock = Duration Function();

final Stopwatch _nostrStopwatch = Stopwatch()..start();

Duration _systemElapsedClock() => _nostrStopwatch.elapsed;

Future<List<NostrEventRecord>> loadFairNostrEvents<T>(
  NostrEventClient client,
  List<T> targets,
  NostrTargetQueryBuilder<T> buildQuery, {
  Duration timeout = nostrHydrationDeadline,
  NostrQueryBudget? budget,
}) async {
  final limited =
      targets.take(maxNostrTargetsPerFamily).toList(growable: false);
  final batches = _queryBatches(limited, buildQuery);
  final deadline = budget ?? NostrQueryBudget(timeout);
  deadline.requireActive();
  try {
    return await _loadBatches(client, batches, deadline)
        .timeout(deadline.remaining);
  } on TimeoutException {
    throw const AppFailure('Nostr interaction hydration timed out.');
  }
}

Future<List<NostrEventRecord>> _loadBatches(
  NostrEventClient client,
  List<List<NostrEventQuery>> batches,
  NostrQueryBudget deadline,
) async {
  final results = <NostrEventRecord>[];
  for (final batch in batches) {
    deadline.requireActive();
    results.addAll(await _loadWithOneSplit(client, batch, deadline));
  }
  return _uniqueEvents(results);
}

List<List<NostrEventQuery>> _queryBatches<T>(
  List<T> targets,
  NostrTargetQueryBuilder<T> buildQuery,
) {
  return <List<NostrEventQuery>>[
    for (var offset = 0; offset < targets.length; offset += 20)
      targets
          .skip(offset)
          .take(maxNostrFiltersPerRequest)
          .map(buildQuery)
          .toList(growable: false),
  ];
}

Future<List<NostrEventRecord>> _loadWithOneSplit(
  NostrEventClient client,
  List<NostrEventQuery> queries,
  NostrQueryBudget deadline,
) async {
  try {
    final events = await client.queryBatch(queries);
    deadline.requireActive();
    return events;
  } on AppFailure {
    if (queries.length <= 10) rethrow;
    deadline.requireActive();
    final first = await client.queryBatch(queries.take(10).toList());
    deadline.requireActive();
    final second = await client.queryBatch(queries.skip(10).toList());
    return <NostrEventRecord>[...first, ...second];
  }
}

class NostrQueryBudget {
  NostrQueryBudget([Duration duration = nostrHydrationDeadline])
      : this._(duration, _systemElapsedClock);

  NostrQueryBudget.withClock(Duration duration, NostrElapsedClock elapsedClock)
      : this._(duration, elapsedClock);

  NostrQueryBudget._(this._duration, this._elapsedClock)
      : _startedAt = _elapsedClock();

  final Duration _duration;
  final NostrElapsedClock _elapsedClock;
  final Duration _startedAt;

  Duration get remaining {
    final duration = _duration - (_elapsedClock() - _startedAt);
    return duration.isNegative ? Duration.zero : duration;
  }

  void requireActive() {
    if (remaining == Duration.zero) {
      throw const AppFailure('Nostr interaction hydration timed out.');
    }
  }
}

List<NostrEventRecord> _uniqueEvents(Iterable<NostrEventRecord> source) {
  final events = <String, NostrEventRecord>{};
  for (final event in source) {
    events[event.id] = event;
  }
  return events.values.toList(growable: false);
}
