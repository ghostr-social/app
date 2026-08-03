import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/features/engagement/data/nostr_engagement_repository.dart';
import 'package:ghostr/features/engagement/domain/nostr_engagement_port.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_reference.dart';

void main() {
  test('an unlike without session evidence surfaces the read failure',
      () async {
    final client = _ReadFailingNostrClient();
    final repository = NostrEngagementRepository(client);

    await expectLater(
      repository.setLike(nostrReference(), VideoLikeIntent.unlike),
      throwsA(isA<AppFailure>()),
    );
    expect(client.events, isEmpty);
  });
}

class _ReadFailingNostrClient extends FakeNostrEventClient {
  _ReadFailingNostrClient() : super(publicKeyHex: testViewerPublicKey);

  @override
  Future<List<NostrEventRecord>> query(NostrEventQuery query) {
    throw const AppFailure('Could not read engagement from Nostr relays.');
  }

  @override
  Future<List<NostrEventRecord>> queryBatch(List<NostrEventQuery> batch) {
    throw const AppFailure('Could not read engagement from Nostr relays.');
  }
}
