import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/video_catalog/data/ffi_video_remote_source.dart';

void main() {
  test('translates a native inventory loader failure', () async {
    final source = FfiVideoRemoteSource(
      snapshotLoader: () => const [],
      loader: () => throw StateError('native bridge unavailable'),
    );

    await expectLater(
      source.loadRemoteFeed(),
      throwsA(isA<AppFailure>()),
    );
  });
}
