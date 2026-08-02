import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/comments/data/nostr_comments_repository.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_reference.dart';

void main() {
  test('publishes and loads a NIP-22 comment thread', () async {
    final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey);
    final publishedAt = DateTime.utc(2026, 8, 2, 12, 30);
    final repository = NostrCommentsRepository(
      client,
      clock: () => publishedAt,
    );
    final video = nostrReference();

    final comment = await repository.publish(
      reference: video,
      content: 'First!',
    );
    expect(comment.createdAt, publishedAt);
    expect(client.events.first.kind, 1111);
    expect(
        client.events.first.tags,
        containsAll(<List<String>>[
          ['E', testEventId],
          ['K', '22'],
          ['P', testCreatorPublicKey],
          ['e', testEventId],
          ['k', '22'],
          ['p', testCreatorPublicKey],
        ]));

    await repository.publish(
      reference: video,
      content: 'A reply',
      replyTo: comment,
    );
    expect(
        client.events.last.tags,
        containsAll(<List<String>>[
          ['E', testEventId],
          ['K', '22'],
          ['P', testCreatorPublicKey],
          ['e', publishedEventId(1)],
          ['k', '1111'],
          ['p', testViewerPublicKey],
        ]));

    final thread = await repository.load(video);

    expect(thread.map((item) => item.content), ['First!', 'A reply']);
    expect(thread.last.parentCommentId, publishedEventId(1));
  });
}
