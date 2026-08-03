import 'dart:convert';
import 'dart:io';

import 'package:crypto/crypto.dart';
import 'package:ghostr/core/media/video_media_cache_identity.dart';
import 'package:ghostr/core/media/video_media_source.dart';

String completedVideoCachePath(
  Directory directory,
  VideoMediaSource media,
) {
  final identity = media.cacheStorageIdentity.value;
  final digest = sha256.convert(utf8.encode(identity)).toString();
  return '${directory.path}${Platform.pathSeparator}$digest.video';
}

Future<VideoMediaSource?> completedVideoCacheMedia(
  File completed,
  VideoMediaSource remote,
) async {
  if (!await completed.exists()) return null;
  final cached = VideoMediaSource.cached(
    completed.path,
    remoteUrl: remote.remoteUrl!,
    fallbackUrls: remote.fallbackUrls,
    delivery: remote.remoteDelivery!,
  );
  final digest = remote.expectedSha256;
  var retained = digest == null
      ? cached
      : VideoMediaSource.withExpectedSha256(cached, digest.value);
  final scope = remote.cacheScope;
  if (scope != null) {
    retained = VideoMediaSource.withCacheScope(retained, scope.value);
  }
  return retained;
}
