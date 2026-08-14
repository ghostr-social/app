import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/video_post_storage_mapper.dart';

import '../support/repost_samples.dart';

void main() {
  test('persists the exact published addressable identifier', () {
    const mapper = VideoPostStorageMapper();
    final post = repostablePublishedPost(
      kind: 34235,
      identifier: 'clip',
      publishedIdentifier: ' clip ',
    );

    final decoded = mapper.fromMap(mapper.toMap(post));

    expect(decoded.nostrReference?.identifier?.value, 'clip');
    expect(decoded.nostrReference?.coordinateIdentifier?.value, ' clip ');
  });
}
