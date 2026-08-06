import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_sharing/data/default_video_share_workflow.dart';
import 'package:ghostr/features/video_sharing/domain/shareable_video_file.dart';
import 'package:ghostr/features/video_sharing/domain/video_share_origin.dart';

import '../support/fake_video_sharing.dart';

void main() {
  test('shares a newly published source file without downloading it', () async {
    final downloader = FakeVideoFileDownloader(
      ShareableVideoFile.parse('/tmp/unexpected.mp4'),
    );
    final sharePort = FakeVideoFileSharePort();
    final workflow = DefaultVideoShareWorkflow(
      downloader: downloader,
      sharePort: sharePort,
    );
    final media = VideoMediaSource.importable(
      '/videos/published.mp4',
      remoteUrl: 'https://media.test/published.mp4',
    );

    await workflow.share(media, origin: VideoShareOrigin.zero);

    expect(downloader.requests, isEmpty);
    expect(sharePort.files.single.path, '/videos/published.mp4');
  });
}
