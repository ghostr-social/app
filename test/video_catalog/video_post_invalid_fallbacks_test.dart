import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/video_post_storage_mapper.dart';

import '../support/sample_data.dart';

void main() {
  test('rejects persisted media fallbacks that are not strings', () {
    final mapper = const VideoPostStorageMapper();
    final map = mapper.toMap(samplePost());
    final media = map['media']! as Map<String, Object?>;
    media['fallbackUrls'] = [1];

    expect(() => mapper.fromMap(map), throwsFormatException);
  });
}
