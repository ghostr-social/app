import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/engagement/domain/video_engagement.dart';
import 'package:ghostr/features/video_catalog/domain/nostr_event_reference.dart';

enum VideoLikeIntent { like, unlike }

abstract interface class NostrEngagementPort {
  Future<VideoEngagement> load(NostrEventReference reference);

  Future<Map<NostrEventId, VideoEngagement>> loadBatch(
    List<NostrEventReference> references,
  );

  Future<VideoEngagement> setLike(
    NostrEventReference reference,
    VideoLikeIntent intent,
  );
}
