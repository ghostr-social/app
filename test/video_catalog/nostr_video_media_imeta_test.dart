import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_catalog/data/nostr_video_media.dart';

void main() {
  test('accepts an imeta video without a mime when the URL is a video', () {
    final media = NostrVideoMedia.fromEvent(
      tags: const [
        ['imeta', 'url https://cdn.example/clip.mp4'],
      ],
      content: '',
    );

    expect(media?.urls, ['https://cdn.example/clip.mp4']);
    expect(media?.delivery, VideoMediaDelivery.progressive);
  });

  test('prefers imeta media over links written into the note text', () {
    final media = NostrVideoMedia.fromEvent(
      tags: const [
        ['imeta', 'url https://cdn.example/tagged.mp4', 'm video/mp4'],
      ],
      content: 'https://cdn.example/from-text.mp4',
    );

    expect(media?.urls, ['https://cdn.example/tagged.mp4']);
  });

  test('rejects imeta media that is neither video mime nor video URL', () {
    final media = NostrVideoMedia.fromEvent(
      tags: const [
        ['imeta', 'url https://cdn.example/cover.jpg', 'm image/jpeg'],
        ['imeta', 'url https://cdn.example/page.html'],
      ],
      content: '',
    );

    expect(media, isNull);
  });
}
