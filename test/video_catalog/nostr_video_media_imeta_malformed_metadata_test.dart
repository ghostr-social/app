import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_metadata.dart';
import 'package:ghostr/features/video_catalog/data/nostr_video_media.dart';

void main() {
  NostrVideoMedia? parse(List<String> imetaFields) {
    return NostrVideoMedia.fromEvent(
      tags: [
        ['imeta', 'url https://cdn.example/clip.mp4', ...imetaFields],
      ],
      content: '',
    );
  }

  test('keeps the media and drops non-numeric size and duration', () {
    final media = parse(const ['size twelve', 'duration abc']);

    expect(media?.urls, ['https://cdn.example/clip.mp4']);
    expect(media?.metadata, VideoMediaMetadata.none);
  });

  test('drops negative and zero values', () {
    final media = parse(const ['size -1', 'duration 0']);

    expect(media?.metadata, VideoMediaMetadata.none);
  });

  test('drops non-finite durations and fractional byte sizes', () {
    final media = parse(const ['size 12.5', 'duration Infinity']);

    expect(media?.metadata, VideoMediaMetadata.none);
  });

  test('drops empty values while keeping well-formed siblings', () {
    final media = parse(const ['size ', 'duration 7']);

    expect(media?.metadata.sizeBytes, isNull);
    expect(media?.metadata.durationMs, 7000);
  });
}
