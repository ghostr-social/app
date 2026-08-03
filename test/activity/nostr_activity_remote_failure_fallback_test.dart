import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/features/activity/data/nostr_activity_repository.dart';

import '../support/fake_activity_repository.dart';
import '../support/fake_nostr_event_client.dart';
import '../support/nostr_test_values.dart';
import '../support/recording_failure_reporter.dart';
import '../support/sample_data.dart';

void main() {
  test('returns local history when relay activity cannot be read', () async {
    final localActivity = sampleActivity();
    final reporter = RecordingFailureReporter();
    final repository = NostrActivityRepository(
      client: _FailingActivityClient(),
      local: FakeActivityRepository(items: [localActivity]),
      failureReporter: reporter,
    );

    final items = await repository.load();

    expect(items, [localActivity]);
    expect(reporter.sources, ['NostrActivityRepository.loadRemote']);
  });
}

class _FailingActivityClient extends FakeNostrEventClient {
  _FailingActivityClient() : super(publicKeyHex: testViewerPublicKey);

  @override
  Future<List<NostrEventRecord>> query(NostrEventQuery query) {
    throw const AppFailure('Relays unavailable.');
  }
}
