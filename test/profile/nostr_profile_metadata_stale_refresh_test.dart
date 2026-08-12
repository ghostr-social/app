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
  test('stale relay metadata cannot replace a newer cached profile', () async {
    SharedPreferences.setMockInitialValues({});
    final cache = LocalProfileMetadataCache(
      await SharedPreferences.getInstance(),
    );
    final profileId = ProfileId.parse(testViewerNpub);
    final current = ProfileSummary(
      id: profileId,
      displayName: 'Current Nora',
      handle: '@current_nora',
      avatarUrl: null,
    );
    await cache.write(current, observedAt: 1700000100);
    final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey)
      ..events.add(
        profileMetadataEvent('{"display_name":"Old Nora","name":"old_nora"}'),
      );

    final refreshed = await NostrProfileMetadataRepository(
      client: client,
      cache: cache,
    ).refresh(profileId);

    expect(refreshed?.displayName, 'Current Nora');
    expect((await cache.read(profileId))?.displayName, 'Current Nora');
  });
}
