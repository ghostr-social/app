import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_sharing/domain/shareable_video_file.dart';
import 'package:ghostr/features/video_sharing/domain/video_file_downloader.dart';
import 'package:ghostr/features/video_sharing/domain/video_file_share_port.dart';
import 'package:ghostr/features/video_sharing/domain/video_share_origin.dart';
import 'package:ghostr/features/video_sharing/domain/video_share_workflow.dart';

final class DefaultVideoShareWorkflow implements VideoShareWorkflow {
  const DefaultVideoShareWorkflow({
    required VideoFileDownloader downloader,
    required VideoFileSharePort sharePort,
  }) : _downloader = downloader,
       _sharePort = sharePort;

  final VideoFileDownloader _downloader;
  final VideoFileSharePort _sharePort;

  @override
  bool supports(VideoMediaSource media) {
    return _existingPath(media) != null ||
        media.remoteDelivery == VideoMediaDelivery.progressive;
  }

  @override
  Future<void> share(
    VideoMediaSource media, {
    required VideoShareOrigin origin,
  }) async {
    if (!supports(media)) {
      throw const AppFailure('Sharing is unavailable for this video.');
    }
    final path = _existingPath(media);
    final file = path == null
        ? await _downloader.download(media)
        : ShareableVideoFile.parse(path);
    await _sharePort.share(file, origin: origin);
  }
}

String? _existingPath(VideoMediaSource media) {
  return media.localPath ?? media.importPath;
}
