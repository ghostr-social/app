import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/publish/data/nostr_video_publisher.dart';
import 'package:ghostr/features/publish/domain/uploaded_video_media.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/fake_video_media_upload_port.dart';
import '../support/nostr_test_values.dart';
import '../support/sample_data.dart';

void main() {
  test('publishes all mirrors while bounding automatic cache candidates',
      () async {
    final fallbacks = List.generate(
      6,
      (index) => 'https://mirror$index.example/video.mp4',
    );
    final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey);
    final uploader = FakeVideoMediaUploadPort(UploadedVideoMedia(
      source: VideoMediaSource.remote(
        'https://media.example/video.mp4',
        fallbackUrls: fallbacks,
      ),
      metadata: VideoUploadMetadata(
        sha256:
            'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
        mimeType: 'video/mp4',
        sizeBytes: 10,
      ),
    ));
    final publisher = NostrVideoPublisher(
      eventClient: client,
      mediaUploader: uploader,
    );

    final post = await publisher.publish(
      session: sampleSession(),
      media: sampleMedia(),
      caption: 'Many mirrors',
    );

    expect(post.media.remoteUrls, hasLength(7));
    expect(post.media.cacheSourceUrls, post.media.remoteUrls.take(5));
    final imeta = client.events.single.tags.singleWhere(
      (tag) => tag.first == 'imeta',
    );
    expect(imeta.where((field) => field.startsWith('fallback ')), hasLength(6));
  });
}
