import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/ffi_video_remote_source.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

void main() {
  test('the native index declines older-page requests', () async {
    final source = FfiVideoRemoteSource(
      snapshotLoader: () => const <VideoPost>[],
      loader: () async => throw StateError('must not consult the gateway'),
    );

    final posts = await source.loadRemoteFeed(
      olderThan: DateTime.utc(2026, 8, 1),
    );

    expect(posts, isEmpty);
  });
}
