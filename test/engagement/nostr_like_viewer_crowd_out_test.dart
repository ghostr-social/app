import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/features/engagement/data/nostr_engagement_repository.dart';
import 'package:ghostr/features/engagement/domain/nostr_engagement_port.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_reference.dart';

void main() {
  test('unlike finds an older viewer reaction beyond the public result limit',
      () async {
    final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey)
      ..events.add(_reaction(1, testViewerPublicKey))
      ..events.addAll([
        for (var sequence = 2; sequence <= 501; sequence += 1)
          _reaction(sequence, _publicKey(sequence)),
      ]);
    final repository = NostrEngagementRepository(client);

    final result = await repository.setLike(
      nostrReference(),
      VideoLikeIntent.unlike,
    );

    expect(result.viewerHasLiked, isFalse);
    expect(client.events.last.kind, 5);
    expect(client.events.last.tagValues('e'), [publishedEventId(1)]);
  });
}

NostrEventRecord _reaction(int sequence, String author) {
  return NostrEventRecord(
    identity: NostrEventIdentity.parse(
      id: publishedEventId(sequence),
      authorPublicKeyHex: author,
      kind: 7,
    ),
    tags: const [
      ['e', testEventId],
    ],
    content: '+',
    createdAt: sequence,
  );
}

String _publicKey(int sequence) {
  return (sequence + 1000).toRadixString(16).padLeft(64, '0');
}
