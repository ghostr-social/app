import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_metadata.dart';
import 'package:ghostr/features/video_catalog/data/nostr_video_media.dart';

void main() {
  test('parses imeta size and duration into media metadata', () {
    final media = NostrVideoMedia.fromEvent(
      tags: const [
        [
          'imeta',
          'url https://cdn.example/clip.mp4',
          'm video/mp4',
          'size 123456',
          'duration 42',
        ],
      ],
      content: '',
    );

    expect(media?.metadata.sizeBytes, 123456);
    expect(media?.metadata.durationMs, 42000);
  });

  test('parses fractional imeta durations to whole milliseconds', () {
    final media = NostrVideoMedia.fromEvent(
      tags: const [
        ['imeta', 'url https://cdn.example/clip.mp4', 'duration 12.5'],
      ],
      content: '',
    );

    expect(media?.metadata.durationMs, 12500);
  });

  test('leaves metadata empty when imeta omits size and duration', () {
    final media = NostrVideoMedia.fromEvent(
      tags: const [
        ['imeta', 'url https://cdn.example/clip.mp4', 'm video/mp4'],
      ],
      content: '',
    );

    expect(media?.urls, ['https://cdn.example/clip.mp4']);
    expect(media?.metadata, VideoMediaMetadata.none);
  });
}
