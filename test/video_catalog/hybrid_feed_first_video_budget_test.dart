import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/engagement/domain/video_engagement.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/nostr_event_reference.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

import '../support/fake_nostr_comments_port.dart';
import '../support/fake_nostr_engagement_port.dart';
import '../support/fake_remote_video_source.dart';
import '../support/hybrid_repository_harness.dart';
import '../support/nostr_reference.dart';
import '../support/sample_data.dart';

void main() {
  test(
    'returns a discovered video while social hydration is still pending',
    () async {
      final post = samplePost(nostrReference: nostrReference());
      final harness = await buildHybridRepositoryHarness(
        FakeRemoteVideoSource(<VideoPost>[post]),
        ports: HybridHarnessPorts(
          engagement: _HangingEngagement(),
          comments: FakeNostrCommentsPort(),
        ),
      );

      final posts = await harness.feed
          .loadFeed(FeedKind.forYou)
          .timeout(const Duration(milliseconds: 500));

      expect(posts, [post]);
    },
  );
}

class _HangingEngagement extends FakeNostrEngagementPort {
  @override
  Future<Map<NostrEventId, VideoEngagement>> loadBatch(
    List<NostrEventReference> references,
  ) {
    return Completer<Map<NostrEventId, VideoEngagement>>().future;
  }
}
