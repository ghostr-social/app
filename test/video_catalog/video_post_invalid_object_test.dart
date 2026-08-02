import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/video_post_storage_mapper.dart';

import '../support/sample_data.dart';

void main() {
  test('rejects a persisted post whose creator is not an object', () {
    final mapper = const VideoPostStorageMapper();
    final map = mapper.toMap(samplePost())..['creator'] = 'invalid';

    expect(() => mapper.fromMap(map), throwsFormatException);
  });
}
