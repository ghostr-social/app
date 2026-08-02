import 'package:flutter_test/flutter_test.dart';
import '../support/fakes.dart';
import '../support/hybrid_repository_harness.dart';
import '../support/sample_data.dart';

void main() {
  test('queries relays for the requested profile instead of the global feed',
      () async {
    final creator = sampleCreator(id: 'npub1creator');
    final remote = FakeRemoteVideoSource([samplePost(creator: creator)]);
    final harness = await buildHybridRepositoryHarness(remote);

    await harness.profile.loadProfile(sampleSession().profile, creator.id);

    expect(remote.requestedCreatorIds, {creator.id});
  });
}
