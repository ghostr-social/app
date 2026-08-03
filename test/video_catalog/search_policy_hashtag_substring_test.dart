import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_search_policy.dart';

import '../support/sample_data.dart';

void main() {
  test('a plain query matches posts through hashtag substrings', () {
    final tagged = samplePost(id: 'tagged', hashtags: const ['dance']);
    final untagged = samplePost(id: 'untagged');

    final posts = const VideoSearchPolicy().select(
      [tagged, untagged],
      query: 'dan',
      blocked: const <ProfileId>{},
    );

    expect(posts.map((post) => post.id), [tagged.id]);
  });
}
