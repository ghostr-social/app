import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_catalog/data/nostr_video_media.dart';

void main() {
  test('reads NIP-94 file metadata from top-level url/m/x tags', () {
    final media = NostrVideoMedia.fromEvent(
      tags: const [
        ['url', 'https://cdn.example/upload.mp4'],
        ['m', 'video/mp4'],
        [
          'x',
          'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
        ],
      ],
      content: 'a fresh upload',
    );

    expect(media?.urls, ['https://cdn.example/upload.mp4']);
    expect(media?.delivery, VideoMediaDelivery.progressive);
    expect(
      media?.expectedSha256,
      'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
    );
  });

  test('rejects file metadata with a malformed digest or non-video mime', () {
    final badDigest = NostrVideoMedia.fromEvent(
      tags: const [
        ['url', 'https://cdn.example/upload.mp4'],
        ['m', 'video/mp4'],
        ['x', 'not-a-digest'],
      ],
      content: '',
    );
    final image = NostrVideoMedia.fromEvent(
      tags: const [
        ['url', 'https://cdn.example/photo.jpg'],
        ['m', 'image/jpeg'],
      ],
      content: '',
    );

    expect(badDigest, isNull);
    expect(image, isNull);
  });
}
