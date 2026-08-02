import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';

void main() {
  test('rejects an empty local video path', () {
    expect(() => VideoMediaSource.local('  '), throwsFormatException);
  });
}
