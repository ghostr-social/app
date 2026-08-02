import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/video_post_storage_mapper.dart';

import '../support/sample_data.dart';

void main() {
  test('rejects a persisted optional field with the wrong type', () {
    final mapper = const VideoPostStorageMapper();
    final map = mapper.toMap(samplePost());
    final creator = map['creator']! as Map<String, Object?>;
    creator['avatarUrl'] = 42;

    expect(() => mapper.fromMap(map), throwsFormatException);
  });
}
