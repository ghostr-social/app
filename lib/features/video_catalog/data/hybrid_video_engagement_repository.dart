import 'package:ghostr/features/engagement/domain/video_engagement_repository.dart';
import 'package:ghostr/features/video_catalog/domain/nostr_video_interactions.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

class HybridVideoEngagementRepository implements VideoEngagementRepository {
  const HybridVideoEngagementRepository(this._interactions);

  final NostrVideoInteractions _interactions;

  @override
  Future<VideoPost> toggleLike(VideoPost post) {
    return _interactions.toggleLike(post);
  }
}
