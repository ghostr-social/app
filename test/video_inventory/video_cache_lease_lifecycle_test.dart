import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/video_cache_lease.dart';

void main() {
  test('a video cache lease releases only after its retained handles', () {
    expect(
      () => VideoCacheLease(
        VideoMediaSource.remote('https://media.test/video.mp4'),
        () {},
      ),
      throwsArgumentError,
    );
    var releases = 0;
    final lease = VideoCacheLease(
      VideoMediaSource.local('/cache/video.mp4'),
      () => releases += 1,
    );
    final retained = lease.retain();

    lease.release();
    lease.release();
    expect(releases, 0);
    retained.release();
    retained.release();

    expect(releases, 1);
    expect(retained.retain, throwsStateError);
  });
}
