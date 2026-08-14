import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/domain/video_repost_attribution.dart';

import 'nostr_test_values.dart';
import 'repostable_post_sample.dart';

VideoPost repostedPost() {
  final original = repostablePost();
  return VideoPost(
    identity: VideoPostIdentity(
      id: original.id,
      creator: original.creator,
      nostrReference: original.nostrReference,
      repost: VideoRepostAttribution(
        eventId: NostrEventId.parse(secondTestEventId),
        reposter: ProfileSummary(
          id: ProfileId.parse(testViewerNpub),
          displayName: 'Bob Relay',
          handle: '@bob',
          avatarUrl: null,
        ),
        repostedAt: DateTime.utc(2026, 2, 1),
        target: VideoRepostTarget.specificEvent,
      ),
    ),
    content: original.content,
    metrics: original.metrics,
  );
}
