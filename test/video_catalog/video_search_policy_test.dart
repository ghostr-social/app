import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/video_search_policy.dart';

import '../support/sample_data.dart';

void main() {
  test('matches normalized fields while excluding blocked creators', () {
    final visible = sampleCreator(id: 'visible', displayName: 'Nora Relay');
    final blocked = sampleCreator(id: 'blocked', displayName: 'Nora Blocked');

    final posts = const VideoSearchPolicy().select(
      [samplePost(creator: visible), samplePost(creator: blocked)],
      query: '  NORA ',
      blocked: {blocked.id},
    );

    expect(posts.map((post) => post.creator.id), [visible.id]);
  });
}
