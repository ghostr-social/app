import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/video_post_storage_mapper.dart';

import '../support/repost_samples.dart';

void main() {
  test('persists protection without storing signed event content', () {
    const mapper = VideoPostStorageMapper();
    final post = repostablePost(protected: true);

    final stored = mapper.toMap(post);
    final decoded = mapper.fromMap(stored);

    expect(decoded.nostrReference?.isProtected, isTrue);
    final reference = stored['nostrReference']! as Map<String, Object?>;
    expect(reference['signedEvent'], isNull);
  });
}
