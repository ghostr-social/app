import 'package:ghostr/core/async/keyed_serial_task_queue.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/errors/failure_reporter.dart';
import 'package:ghostr/core/media/selected_media.dart';
import 'package:ghostr/features/publish/domain/nostr_video_publisher_port.dart';
import 'package:ghostr/features/publish/domain/video_publication.dart';
import 'package:ghostr/features/publish/domain/video_publishing_repository.dart';
import 'package:ghostr/features/session/domain/user_session.dart';
import 'package:ghostr/features/video_catalog/domain/published_video_store.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

class HybridVideoPublishingRepository implements VideoPublishingRepository {
  HybridVideoPublishingRepository(
    this._local,
    this._publisher,
    this._failureReporter,
  ) : _cacheQueue = KeyedSerialTaskQueue();

  final PublishedVideoStore _local;
  final NostrVideoPublisherPort _publisher;
  final FailureReporter _failureReporter;
  final KeyedSerialTaskQueue _cacheQueue;

  @override
  Future<VideoPublication> publish({
    required UserSession session,
    required SelectedMedia media,
    required String caption,
  }) async {
    final local = _local.snapshotForActiveAccount();
    _requireAccount(local, session);
    final post = await _publisher.publish(
      session: session,
      media: media,
      caption: caption,
    );
    final cacheStatus = await _cacheQueue.run(
      local.accountPublicKey,
      () => _cache(local, post),
    );
    return VideoPublication(post: post, cacheStatus: cacheStatus);
  }

  void _requireAccount(PublishedVideoStore local, UserSession session) {
    if (local.accountPublicKey != session.identity.publicKeyHex) {
      throw const AppFailure('The active account changed. Try again.');
    }
  }

  Future<VideoPublicationCacheStatus> _cache(
    PublishedVideoStore local,
    VideoPost post,
  ) async {
    try {
      final posts = await local.loadPublishedPosts();
      await local.savePublishedPosts(<VideoPost>[post, ...posts]);
      return VideoPublicationCacheStatus.stored;
    } on Object catch (error, stackTrace) {
      _failureReporter.report(
        source: 'HybridVideoPublishingRepository.savePublishedPosts',
        error: error,
        stackTrace: stackTrace,
      );
      return VideoPublicationCacheStatus.unavailable;
    }
  }
}
