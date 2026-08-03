import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/features/activity/data/nostr_activity_repository.dart';

import '../support/fake_activity_repository.dart';
import '../support/fake_nostr_event_client.dart';
import '../support/nostr_test_values.dart';
import '../support/recording_failure_reporter.dart';
import '../support/sample_data.dart';

void main() {
  test('merges incoming Nostr likes comments and follows with local history',
      () async {
    final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey);
    client.events.addAll([
      _event(_identity(testEventId, testAuthorPublicKey, 3), 10, const [
        ['p', testViewerPublicKey],
      ]),
      _event(_identity(secondTestEventId, testFanPublicKey, 7), 20, const [
        ['p', testViewerPublicKey],
      ]),
      _event(
          _identity(publishedTestEventId, testCreatorPublicKey, 1111),
          30,
          const [
            ['P', testViewerPublicKey],
          ],
          content: 'Great clip!'),
      _event(_identity(publishedEventId(4), testViewerPublicKey, 7), 40, const [
        ['p', testViewerPublicKey],
      ]),
    ]);
    final repository = NostrActivityRepository(
      client: client,
      local: FakeActivityRepository(items: [sampleActivity()]),
      failureReporter: RecordingFailureReporter(),
    );

    final items = await repository.load();

    expect(items.map((item) => item.id), [
      'activity-1',
      publishedTestEventId,
      secondTestEventId,
      testEventId,
    ]);
    expect(
        items.map((item) => item.title),
        containsAll([
          'New comment',
          'New like',
          'New follower',
        ]));
  });
}

NostrEventRecord _event(
  NostrEventIdentity identity,
  int createdAt,
  List<List<String>> tags, {
  String content = '',
}) {
  return NostrEventRecord(
    identity: identity,
    tags: tags,
    content: content,
    createdAt: createdAt,
  );
}

NostrEventIdentity _identity(String id, String author, int kind) {
  return NostrEventIdentity.parse(
    id: id,
    authorPublicKeyHex: author,
    kind: kind,
  );
}
