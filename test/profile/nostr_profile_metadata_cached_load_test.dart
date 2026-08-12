import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/profile/data/local_profile_metadata_cache.dart';
import 'package:ghostr/features/profile/data/nostr_profile_metadata_repository.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_test_values.dart';

void main() {
  test('cached profile metadata loads without a relay query', () async {
    SharedPreferences.setMockInitialValues({});
    final preferences = await SharedPreferences.getInstance();
    final cache = LocalProfileMetadataCache(preferences);
    final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey);
    final profile = ProfileSummary(
      id: ProfileId.parse(testViewerNpub),
      displayName: 'Nora Relay',
      handle: '@nora',
      avatarUrl: null,
    );
    await cache.write(profile);
    final repository = NostrProfileMetadataRepository(
      client: client,
      cache: cache,
    );

    final restored = await repository.loadCached(profile.id);

    expect(restored?.displayName, 'Nora Relay');
    expect(client.queries, isEmpty);
  });
}
