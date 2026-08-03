import 'dart:async';

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
  test('keeps an activity load scoped to its initiating account', () async {
    SharedPreferences.setMockInitialValues({});
    var account = NostrPublicKeyHex.parse(testViewerPublicKey);
    final local = LocalActivityRepository(
      await SharedPreferences.getInstance(),
      accountScope: AccountStorageScope(() => account),
    );
    final first = sampleActivity();
    await local.record(first);
    final client = _DelayedActivityClient();
    final repository = NostrActivityRepository(
      client: client,
      local: local,
      failureReporter: RecordingFailureReporter(),
    );

    final pending = repository.load();
    await client.started.future;
    account = NostrPublicKeyHex.parse(testAuthorPublicKey);
    client.publicKeyHex = account;
    await local.record(ActivityItem(
      id: ActivityId.parse('account-b'),
      type: first.type,
      description: first.description,
      occurredAt: first.occurredAt,
    ));
    client.release.complete();

    final result = await pending;
    expect(result.map((item) => item.id), [first.id]);
  });
}

class _DelayedActivityClient extends FakeNostrEventClient {
  _DelayedActivityClient() : super(publicKeyHex: testViewerPublicKey);

  final started = Completer<void>();
  final release = Completer<void>();

  @override
  Future<List<NostrEventRecord>> query(NostrEventQuery query) async {
    if (!started.isCompleted) started.complete();
    await release.future;
    return const [];
  }
}
