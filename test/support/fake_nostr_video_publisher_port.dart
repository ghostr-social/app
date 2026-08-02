import 'package:ghostr/core/media/selected_media.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/publish/domain/nostr_video_publisher_port.dart';
import 'package:ghostr/features/session/domain/user_session.dart';
import 'package:ghostr/features/video_catalog/domain/nostr_event_reference.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/domain/video_post_id.dart';

import 'nostr_test_values.dart';

class FakeNostrVideoPublisherPort implements NostrVideoPublisherPort {
  int publishCount = 0;

  @override
  Future<VideoPost> publish({
    required UserSession session,
    required SelectedMedia media,
    required String caption,
  }) async {
    publishCount += 1;
    final id = VideoPostId.parse(publishedEventId(publishCount));
    return VideoPost(
      identity: VideoPostIdentity(
        id: id,
        creator: session.profile,
        nostrReference: NostrEventReference(
          eventId: NostrEventId.parse(id.value),
          authorPublicKeyHex: session.identity.publicKeyHex,
          kind: NostrEventKind.parse(22),
        ),
      ),
      content: VideoPostContent(
        caption: caption,
        songName: media.label,
        media: VideoMediaSource.remote('https://media.example/$id.mp4'),
        publishedAt: DateTime(2026, 8, 2),
      ),
      metrics: VideoPostMetrics(
        likeCount: 0,
        commentCount: 0,
        viewerHasLiked: false,
      ),
    );
  }
}
