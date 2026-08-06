import 'dart:async';

import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_sharing/domain/shareable_video_file.dart';
import 'package:ghostr/features/video_sharing/domain/video_file_downloader.dart';
import 'package:ghostr/features/video_sharing/domain/video_file_share_port.dart';
import 'package:ghostr/features/video_sharing/domain/video_share_origin.dart';
import 'package:ghostr/features/video_sharing/domain/video_share_workflow.dart';

final class VideoShareRequestRecord {
  const VideoShareRequestRecord(this.media, this.origin);

  final VideoMediaSource media;
  final VideoShareOrigin origin;
}

final class FakeVideoShareWorkflow implements VideoShareWorkflow {
  FakeVideoShareWorkflow({this.isSupported = true, this.failure, this.pending});

  final bool isSupported;
  final Object? failure;
  final Completer<void>? pending;
  final requests = <VideoShareRequestRecord>[];

  @override
  bool supports(VideoMediaSource media) => isSupported;

  @override
  Future<void> share(
    VideoMediaSource media, {
    required VideoShareOrigin origin,
  }) async {
    requests.add(VideoShareRequestRecord(media, origin));
    if (failure case final Object error) throw error;
    await pending?.future;
  }
}

final class FakeVideoFileDownloader implements VideoFileDownloader {
  FakeVideoFileDownloader(this.result);

  final ShareableVideoFile result;
  final requests = <VideoMediaSource>[];

  @override
  Future<ShareableVideoFile> download(VideoMediaSource media) async {
    requests.add(media);
    return result;
  }
}

final class FakeVideoFileSharePort implements VideoFileSharePort {
  final files = <ShareableVideoFile>[];
  final origins = <VideoShareOrigin>[];

  @override
  Future<void> share(
    ShareableVideoFile file, {
    required VideoShareOrigin origin,
  }) async {
    files.add(file);
    origins.add(origin);
  }
}
