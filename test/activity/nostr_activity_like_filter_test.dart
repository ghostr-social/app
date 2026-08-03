import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/features/activity/data/nostr_activity_repository.dart';

import '../support/fake_activity_repository.dart';
import '../support/fake_nostr_event_client.dart';
import '../support/nostr_test_values.dart';
import '../support/recording_failure_reporter.dart';

void main() {
  test('exposes only empty and plus reactions as like activity', () async {
    final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey)
      ..events.addAll([
        _reaction(testEventId, '', 1),
        _reaction(secondTestEventId, '+', 2),
        _reaction(publishedTestEventId, '-', 3),
        _reaction(publishedEventId(4), '🔥', 4),
      ]);
    final repository = NostrActivityRepository(
      client: client,
      local: FakeActivityRepository(),
      failureReporter: RecordingFailureReporter(),
    );

    final items = await repository.load();

    expect(items.map((item) => item.id).toSet(), {
      testEventId,
      secondTestEventId,
    });
  });
}

NostrEventRecord _reaction(String id, String content, int createdAt) {
  return NostrEventRecord(
    identity: NostrEventIdentity.parse(
      id: id,
      authorPublicKeyHex: testAuthorPublicKey,
      kind: 7,
    ),
    tags: const [
      ['p', testViewerPublicKey],
    ],
    content: content,
    createdAt: createdAt,
  );
}
