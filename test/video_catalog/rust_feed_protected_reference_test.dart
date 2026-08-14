import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_post_mapper.dart';

import '../support/rust_protected_feed_fixture.dart';

void main() {
  test('maps a protected Rust row without requiring signed source JSON', () {
    final row = protectedRustFeedPost();

    final post = const RustFeedPostMapper().map(row);

    expect(post.nostrReference?.isProtected, isTrue);
    expect(post.nostrReference?.signedEvent, isNull);
  });
}
