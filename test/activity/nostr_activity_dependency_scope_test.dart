import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/storage/account_storage_scope.dart';
import 'package:ghostr/features/activity/data/local_activity_repository.dart';
import 'package:ghostr/features/activity/data/nostr_activity_repository.dart';
import 'package:ghostr/features/activity/domain/activity_item.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_event_fixture.dart';
import '../support/nostr_test_values.dart';
import '../support/recording_failure_reporter.dart';
import '../support/sample_data.dart';

void main() {
  test('snapshot re-scopes a pre-pinned local store to its viewer', () async {
    SharedPreferences.setMockInitialValues({});
    var account = NostrPublicKeyHex.parse(testViewerPublicKey);
    final local = LocalActivityRepository(
      await SharedPreferences.getInstance(),
      accountScope: AccountStorageScope(() => account),
    );
    final activityA = sampleActivity();
    await local.record(activityA);
    final pinnedA = local.snapshotForActiveAccount();
    account = NostrPublicKeyHex.parse(testAuthorPublicKey);
    final activityB = ActivityItem(
      id: ActivityId.parse('local-account-b'),
      type: activityA.type,
      description: activityA.description,
      occurredAt: activityA.occurredAt,
    );
    await local.record(activityB);
    final client = FakeNostrEventClient(publicKeyHex: testAuthorPublicKey)
      ..events.add(nostrEventFixture(
        testEventId,
        testFanPublicKey,
        7,
        const [
          ['p', testAuthorPublicKey],
        ],
      ));
    final repository = NostrActivityRepository(
      client: client,
      local: pinnedA,
      failureReporter: RecordingFailureReporter(),
    );

    final items = await repository.snapshotForActiveAccount().load();

    expect(items.map((item) => item.id).toSet(), {
      activityB.id,
      testEventId,
    });
  });
}
