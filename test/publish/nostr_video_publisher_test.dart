import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/publish/data/nostr_video_publisher.dart';
import 'package:ghostr/features/publish/domain/uploaded_video_media.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/fake_video_media_upload_port.dart';
import '../support/sample_data.dart';
import '../support/nostr_test_values.dart';

void main() {
  test('uploads media then publishes a referenced NIP-71 short video', () async {
    final publishedAt = DateTime.utc(2026, 8, 2, 12, 30);
    final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey);
    final uploader = FakeVideoMediaUploadPort(
      UploadedVideoMedia(
        source: VideoMediaSource.remote(
          'https://media.example/video.mp4',
          fallbackUrls: ['https://mirror.example/video.mp4'],
        ),
        metadata: VideoUploadMetadata(
          sha256:
              'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
          mimeType: 'video/mp4',
          sizeBytes: 4096,
        ),
      ),
    );
    final publisher = NostrVideoPublisher(
      eventClient: client,
      mediaUploader: uploader,
      clock: () => publishedAt,
    );

    final post = await publisher.publish(
      session: sampleSession(),
      media: sampleMedia(),
      caption: 'Relay dance',
    );

    expect(uploader.uploadedMedia?.path, sampleMedia().path);
    expect(client.events.single.kind, 22);
    expect(client.events.single.content, 'Relay dance');
    expect(
      client.events.single.tags,
      containsAll(<List<String>>[
        ['title', 'Relay dance'],
        ['alt', 'Relay dance'],
      ]),
    );
    expect(
      client.events.single.tags.singleWhere((tag) => tag.first == 'imeta'),
      containsAll(<String>[
        'url https://media.example/video.mp4',
        'fallback https://mirror.example/video.mp4',
        'm video/mp4',
        'x aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
        'size 4096',
      ]),
    );
    expect(post.id, publishedEventId(1));
    expect(post.media.remoteUrl, 'https://media.example/video.mp4');
    expect(post.media.remoteUrls, [
      'https://media.example/video.mp4',
      'https://mirror.example/video.mp4',
    ]);
    expect(
      post.media.expectedSha256?.value,
      'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
    );
    expect(post.media.cacheScope?.value, publishedEventId(1));
    expect(post.nostrReference?.kind, 22);
    expect(post.nostrReference?.signedEvent, isNotNull);
    expect(post.publishedAt, publishedAt);
    expect(
      client.events.single.tags.singleWhere(
        (tag) => tag.first == 'published_at',
      ),
      <String>['published_at', '1785673800'],
    );
  });
}
