import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/nostr_event_reference.dart';
import 'package:ghostr/features/video_catalog/domain/repost_hydrated_video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_refresh_repository.dart';

import '../support/fakes.dart';
import '../support/nostr_test_values.dart';
import '../support/sample_data.dart';

void main() {
  test('repost hydration preserves exact refresh eligibility', () async {
    final eligible = samplePost(
      id: 'eligible-revision',
      nostrReference: _addressable(testEventId),
    );
    final rejected = samplePost(
      id: 'rejected-revision',
      nostrReference: _addressable(secondTestEventId),
    );
    final source = _RefreshSource(
      VideoFeedRefreshSnapshot(
        allPosts: [eligible, rejected],
        eligiblePosts: [eligible],
      ),
    );
    final feed = RepostHydratedVideoFeedRepository(source, source);

    final result = await feed.loadRefresh(FeedKind.forYou);

    expect(result.eligiblePosts.map((post) => post.id.value), [
      'eligible-revision',
    ]);
  });
}

final class _RefreshSource extends FakeVideoCatalogRepository
    implements VideoFeedRefreshRepository {
  _RefreshSource(this.snapshot) : super(forYouFeed: snapshot.allPosts);

  final VideoFeedRefreshSnapshot snapshot;

  @override
  Future<VideoFeedRefreshSnapshot> loadRefresh(FeedKind kind) async => snapshot;
}

NostrEventReference _addressable(String eventId) {
  return NostrEventReference(
    eventId: NostrEventId.parse(eventId),
    authorPublicKeyHex: NostrPublicKeyHex.parse(testAuthorPublicKey),
    kind: NostrEventKind.parse(34236),
    details: NostrEventReferenceDetails(
      identifier: NostrEventIdentifier.parse('stable-coordinate'),
    ),
  );
}
