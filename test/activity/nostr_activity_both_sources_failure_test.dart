import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/features/activity/data/local_activity_repository.dart';
import 'package:ghostr/features/activity/data/nostr_activity_repository.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_test_values.dart';
import '../support/recording_failure_reporter.dart';
import '../support/test_account_storage_scope.dart';

void main() {
  test('reports both source failures and surfaces an AppFailure', () async {
    final scope = testAccountStorageScope();
    SharedPreferences.setMockInitialValues({
      scope.capture().key('ghostr.activity.items'): '{malformed',
    });
    final reporter = RecordingFailureReporter();
    final repository = NostrActivityRepository(
      client: _FailingActivityClient(),
      local: LocalActivityRepository(
        await SharedPreferences.getInstance(),
        accountScope: scope,
      ),
      failureReporter: reporter,
    );

    await expectLater(repository.load(), throwsA(isA<AppFailure>()));

    expect(
      reporter.sources,
      unorderedEquals([
        'NostrActivityRepository.loadRemote',
        'NostrActivityRepository.loadLocal',
      ]),
    );
  });
}

class _FailingActivityClient extends FakeNostrEventClient {
  _FailingActivityClient() : super(publicKeyHex: testViewerPublicKey);

  @override
  Future<List<NostrEventRecord>> query(NostrEventQuery query) {
    throw const AppFailure('Relays unavailable.');
  }
}
