import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/features/engagement/data/nostr_engagement_repository.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_reference.dart';

void main() {
  test('other authors cannot crowd out the viewer reaction deletion', () async {
    final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey)
      ..events.addAll([
        _reaction(1, testViewerPublicKey),
        for (var index = 1; index <= 250; index += 1)
          _reaction(index + 1, _author(index)),
        _deletion(1000, testViewerPublicKey, publishedEventId(1), 1000),
        for (var index = 1; index <= 250; index += 1) ...[
          _deletion(
            2000 + index * 2,
            _author(index),
            publishedEventId(index + 1),
            2000 + index * 2,
          ),
          _deletion(
            2001 + index * 2,
            _author(index),
            publishedEventId(index + 1),
            2001 + index * 2,
          ),
        ],
      ]);

    final engagement = await NostrEngagementRepository(client).load(
      nostrReference(),
    );

    expect(engagement.viewerHasLiked, isFalse);
    expect(engagement.likeCount, 0);
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

NostrEventRecord _deletion(
  int sequence,
  String author,
  String target,
  int createdAt,
) {
  return NostrEventRecord(
    identity: NostrEventIdentity.parse(
      id: publishedEventId(sequence),
      authorPublicKeyHex: author,
      kind: 5,
    ),
    tags: [
      ['e', target],
    ],
    content: 'deleted',
    createdAt: createdAt,
  );
}

String _author(int sequence) {
  return (sequence + 10000).toRadixString(16).padLeft(64, '0');
}
