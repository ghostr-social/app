import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';

import '../support/discovery_search_harness.dart';
import '../support/sample_data.dart';

void main() {
  test('creator search filters blocked profiles and tolerates failures',
      () async {
    final alice = sampleCreator(id: 'npub-alice', displayName: 'Alice');
    final mallory = sampleCreator(id: 'npub-mallory', displayName: 'Mallory');
    final harness = DiscoverySearchHarness(creators: [alice, mallory]);
    harness.social.blocked.add(mallory.id);

    final creators = await harness.repository.searchCreators('ali');
    expect(creators.map((creator) => creator.displayName), ['Alice']);
    expect(harness.creators.queries, ['ali']);

    expect(await harness.repository.searchCreators('#dance'), isEmpty);
    expect(harness.creators.queries, hasLength(1));

    harness.creators.failure = const AppFailure('search relay down');
    expect(await harness.repository.searchCreators('ali'), isEmpty);
    expect(harness.reporter.sources, isNotEmpty);
  });
}
