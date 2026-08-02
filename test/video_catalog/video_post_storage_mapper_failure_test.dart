import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/video_post_storage_mapper.dart';

void main() {
  test('rejects an invalid persisted video-post payload', () {
    expect(
      () => const VideoPostStorageMapper().fromMap(const {'id': 12}),
      throwsA(isA<FormatException>()),
    );
  });
}
