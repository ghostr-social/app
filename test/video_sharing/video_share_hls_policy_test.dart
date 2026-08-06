import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_sharing/data/default_video_share_workflow.dart';
import 'package:ghostr/features/video_sharing/domain/shareable_video_file.dart';
import 'package:ghostr/features/video_sharing/domain/video_share_origin.dart';

import '../support/fake_video_sharing.dart';

void main() {
  test('refuses HLS instead of sharing its playlist URL', () async {
    final downloader = FakeVideoFileDownloader(
      ShareableVideoFile.parse('/tmp/unexpected.mp4'),
    );
    final sharePort = FakeVideoFileSharePort();
    final workflow = DefaultVideoShareWorkflow(
      downloader: downloader,
      sharePort: sharePort,
    );
    final media = VideoMediaSource.remote(
      'https://media.test/playlist.m3u8',
      delivery: VideoMediaDelivery.hls,
    );

    expect(workflow.supports(media), isFalse);
    await expectLater(
      workflow.share(media, origin: VideoShareOrigin.zero),
      throwsA(isA<Exception>()),
    );
    expect(downloader.requests, isEmpty);
    expect(sharePort.files, isEmpty);
  });
}
