import 'package:flutter_test/flutter_test.dart';

import '../support/fakes.dart';
import '../support/hybrid_repository_harness.dart';
import '../support/sample_data.dart';

void main() {
  test('excludes blocked creators from search results', () async {
    final blocked = sampleCreator(id: 'npub1blocked');
    final visible = sampleCreator(id: 'npub1visible');
    final harness = await buildHybridRepositoryHarness(
      FakeRemoteVideoSource([
        samplePost(caption: 'relay clip', creator: blocked),
        samplePost(id: 'visible', caption: 'relay clip', creator: visible),
      ]),
      ports: HybridHarnessPorts(
        social: FakeNostrSocialPort(blockedProfiles: {blocked.id}),
      ),
    );

    final results = await harness.search.search('relay');

    expect(results.map((post) => post.creator.id), [visible.id]);
  });
}
