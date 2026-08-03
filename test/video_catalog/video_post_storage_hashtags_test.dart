import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/video_post_storage_mapper.dart';

import '../support/sample_data.dart';

void main() {
  test('round trips hashtags and defaults a missing key to an empty list', () {
    const mapper = VideoPostStorageMapper();
    final post = samplePost(hashtags: ['nostr', 'dance']);

    final decoded = mapper.fromMap(mapper.toMap(post));
    expect(decoded.hashtags, ['nostr', 'dance']);

    final withoutHashtags = mapper.toMap(post)..remove('hashtags');
    expect(mapper.fromMap(withoutHashtags).hashtags, isEmpty);
  });
}
