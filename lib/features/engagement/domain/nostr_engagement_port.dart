import 'package:ghostr/features/engagement/domain/video_engagement.dart';
import 'package:ghostr/features/video_catalog/domain/nostr_event_reference.dart';

abstract interface class NostrEngagementPort {
  Future<VideoEngagement> load(NostrEventReference reference);

  Future<VideoEngagement> toggleLike(NostrEventReference reference);
}
