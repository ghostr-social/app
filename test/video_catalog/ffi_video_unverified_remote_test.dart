import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/ffi_video_remote_source.dart';
import 'package:ghostr/src/rust/video/video.dart';

import '../support/ffi_video_fixture.dart';

void main() {
  test('skips unmatched progressive native media without verified local bytes',
      () async {
    final source = FfiVideoRemoteSource(
      snapshotLoader: () => const [],
      loader: () async => [
        ffiVideo(id: 'rejected', user: const FfiUserData()),
      ],
    );

    expect(await source.loadRemoteFeed(), isEmpty);
  });
}
