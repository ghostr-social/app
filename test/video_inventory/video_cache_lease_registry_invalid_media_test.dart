import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/video_cache_lease_registry.dart';

void main() {
  test('rejects a cache lease without a managed local path', () {
    final registry = VideoCacheLeaseRegistry();
    final remote = VideoMediaSource.remote('https://media.test/video.mp4');

    expect(() => registry.acquire(remote), throwsStateError);
  });
}
