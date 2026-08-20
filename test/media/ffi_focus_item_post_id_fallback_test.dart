import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/ffi_focus_item_media_mapper.dart';

void main() {
  final storeSafe = RegExp(r'^[A-Za-z0-9_-]+$');

  test('falls back through digests when the scope is not store-safe', () {
    final unsafeScoped = VideoMediaSource.withCacheScope(
      VideoMediaSource.withExpectedSha256(
        VideoMediaSource.remote('https://media.test/clip.mp4'),
        'b' * 64,
      ),
      'scope:with:colons',
    );

    expect(ffiPostIdForMedia(unsafeScoped), 'b' * 64);
  });

  test('derives a stable store-safe id from the primary URL', () {
    final bare = VideoMediaSource.remote('https://media.test/clip.mp4');
    final again = VideoMediaSource.remote('https://media.test/clip.mp4');
    final other = VideoMediaSource.remote('https://media.test/other.mp4');

    final id = ffiPostIdForMedia(bare);

    expect(
      id,
      'url-9749fdddd453caaca021690db04c6aeaa579386dd6e8fb127cd82c47a3d52f55',
    );
    expect(storeSafe.hasMatch(id), isTrue);
    expect(ffiPostIdForMedia(again), id);
    expect(ffiPostIdForMedia(other), isNot(id));
  });
}
