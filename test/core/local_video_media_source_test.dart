import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';

void main() {
  test('reports a normalized local video location', () {
    final source = VideoMediaSource.local(' /tmp/video.mp4 ');

    expect(source.isLocal, isTrue);
    expect(source.canCacheAsSingleFile, isFalse);
    expect(source.remoteDelivery, isNull);
    expect(source.localPath, '/tmp/video.mp4');
    expect(source.remoteUrl, isNull);
    expect(source.remoteUrls, isEmpty);
    expect(source.fallbackUrls, isEmpty);
    expect(source.debugLabel, '/tmp/video.mp4');
  });
}
