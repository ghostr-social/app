import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_updates.dart';
import 'package:ghostr/features/video_catalog/domain/video_search_updates.dart';

import '../support/discovery_search_harness.dart';
import '../support/sample_data.dart';

void main() {
  test('live search preserves every native phase and filters its posts',
      () async {
    final blocked = sampleCreator(id: 'npub-blocked');
    final harness = DiscoverySearchHarness(posts: [
      samplePost(id: 'visible', hashtags: const ['dance']),
      samplePost(
        id: 'hidden',
        creator: blocked,
        hashtags: const ['dance'],
      ),
    ]);
    harness.social.blocked.add(blocked.id);
    const phases = {
      RemoteVideoPhase.loading: VideoSearchPhase.loading,
      RemoteVideoPhase.settled: VideoSearchPhase.settled,
      RemoteVideoPhase.failed: VideoSearchPhase.failed,
    };

    for (final phase in phases.entries) {
      harness.source.snapshotPhase = phase.key;
      final snapshot = await harness.repository.watchVideos(' #Dance ').first;

      expect(snapshot.revision, BigInt.one);
      expect(snapshot.phase, phase.value);
      expect(snapshot.page.posts.single.id.value, 'visible');
    }
    expect(harness.source.watchQueries, [null, null, null]);
    expect(harness.source.watchHashtags, [
      {'dance'},
      {'dance'},
      {'dance'},
    ]);
  });
}
