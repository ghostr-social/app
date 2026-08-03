import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_catalog/data/ffi_native_video_post_mapper.dart';
import 'package:ghostr/src/rust/video/video.dart';

import '../support/ffi_video_fixture.dart';
import '../support/nostr_test_values.dart';

void main() {
  test('a native video without an npub still gets an npub creator id', () {
    final video = ffiVideo(
      id: 'a' * 64,
      user: const FfiUserData(),
      options: const FfiVideoFixtureOptions(localPath: '/cache/video.mp4'),
      event: ffiNostrEvent(authorPublicKeyHex: testViewerPublicKey),
    );

    final post = tryMapFfiNativeVideo(
      video,
      (item) => VideoMediaSource.remote(item.url),
    );

    expect(post, isNotNull);
    expect(post!.creator.id.value, testViewerNpub);
  });
}
