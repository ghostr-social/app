import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/activity/data/local_activity_repository.dart';
import 'package:ghostr/features/activity/data/nostr_activity_repository.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_event_fixture.dart';
import '../support/nostr_test_values.dart';
import '../support/recording_failure_reporter.dart';
import '../support/test_account_storage_scope.dart';

void main() {
  test('returns relay activity when local history cannot be read', () async {
    final scope = testAccountStorageScope();
    SharedPreferences.setMockInitialValues({
      scope.capture().key('ghostr.activity.items'): '{malformed',
    });
    final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey)
      ..events.add(nostrEventFixture(
        testEventId,
        testAuthorPublicKey,
        7,
        const [
          ['p', testViewerPublicKey],
        ],
      ));
    final reporter = RecordingFailureReporter();
    final repository = NostrActivityRepository(
      client: client,
      local: LocalActivityRepository(
        await SharedPreferences.getInstance(),
        accountScope: scope,
      ),
      failureReporter: reporter,
    );

    final items = await repository.load();

    expect(items.single.id.value, testEventId);
    expect(reporter.sources, ['NostrActivityRepository.loadLocal']);
  });
}
