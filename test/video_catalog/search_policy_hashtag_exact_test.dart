import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_search_policy.dart';

import '../support/sample_data.dart';

void main() {
  test('a hashtag query selects only posts tagged with that hashtag', () {
    final tagged = samplePost(
      id: 'tagged',
      caption: 'Footwork drill',
      hashtags: const ['dance'],
    );
    final captionOnly = samplePost(
      id: 'caption-only',
      caption: 'I love to dance daily',
    );

    final posts = VideoSearchPolicy().select(
      [tagged, captionOnly],
      query: '#dance',
      blocked: const <ProfileId>{},
    );

    expect(posts.map((post) => post.id), [tagged.id]);
  });
}
