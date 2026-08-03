import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/engagement/domain/nostr_engagement_port.dart';
import 'package:ghostr/features/engagement/domain/video_engagement.dart';
import 'package:ghostr/features/video_catalog/domain/nostr_event_reference.dart';

class FakeNostrEngagementPort implements NostrEngagementPort {
  final Map<String, VideoEngagement> engagements = <String, VideoEngagement>{};
  final List<VideoLikeIntent> intents = <VideoLikeIntent>[];
  AppFailure? loadFailure;

  @override
  Future<VideoEngagement> load(NostrEventReference reference) async {
    if (loadFailure case final failure?) throw failure;
    return engagements[reference.eventId] ??
        VideoEngagement(likeCount: 0, viewerHasLiked: false);
  }

  @override
  Future<Map<NostrEventId, VideoEngagement>> loadBatch(
    List<NostrEventReference> references,
  ) async {
    if (loadFailure case final failure?) throw failure;
    return <NostrEventId, VideoEngagement>{
      for (final reference in references)
        reference.eventId: engagements[reference.eventId] ??
            VideoEngagement(likeCount: 0, viewerHasLiked: false),
    };
  }

  @override
  Future<VideoEngagement> setLike(
    NostrEventReference reference,
    VideoLikeIntent intent,
  ) async {
    intents.add(intent);
    final current = await load(reference);
    final liked = intent == VideoLikeIntent.like;
    if (liked == current.viewerHasLiked) return current;
    final updated = VideoEngagement(
      likeCount: current.likeCount + (liked ? 1 : -1),
      viewerHasLiked: liked,
    );
    engagements[reference.eventId] = updated;
    return updated;
  }
}
