import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/publish/data/nostr_video_publisher.dart';
import 'package:ghostr/features/publish/domain/uploaded_video_media.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/fake_video_media_upload_port.dart';
import '../support/sample_data.dart';
import '../support/nostr_test_values.dart';

void main() {
  test('publishing a caption with hashtags emits t tags on the event',
      () async {
    final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey);
    final uploader = FakeVideoMediaUploadPort(UploadedVideoMedia(
      source: VideoMediaSource.remote('https://media.example/video.mp4'),
      metadata: VideoUploadMetadata(
        sha256:
            'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
        mimeType: 'video/mp4',
        sizeBytes: 4096,
      ),
    ));
    final publisher = NostrVideoPublisher(
      eventClient: client,
      mediaUploader: uploader,
      clock: () => DateTime.utc(2026, 8, 2, 12, 30),
    );

    final post = await publisher.publish(
      session: sampleSession(),
      media: sampleMedia(),
      caption: 'Relay moves #Dance #Beats',
    );

    expect(
        client.events.single.tags,
        containsAll(<List<String>>[
          ['t', 'dance'],
          ['t', 'beats'],
        ]));
    expect(post.hashtags, ['dance', 'beats']);
  });
}
