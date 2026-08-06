import 'dart:ui';

import 'package:ghostr/core/errors/boundary_failure.dart';
import 'package:ghostr/features/video_sharing/domain/shareable_video_file.dart';
import 'package:ghostr/features/video_sharing/domain/video_file_share_port.dart';
import 'package:ghostr/features/video_sharing/domain/video_share_origin.dart';
import 'package:share_plus/share_plus.dart';

typedef PlatformFileShare = Future<ShareResult> Function(ShareParams params);

final class SharePlusVideoFilePort implements VideoFileSharePort {
  SharePlusVideoFilePort({PlatformFileShare platformShare = shareVideoFile})
    : _platformShare = platformShare;

  final PlatformFileShare _platformShare;

  @override
  Future<void> share(
    ShareableVideoFile file, {
    required VideoShareOrigin origin,
  }) async {
    try {
      await _platformShare(_parameters(file, origin));
    } on Object catch (error, stackTrace) {
      throw translatedBoundaryFailure(
        source: 'ghostr.video.share.sheet',
        message: 'Could not open video sharing.',
        error: error,
        stackTrace: stackTrace,
      );
    }
  }
}

// coverage:ignore-start
Future<ShareResult> shareVideoFile(ShareParams params) {
  return SharePlus.instance.share(params);
}
// coverage:ignore-end

ShareParams _parameters(ShareableVideoFile file, VideoShareOrigin origin) {
  return ShareParams(
    title: 'Share video',
    files: [XFile(file.path, mimeType: 'video/mp4')],
    fileNameOverrides: const ['ghostr-video.mp4'],
    sharePositionOrigin: Rect.fromLTWH(
      origin.left,
      origin.top,
      origin.width,
      origin.height,
    ),
  );
}
