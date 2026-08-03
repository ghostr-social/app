import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_catalog/data/nostr_video_media.dart';

void main() {
  test('extracts the first direct video link written into a note', () {
    final media = NostrVideoMedia.fromEvent(
      tags: const [],
      content: 'new clip! https://cdn.example/dance.mp4 #dance',
    );

    expect(media?.urls, ['https://cdn.example/dance.mp4']);
    expect(media?.delivery, VideoMediaDelivery.progressive);
    expect(media?.expectedSha256, isNull);
  });

  test('recognizes streaming playlists and trims trailing punctuation', () {
    final media = NostrVideoMedia.fromEvent(
      tags: const [],
      content: 'live set: https://cdn.example/set.m3u8, enjoy!',
    );

    expect(media?.urls, ['https://cdn.example/set.m3u8']);
    expect(media?.delivery, VideoMediaDelivery.hls);
  });

  test('keeps query strings while matching on the URL path', () {
    final media = NostrVideoMedia.fromEvent(
      tags: const [],
      content: 'https://cdn.example/clip.MP4?token=abc',
    );

    expect(media?.urls, ['https://cdn.example/clip.MP4?token=abc']);
  });

  test('ignores notes whose links are not videos', () {
    final media = NostrVideoMedia.fromEvent(
      tags: const [],
      content: 'read this https://blog.example/post and https://x.example/a.jpg',
    );

    expect(media, isNull);
  });
}
