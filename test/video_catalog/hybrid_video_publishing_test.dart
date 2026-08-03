import 'package:flutter_test/flutter_test.dart';

import '../support/fakes.dart';
import '../support/hybrid_repository_harness.dart';
import '../support/sample_data.dart';

void main() {
  test('publishes a Nostr video and persists it in the local catalog',
      () async {
    final harness = await buildHybridRepositoryHarness(
      FakeRemoteVideoSource([]),
    );

    final published = await harness.publishing.publish(
      session: sampleSession(),
      media: sampleMedia(),
      caption: 'Persisted Nostr clip',
    );

    expect(published.post.caption, 'Persisted Nostr clip');
    final stored = (await harness.localStore.loadPublishedPosts()).single;
    expect(stored.id, published.post.id);
    expect(stored.caption, published.post.caption);
  });
}
