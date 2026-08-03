import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/features/engagement/data/nostr_engagement_repository.dart';
import 'package:ghostr/features/engagement/domain/nostr_engagement_port.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_reference.dart';

void main() {
  test('an unlike deletes the session-journaled reaction when reads fail',
      () async {
    final client = _FlakyReadNostrClient();
    final repository = NostrEngagementRepository(client);
    final reference = nostrReference();
    await repository.setLike(reference, VideoLikeIntent.like);
    client.failReads = true;

    final engagement = await repository.setLike(
      reference,
      VideoLikeIntent.unlike,
    );

    expect(engagement.viewerHasLiked, isFalse);
    final deletion = client.events.last;
    expect(deletion.kind.value, 5);
    expect(deletion.tagValues('e'), contains(client.events.first.id));
  });
}

class _FlakyReadNostrClient extends FakeNostrEventClient {
  _FlakyReadNostrClient() : super(publicKeyHex: testViewerPublicKey);

  bool failReads = false;

  @override
  Future<List<NostrEventRecord>> query(NostrEventQuery query) {
    if (failReads) {
      throw const AppFailure('Could not read engagement from Nostr relays.');
    }
    return super.query(query);
  }

  @override
  Future<List<NostrEventRecord>> queryBatch(List<NostrEventQuery> batch) {
    if (failReads) {
      throw const AppFailure('Could not read engagement from Nostr relays.');
    }
    return super.queryBatch(batch);
  }
}
