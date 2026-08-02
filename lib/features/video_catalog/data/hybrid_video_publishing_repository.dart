import 'package:ghostr/core/media/selected_media.dart';
import 'package:ghostr/features/publish/domain/nostr_video_publisher_port.dart';
import 'package:ghostr/features/publish/domain/video_publishing_repository.dart';
import 'package:ghostr/features/session/domain/user_session.dart';
import 'package:ghostr/features/video_catalog/domain/published_video_store.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

class HybridVideoPublishingRepository implements VideoPublishingRepository {
  const HybridVideoPublishingRepository(this._local, this._publisher);

  final PublishedVideoStore _local;
  final NostrVideoPublisherPort _publisher;

  @override
  Future<VideoPost> publish({
    required UserSession session,
    required SelectedMedia media,
    required String caption,
  }) async {
    final posts = await _local.loadPublishedPosts();
    final post = await _publisher.publish(
      session: session,
      media: media,
      caption: caption,
    );
    await _local.savePublishedPosts(<VideoPost>[post, ...posts]);
    return post;
  }
}
