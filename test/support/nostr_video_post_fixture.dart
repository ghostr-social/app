import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/video_catalog/domain/nostr_event_reference.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/domain/video_post_id.dart';

import 'nostr_test_values.dart';

class NostrVideoPostFixture {
  const NostrVideoPostFixture({
    required this.eventId,
    required this.mediaId,
    this.creator = const NostrCreatorFixture(),
    this.text = const NostrVideoTextFixture(),
  });

  final String eventId;
  final String mediaId;
  final NostrCreatorFixture creator;
  final NostrVideoTextFixture text;
}

class NostrCreatorFixture {
  const NostrCreatorFixture({
    this.npub = 'npub1creator',
    this.name = 'Nora Relay',
  });

  final String npub;
  final String name;
}

class NostrVideoTextFixture {
  const NostrVideoTextFixture({
    this.caption = 'Relay dance',
    this.songName = 'Original sound',
  });

  final String caption;
  final String songName;
}

VideoPost nostrVideoPost(NostrVideoPostFixture fixture) {
  return VideoPost(
    identity: VideoPostIdentity(
      id: VideoPostId.parse(fixture.eventId),
      creator: ProfileSummary(
        id: ProfileId.parse(fixture.creator.npub),
        displayName: fixture.creator.name,
        handle: '@${fixture.creator.npub}',
        avatarUrl: null,
      ),
      nostrReference: _reference(fixture),
    ),
    content: VideoPostContent(
      caption: fixture.text.caption,
      songName: fixture.text.songName,
      media: VideoMediaSource.remote(
        'https://source.example/${fixture.mediaId}.mp4',
      ),
      publishedAt: DateTime.utc(2026, 8, 2),
    ),
    metrics: VideoPostMetrics(
      likeCount: 4,
      commentCount: 2,
      viewerHasLiked: false,
    ),
  );
}

NostrEventReference _reference(NostrVideoPostFixture fixture) {
  return NostrEventReference(
    eventId: NostrEventId.parse(fixture.eventId),
    authorPublicKeyHex: NostrPublicKeyHex.parse(testCreatorPublicKey),
    kind: NostrEventKind.parse(34235),
    details: NostrEventReferenceDetails(
      identifier: NostrEventIdentifier.parse(fixture.mediaId),
    ),
  );
}
