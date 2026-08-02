import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/errors/failure_reporter.dart';
import 'package:ghostr/features/video_catalog/domain/nostr_video_interactions.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/published_video_store.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_source.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/domain/video_post_reader.dart';

class HybridVideoReader implements VideoPostReader {
  const HybridVideoReader({
    required RemoteVideoSource remote,
    required PublishedVideoStore local,
    required NostrVideoInteractions interactions,
    required FailureReporter failureReporter,
  })  : _remote = remote,
        _local = local,
        _interactions = interactions,
        _failureReporter = failureReporter;

  final RemoteVideoSource _remote;
  final PublishedVideoStore _local;
  final NostrVideoInteractions _interactions;
  final FailureReporter _failureReporter;

  @override
  Future<List<VideoPost>> load({
    Set<ProfileId>? creatorIds,
    String? searchQuery,
  }) async {
    final localPosts = await _local.loadPublishedPosts();
    try {
      final remotePosts = await _remote.loadRemoteFeed(
        creatorIds: creatorIds,
        searchQuery: searchQuery,
      );
      return _hydrate(_merge(localPosts, remotePosts));
    } on AppFailure catch (error, stackTrace) {
      _report(error, stackTrace);
      if (localPosts.isEmpty) rethrow;
      return localPosts;
    }
  }

  Future<List<VideoPost>> _hydrate(List<VideoPost> posts) {
    return Future.wait(posts.map(_interactions.hydrate));
  }

  List<VideoPost> _merge(List<VideoPost> local, List<VideoPost> remote) {
    final merged = <String, VideoPost>{};
    for (final post in <VideoPost>[...local, ...remote]) {
      final key = _postCoordinate(post);
      final current = merged[key];
      if (current == null || _isNewer(post, current)) merged[key] = post;
    }
    final items = merged.values.toList();
    items.sort((left, right) => right.publishedAt.compareTo(left.publishedAt));
    return items;
  }

  String _postCoordinate(VideoPost post) {
    final reference = post.nostrReference;
    final identifier = reference?.identifier;
    final kind = reference?.kind.value;
    if (reference == null ||
        identifier == null ||
        kind! < 30000 ||
        kind >= 40000) {
      return post.id.value;
    }
    return '$kind:${reference.authorPublicKeyHex.value}:${identifier.value}';
  }

  bool _isNewer(VideoPost incoming, VideoPost current) {
    final time = incoming.publishedAt.compareTo(current.publishedAt);
    return time > 0 ||
        (time == 0 && incoming.id.value.compareTo(current.id.value) < 0);
  }

  void _report(Object error, StackTrace stackTrace) {
    _failureReporter.report(
      source: 'HybridVideoReader.load',
      error: error,
      stackTrace: stackTrace,
    );
  }
}
