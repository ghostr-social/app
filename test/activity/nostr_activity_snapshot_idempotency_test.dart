import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/storage/account_storage_scope.dart';
import 'package:ghostr/features/activity/data/local_activity_repository.dart';
import 'package:ghostr/features/activity/data/nostr_activity_repository.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_test_values.dart';
import '../support/recording_failure_reporter.dart';

void main() {
  test('a pinned Nostr activity snapshot preserves its local account',
      () async {
    SharedPreferences.setMockInitialValues({});
    var account = NostrPublicKeyHex.parse(testViewerPublicKey);
    final local = LocalActivityRepository(
      await SharedPreferences.getInstance(),
      accountScope: AccountStorageScope(() => account),
    );
    final repository = NostrActivityRepository(
      client: FakeNostrEventClient(publicKeyHex: testViewerPublicKey),
      local: local,
      failureReporter: RecordingFailureReporter(),
    );
    final pinned = repository.snapshotForActiveAccount();

    account = NostrPublicKeyHex.parse(testAuthorPublicKey);

    expect(pinned.snapshotForActiveAccount(), same(pinned));
  });
}
