import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/domain/video_repost_attribution.dart';

/// Selects original content and feed occurrence independently for each video.
List<VideoPost> canonicalVideoPosts(Iterable<VideoPost> posts) {
  final selected = <String, _CanonicalVideo>{};
  for (final post in posts) {
    selected.update(
      _coordinate(post),
      (current) => current..consider(post),
      ifAbsent: () => _CanonicalVideo(post),
    );
  }
  final result = selected.values.map((item) => item.combined()).toList();
  result.sort(_newestOccurrenceFirst);
  return result;
}

String _coordinate(VideoPost post) {
  final reference = post.nostrReference;
  final identifier = reference?.coordinateIdentifier;
  final kind = reference?.kind.value;
  if (reference == null ||
      identifier == null ||
      kind! < 30000 ||
      kind >= 40000) {
    return post.id.value;
  }
  return '$kind:${reference.authorPublicKeyHex.value}:${identifier.value}';
}

bool _newerContent(VideoPost incoming, VideoPost current) {
  final time = incoming.publishedAt.compareTo(current.publishedAt);
  return time > 0 ||
      (time == 0 && incoming.id.value.compareTo(current.id.value) < 0);
}

bool _newerOccurrence(VideoPost incoming, VideoPost current) {
  final time = incoming.feedActivityAt.compareTo(current.feedActivityAt);
  if (time != 0) return time > 0;
  final incomingDirect = incoming.repost == null;
  final currentDirect = current.repost == null;
  if (incomingDirect != currentDirect) return incomingDirect;
  return _activityId(incoming).compareTo(_activityId(current)) < 0;
}

int _newestOccurrenceFirst(VideoPost left, VideoPost right) {
  final time = right.feedActivityAt.compareTo(left.feedActivityAt);
  return time != 0 ? time : _activityId(left).compareTo(_activityId(right));
}

String _activityId(VideoPost post) =>
    post.repost?.eventId.value ?? post.id.value;

final class _CanonicalVideo {
  _CanonicalVideo(VideoPost post) : content = post, occurrence = post;

  VideoPost content;
  VideoPost occurrence;

  void consider(VideoPost post) {
    if (_newerContent(post, content)) content = post;
    if (_newerOccurrence(post, occurrence)) occurrence = post;
  }

  VideoPost combined() {
    if (identical(content, occurrence) || _pinsExactEvent(occurrence)) {
      return occurrence;
    }
    return VideoPost(
      identity: VideoPostIdentity(
        id: content.id,
        creator: content.creator,
        nostrReference: content.nostrReference,
        repost: occurrence.repost,
      ),
      content: content.content,
      metrics: content.metrics,
      repostContext: content.repostContext,
    );
  }
}

bool _pinsExactEvent(VideoPost post) {
  return post.repost?.target == VideoRepostTarget.specificEvent;
}
