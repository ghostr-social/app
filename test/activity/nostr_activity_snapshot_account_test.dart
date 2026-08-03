import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/core/storage/account_storage_scope.dart';
import 'package:ghostr/features/activity/data/local_activity_repository.dart';
import 'package:ghostr/features/activity/data/nostr_activity_repository.dart';
import 'package:ghostr/features/activity/domain/activity_item.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_test_values.dart';
import '../support/recording_failure_reporter.dart';
import '../support/sample_data.dart';

void main() {
  test('activity snapshot pins local history and its remote viewer', () async {
    SharedPreferences.setMockInitialValues({});
    var account = NostrPublicKeyHex.parse(testViewerPublicKey);
    final local = LocalActivityRepository(
      await SharedPreferences.getInstance(),
      accountScope: AccountStorageScope(() => account),
    );
    final localA = sampleActivity();
    await local.record(localA);
    final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey)
      ..events.addAll([
        _reaction(testEventId, testViewerPublicKey),
        _reaction(secondTestEventId, testAuthorPublicKey),
      ]);
    final repository = NostrActivityRepository(
      client: client,
      local: local,
      failureReporter: RecordingFailureReporter(),
    );
    final snapshot = repository.snapshotForActiveAccount();

    account = NostrPublicKeyHex.parse(testAuthorPublicKey);
    client.publicKeyHex = account;
    await local.record(ActivityItem(
      id: ActivityId.parse('local-account-b'),
      type: localA.type,
      description: localA.description,
      occurredAt: localA.occurredAt,
    ));

    final items = await snapshot.load();

    expect(items.map((item) => item.id).toSet(), {
      localA.id,
      testEventId,
    });
  });
}

NostrEventRecord _reaction(String id, String viewer) {
  return NostrEventRecord(
    identity: NostrEventIdentity.parse(
      id: id,
      authorPublicKeyHex: testFanPublicKey,
      kind: 7,
    ),
    tags: [
      ['p', viewer],
    ],
    content: '+',
    createdAt: 1,
  );
}
