import 'package:flutter_test/flutter_test.dart';
import '../support/fakes.dart';
import '../support/hybrid_repository_harness.dart';
import '../support/sample_data.dart';

void main() {
  test('sends the normalized search query to Nostr relays', () async {
    final remote = FakeRemoteVideoSource([samplePost()]);
    final harness = await buildHybridRepositoryHarness(remote);

    await harness.search.search('  ReLaY  ');

    expect(remote.requestedSearchQuery, 'relay');
  });
}
