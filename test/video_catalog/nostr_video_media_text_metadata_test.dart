import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_metadata.dart';
import 'package:ghostr/features/video_catalog/data/nostr_video_media.dart';

void main() {
  test('text-fallback media still resolves and carries empty metadata', () {
    final media = NostrVideoMedia.fromEvent(
      tags: const [],
      content: 'watch this https://cdn.example/clip.mp4',
    );

    expect(media?.urls, ['https://cdn.example/clip.mp4']);
    expect(media?.metadata, VideoMediaMetadata.none);
  });

  test('file-tag media still resolves and carries empty metadata', () {
    final media = NostrVideoMedia.fromEvent(
      tags: const [
        ['url', 'https://cdn.example/clip.mp4'],
        ['m', 'video/mp4'],
        ['size', '123456'],
      ],
      content: '',
    );

    expect(media?.urls, ['https://cdn.example/clip.mp4']);
    expect(media?.metadata, VideoMediaMetadata.none);
  });
}
