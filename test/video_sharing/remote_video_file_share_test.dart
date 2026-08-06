import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_sharing/data/default_video_share_workflow.dart';
import 'package:ghostr/features/video_sharing/domain/shareable_video_file.dart';
import 'package:ghostr/features/video_sharing/domain/video_share_origin.dart';

import '../support/fake_video_sharing.dart';
import '../support/sample_data.dart';

void main() {
  test('shares the downloaded file without sharing the remote URL', () async {
    final file = ShareableVideoFile.parse('/tmp/downloaded-video.mp4');
    final downloader = FakeVideoFileDownloader(file);
    final sharePort = FakeVideoFileSharePort();
    final workflow = DefaultVideoShareWorkflow(
      downloader: downloader,
      sharePort: sharePort,
    );
    final media = samplePost().media;
    const origin = VideoShareOrigin(left: 1, top: 2, width: 3, height: 4);

    await workflow.share(media, origin: origin);

    expect(downloader.requests, [media]);
    expect(sharePort.files, [file]);
    expect(sharePort.files.single.path, isNot(media.remoteUrl));
    expect(sharePort.origins, [origin]);
  });
}
