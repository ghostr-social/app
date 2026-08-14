import 'package:flutter_test/flutter_test.dart';

import '../support/repost_samples.dart';

void main() {
  test('repost activity time does not replace original publication time', () {
    final post = repostedPost();

    expect(post.publishedAt, DateTime.utc(2026, 1, 1));
    expect(post.feedActivityAt, DateTime.utc(2026, 2, 1));
    expect(post.creator.displayName, 'Nora Relay');
    expect(post.repost?.reposter.displayName, 'Bob Relay');
  });
}
