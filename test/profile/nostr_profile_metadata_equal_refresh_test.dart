import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/profile/data/local_profile_metadata_cache.dart';
import 'package:ghostr/features/profile/data/nostr_profile_metadata_repository.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_test_values.dart';
import '../support/profile_metadata_event.dart';

void main() {
  test('equal-second relay metadata cannot replace a local edit', () async {
    SharedPreferences.setMockInitialValues({});
    final cache = LocalProfileMetadataCache(
      await SharedPreferences.getInstance(),
    );
    final profileId = ProfileId.parse(testViewerNpub);
    await cache.write(
      ProfileSummary(
        id: profileId,
        displayName: 'Locally Edited Nora',
        handle: '@local',
        avatarUrl: null,
      ),
      observedAt: 1700000000,
    );
    final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey)
      ..events.add(
        profileMetadataEvent('{"display_name":"Relay Nora","name":"relay"}'),
      );
    final repository = NostrProfileMetadataRepository(
      client: client,
      cache: cache,
    );

    final refreshed = await repository.refresh(profileId);

    expect(refreshed?.displayName, 'Locally Edited Nora');
    expect((await cache.read(profileId))?.handle, '@local');
  });
}
