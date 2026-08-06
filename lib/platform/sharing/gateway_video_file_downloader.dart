import 'dart:convert';
import 'dart:io';

import 'package:crypto/crypto.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/errors/boundary_failure.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/progressive_playback_gateway_port.dart';
import 'package:ghostr/features/video_sharing/domain/shareable_video_file.dart';
import 'package:ghostr/features/video_sharing/domain/video_file_downloader.dart';

typedef TemporaryDirectoryPath = Future<String> Function();

abstract interface class VideoFileTransfer {
  Future<void> transfer(Uri source, String destination);
}

final class GatewayVideoFileDownloader implements VideoFileDownloader {
  const GatewayVideoFileDownloader({
    required ProgressivePlaybackGatewayPort gateway,
    required VideoFileTransfer transfer,
    required TemporaryDirectoryPath temporaryDirectoryPath,
  }) : _gateway = gateway,
       _transfer = transfer,
       _temporaryDirectoryPath = temporaryDirectoryPath;

  final ProgressivePlaybackGatewayPort _gateway;
  final VideoFileTransfer _transfer;
  final TemporaryDirectoryPath _temporaryDirectoryPath;

  @override
  Future<ShareableVideoFile> download(VideoMediaSource media) async {
    try {
      final resolved = await _gateway.resolve(media);
      final directory = await _temporaryDirectoryPath();
      final destination = _destination(directory, media);
      await _transfer.transfer(resolved.playbackUri, destination);
      return ShareableVideoFile.parse(destination);
    } on AppFailure {
      rethrow;
    } on Object catch (error, stackTrace) {
      throw translatedBoundaryFailure(
        source: 'ghostr.video.share.gateway',
        message: 'Could not download this video.',
        error: error,
        stackTrace: stackTrace,
      );
    }
  }
}

String _destination(String directory, VideoMediaSource media) {
  final source = media.remoteUrl;
  if (source == null) {
    throw ArgumentError.value(media, 'media', 'Remote media is required.');
  }
  final id = sha256.convert(utf8.encode(source));
  return '$directory${Platform.pathSeparator}ghostr-share-$id.mp4';
}
