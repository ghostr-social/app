import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/domain/video_repost_attribution.dart';

import 'nostr_test_values.dart';
import 'nostr_video_post_fixture.dart';
import 'sample_data.dart';

VideoPost addressableRevision(
  String eventId,
  DateTime publishedAt,
  String caption,
) {
  final post = nostrVideoPost(
    NostrVideoPostFixture(
      eventId: eventId,
      mediaId: 'same-video',
      text: NostrVideoTextFixture(caption: caption),
    ),
  );
  return VideoPost(
    identity: post.identity,
    content: VideoPostContent(
      caption: caption,
      songName: post.songName,
      media: post.media,
      publishedAt: publishedAt,
    ),
    metrics: post.metrics,
  );
}

VideoPost coordinateRepost(VideoPost post) {
  return VideoPost(
    identity: VideoPostIdentity(
      id: post.id,
      creator: post.creator,
      nostrReference: post.nostrReference,
      repost: VideoRepostAttribution(
        eventId: NostrEventId.parse(publishedTestEventId),
        reposter: sampleCreator(id: 'reposter'),
        repostedAt: DateTime.utc(2026, 3),
        target: VideoRepostTarget.coordinate,
      ),
    ),
    content: post.content,
    metrics: post.metrics,
  );
}
