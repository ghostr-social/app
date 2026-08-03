import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/core/nostr/nostr_fair_query.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_test_values.dart';

void main() {
  test('uses an injected monotonic elapsed clock for its budget', () {
    var elapsed = Duration.zero;
    final budget = NostrQueryBudget.withClock(
      const Duration(seconds: 1),
      () => elapsed,
    );

    elapsed = const Duration(milliseconds: 400);
    expect(budget.remaining, const Duration(milliseconds: 600));
    elapsed = const Duration(seconds: 1);
    expect(budget.requireActive, throwsA(isA<AppFailure>()));
  });

  test('fails a stalled hydration family at its deadline', () async {
    final client = _DelayedBatchClient();
    final pending = loadFairNostrEvents(
      client,
      <int>[1],
      _query,
      timeout: const Duration(milliseconds: 100),
    );
    await client.started.future;

    await expectLater(pending, throwsA(isA<AppFailure>()));
    client.release.complete();
  });

  test('does not issue another batch after the deadline', () async {
    final client = _DelayedBatchClient();
    final pending = loadFairNostrEvents(
      client,
      List<int>.generate(40, (index) => index),
      _query,
      timeout: const Duration(milliseconds: 100),
    );
    await client.started.future;
    await expectLater(pending, throwsA(isA<AppFailure>()));

    client.release.complete();
    await Future<void>.delayed(Duration.zero);

    expect(client.calls, 1);
  });
}

NostrEventQuery _query(int index) => NostrEventQuery(kinds: const <int>[7]);

class _DelayedBatchClient extends FakeNostrEventClient {
  _DelayedBatchClient() : super(publicKeyHex: testViewerPublicKey);

  final release = Completer<void>();
  final started = Completer<void>();
  var calls = 0;

  @override
  Future<List<NostrEventRecord>> queryBatch(
    List<NostrEventQuery> batch,
  ) async {
    calls += 1;
    if (!started.isCompleted) started.complete();
    await release.future;
    return const <NostrEventRecord>[];
  }
}
