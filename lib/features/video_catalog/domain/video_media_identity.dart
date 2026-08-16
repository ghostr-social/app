import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/media/video_identity_url.dart';
import 'package:ghostr/core/media/video_url_sha256.dart';
import 'package:ghostr/features/video_catalog/domain/video_interaction_target.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

/// Whether two revisions still point at the same playable media.
bool sharesVideoMedia(VideoPost left, VideoPost right) {
  if (left.media.remoteDelivery != right.media.remoteDelivery) return false;
  final leftDigest = _mediaDigest(left.media);
  final rightDigest = _mediaDigest(right.media);
  if (leftDigest != null && rightDigest != null && leftDigest != rightDigest) {
    return false;
  }
  final leftUrls = _identityUrls(left.media).toSet();
  if (_identityUrls(right.media).any(leftUrls.contains)) return true;
  return leftDigest != null && leftDigest == rightDigest;
}

/// The identities one clip travels under: its event coordinate, its media
/// URLs, and its file digest. Two posts sharing any of them play the same
/// video, and the feed must never queue that video twice.
final class SeenVideoIdentities {
  SeenVideoIdentities([Iterable<VideoPost> posts = const []]) {
    posts.forEach(add);
  }

  final _targets = <VideoInteractionTarget>{};
  final _urls = <String>{};
  final _digests = <String>{};

  /// Claims every identity of [post]; false when one was already claimed.
  bool add(VideoPost post) {
    final target = VideoInteractionTarget.fromPost(post);
    final urls = _identityUrls(post.media);
    final digest = _mediaDigest(post.media);
    final duplicate =
        _targets.contains(target) ||
        urls.any(_urls.contains) ||
        (digest != null && _digests.contains(digest));
    _targets.add(target);
    _urls.addAll(urls);
    if (digest != null) _digests.add(digest);
    return !duplicate;
  }
}

Iterable<String> _identityUrls(VideoMediaSource media) => media.remoteUrls
    .map(canonicalVideoIdentityUrl)
    .where((url) => url.isNotEmpty);

String? _mediaDigest(VideoMediaSource media) {
  if (media.expectedSha256 case final digest?) return digest.value;
  for (final url in media.remoteUrls) {
    if (inferVideoSha256FromUrl(url) case final digest?) return digest.value;
  }
  return null;
}

/// The posts of [posts] with every later same-video repeat removed.
List<VideoPost> distinctVideoPosts(List<VideoPost> posts) {
  final seen = SeenVideoIdentities();
  return [
    for (final post in posts)
      if (seen.add(post)) post,
  ];
}
