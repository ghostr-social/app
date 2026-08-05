import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_metadata.dart';

void main() {
  test('imeta parsing keeps positive finite size and duration values', () {
    expect(
      VideoMediaMetadata.fromImeta(size: ' 2048 ', duration: '1.2345'),
      const VideoMediaMetadata(sizeBytes: 2048, durationMs: 1235),
    );
    for (final raw in [null, '', 'bad', '0', '-1', 'NaN', 'Infinity']) {
      expect(
        VideoMediaMetadata.fromImeta(size: raw, duration: raw),
        VideoMediaMetadata.none,
        reason: '$raw',
      );
    }
  });
}
