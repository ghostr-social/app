import 'dart:async';

import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/errors/failure_reporter.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/comments/domain/nostr_comments_port.dart';
import 'package:ghostr/features/comments/domain/video_comment.dart';
import 'package:ghostr/features/engagement/domain/nostr_engagement_port.dart';
import 'package:ghostr/features/engagement/domain/video_engagement.dart';
import 'package:ghostr/features/video_catalog/domain/nostr_event_reference.dart';
import 'package:ghostr/features/video_catalog/domain/video_like_policy.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

class NostrVideoInteractions {
  const NostrVideoInteractions(
    this._engagement,
    this._comments,
    this._failureReporter, {
    VideoLikePolicy likePolicy = const VideoLikePolicy(),
    Duration hydrationTimeout = const Duration(seconds: 10),
  })  : _likePolicy = likePolicy,
        _hydrationTimeout = hydrationTimeout;

  final NostrEngagementPort _engagement;
  final NostrCommentsPort _comments;
  final FailureReporter _failureReporter;
  final VideoLikePolicy _likePolicy;
  final Duration _hydrationTimeout;

  Future<VideoPost> hydrate(VideoPost post) async {
    return (await hydrateAll(<VideoPost>[post])).single;
  }

  Future<List<VideoPost>> hydrateAll(List<VideoPost> posts) async {
    final references = posts
        .map((post) => post.nostrReference)
        .whereType<NostrEventReference>()
        .toList(growable: false);
    if (references.isEmpty) return posts;
    try {
      return await _hydrateAll(posts, references).timeout(_hydrationTimeout);
    } on TimeoutException catch (error, stackTrace) {
      _report('NostrVideoInteractions.hydrateAll', error, stackTrace);
      return posts;
    }
  }

  Future<List<VideoPost>> _hydrateAll(
    List<VideoPost> posts,
    List<NostrEventReference> references,
  ) async {
    final engagementFuture = _loadEngagementBatch(references);
    final commentCountsFuture = _loadCommentCountsBatch(references);
    final engagements = await engagementFuture;
    final commentCounts = await commentCountsFuture;
    return posts.map((post) {
      return _withHydratedInteractions(post, engagements, commentCounts);
    }).toList(growable: false);
  }

  VideoPost _withHydratedInteractions(
    VideoPost post,
    _EngagementLoad engagements,
    _CommentCountLoad commentCounts,
  ) {
    final reference = post.nostrReference;
    if (reference == null) return post;
    final eventId = reference.eventId;
    final engagement = _engagementFor(post, eventId, engagements);
    return post.withInteraction(
      VideoInteractionUpdate(
        likeCount: engagement.likeCount,
        viewerHasLiked: engagement.viewerHasLiked,
        commentCount: commentCounts.values[eventId] ?? post.commentCount,
        observations: VideoMetricObservationUpdate(
          likes: _observation(
            engagements.succeeded,
            engagements.values.containsKey(eventId),
          ),
          comments: _observation(
            commentCounts.succeeded,
            commentCounts.values.containsKey(eventId),
          ),
        ),
      ),
    );
  }

  VideoEngagement _engagementFor(
    VideoPost post,
    NostrEventId eventId,
    _EngagementLoad engagements,
  ) {
    return engagements.values[eventId] ??
        VideoEngagement(
          likeCount: post.likeCount,
          viewerHasLiked: post.viewerHasLiked,
        );
  }

  VideoMetricObservation _observation(bool succeeded, bool hasValue) {
    return succeeded && hasValue
        ? VideoMetricObservation.observed
        : VideoMetricObservation.unobserved;
  }

  Future<VideoPost> toggleLike(VideoPost post) async {
    final reference = post.nostrReference;
    if (reference == null) return _likePolicy.toggle(post);
    final intent =
        post.viewerHasLiked ? VideoLikeIntent.unlike : VideoLikeIntent.like;
    final engagement = await _engagement.setLike(reference, intent);
    // A journal-only mutation reports a count with no relay baseline, so the
    // post-derived expectation is the floor for the displayed count.
    final expected = _likePolicy.toggle(post).likeCount;
    return post.withInteraction(
      VideoInteractionUpdate(
        likeCount:
            engagement.likeCount > expected ? engagement.likeCount : expected,
        viewerHasLiked: engagement.viewerHasLiked,
      ),
    );
  }

  Future<List<VideoComment>> loadComments(VideoPost post) {
    final reference = post.nostrReference;
    if (reference == null) return Future.value(const <VideoComment>[]);
    return _comments.load(reference);
  }

  Future<VideoComment> publishComment({
    required VideoPost post,
    required String content,
    VideoComment? replyTo,
  }) {
    final reference = post.nostrReference;
    if (reference == null) {
      throw const AppFailure('This video has no Nostr event to comment on.');
    }
    return _comments.publish(
      reference: reference,
      content: content,
      replyTo: replyTo,
    );
  }

  Future<_EngagementLoad> _loadEngagementBatch(
    List<NostrEventReference> references,
  ) async {
    try {
      return (
        values: await _engagement.loadBatch(references),
        succeeded: true,
      );
    } on AppFailure catch (error, stackTrace) {
      _report('NostrVideoInteractions.loadEngagement', error, stackTrace);
      return (
        values: const <NostrEventId, VideoEngagement>{},
        succeeded: false
      );
    }
  }

  Future<_CommentCountLoad> _loadCommentCountsBatch(
    List<NostrEventReference> references,
  ) async {
    try {
      final comments = await _comments.loadBatch(references);
      return (
        values: <NostrEventId, int>{
          for (final entry in comments.entries) entry.key: entry.value.length,
        },
        succeeded: true,
      );
    } on AppFailure catch (error, stackTrace) {
      _report('NostrVideoInteractions.loadCommentCount', error, stackTrace);
      return (values: const <NostrEventId, int>{}, succeeded: false);
    }
  }

  void _report(String source, Object error, StackTrace stackTrace) {
    _failureReporter.report(
      source: source,
      error: error,
      stackTrace: stackTrace,
    );
  }
}

typedef _EngagementLoad = ({
  Map<NostrEventId, VideoEngagement> values,
  bool succeeded,
});
typedef _CommentCountLoad = ({
  Map<NostrEventId, int> values,
  bool succeeded,
});
