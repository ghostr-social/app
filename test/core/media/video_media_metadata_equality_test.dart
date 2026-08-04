import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_metadata.dart';

void main() {
  test('metadata values compare by size and duration', () {
    const a = VideoMediaMetadata(sizeBytes: 1, durationMs: 2);
    const b = VideoMediaMetadata(sizeBytes: 1, durationMs: 2);

    expect(a, b);
    expect(a.hashCode, b.hashCode);
    expect(a, isNot(const VideoMediaMetadata(sizeBytes: 1)));
    expect(a, isNot(const VideoMediaMetadata(durationMs: 2)));
    expect(VideoMediaMetadata.none, const VideoMediaMetadata());
  });
}
