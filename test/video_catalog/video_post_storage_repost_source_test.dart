import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/video_post_storage_mapper.dart';

import '../support/repost_samples.dart';

void main() {
  test('persists the signed source needed to repost a cached video', () {
    const mapper = VideoPostStorageMapper();
    final post = repostablePost();

    final decoded = mapper.fromMap(mapper.toMap(post));

    expect(
      decoded.nostrReference?.signedEvent,
      post.nostrReference?.signedEvent,
    );
    expect(decoded.nostrReference?.isProtected, isFalse);
  });
}
