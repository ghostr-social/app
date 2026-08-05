import 'package:flutter_test/flutter_test.dart';

import '../support/discovery_search_harness.dart';
import '../support/sample_data.dart';

void main() {
  test('a hashtag query becomes a tag filter revalidated on results', () async {
    final harness = DiscoverySearchHarness(posts: [
      samplePost(id: 'tagged', hashtags: const ['dance', 'music']),
      samplePost(id: 'untagged', hashtags: const ['other']),
    ]);

    final page = await harness.repository.searchVideos(' #Dance ');

    expect(harness.source.searchQueries, [null]);
    expect(harness.source.hashtags, [
      {'dance'},
    ]);
    expect(page.posts.map((post) => post.id.value), ['tagged']);
    expect(page.hasMore, isTrue);
  });
}
