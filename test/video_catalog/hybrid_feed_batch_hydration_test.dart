import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/features/comments/data/nostr_comments_repository.dart';
import 'package:ghostr/features/engagement/data/nostr_engagement_repository.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';

import '../support/fakes.dart';
import '../support/hybrid_repository_harness.dart';
import '../support/nostr_reference.dart';
import '../support/sample_data.dart';

void main() {
  test('hydrates a mixed feed with at most eight relay requests', () async {
    final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey);
    final regular = nostrReference();
    final addressable = nostrReference(
      eventId: secondTestEventId,
      kind: 34236,
      identifier: 'clip-2',
    );
    final coordinate =
        '${addressable.kind}:${addressable.authorPublicKeyHex}:clip-2';
    client.events.addAll([
      _event(publishedEventId(40), 7, '+', [
        ['e', regular.eventId],
      ]),
      _event(publishedEventId(41), 7, '+', [
        ['a', coordinate],
      ]),
      _event(publishedEventId(42), 1111, 'Regular comment', [
        ['E', regular.eventId],
      ]),
      _event(publishedEventId(43), 1111, 'Addressable comment', [
        ['A', coordinate],
      ]),
    ]);
    final harness = await buildHybridRepositoryHarness(
      FakeRemoteVideoSource([
        samplePost(id: 'regular', nostrReference: regular),
        samplePost(id: 'addressable', nostrReference: addressable),
      ]),
      ports: HybridHarnessPorts(
        engagement: NostrEngagementRepository(client),
        comments: NostrCommentsRepository(client),
      ),
    );

    final posts = await harness.feed.loadFeed(FeedKind.forYou);
    final byId = {for (final post in posts) post.nostrReference!.eventId: post};

    final viewerReactionBatches = client.queryBatches.where((batch) {
      return batch.isNotEmpty &&
          batch.every((query) {
            return query.kinds.single.value == 7 &&
                query.authors.length == 1 &&
                query.authors.single == client.publicKeyHex;
          });
    });
    expect(client.requestCount, lessThanOrEqualTo(8));
    expect(viewerReactionBatches, hasLength(2));
    expect(byId[regular.eventId]!.likeCount, 1);
    expect(byId[regular.eventId]!.commentCount, 1);
    expect(byId[addressable.eventId]!.likeCount, 1);
    expect(byId[addressable.eventId]!.commentCount, 1);
  });
}

NostrEventRecord _event(
  String id,
  int kind,
  String content,
  List<List<String>> tags,
) {
  return NostrEventRecord(
    identity: NostrEventIdentity.parse(
      id: id,
      authorPublicKeyHex: testFanPublicKey,
      kind: kind,
    ),
    tags: tags,
    content: content,
    createdAt: 1,
  );
}
