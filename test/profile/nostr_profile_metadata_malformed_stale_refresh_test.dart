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
  test(
    'newer cached metadata shields against a malformed stale event',
    () async {
      SharedPreferences.setMockInitialValues({});
      final cache = LocalProfileMetadataCache(
        await SharedPreferences.getInstance(),
      );
      final profileId = ProfileId.parse(testViewerNpub);
      await cache.write(
        ProfileSummary(
          id: profileId,
          displayName: 'Current Nora',
          handle: '@current',
          avatarUrl: null,
        ),
        observedAt: 1700000100,
      );
      final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey)
        ..events.add(profileMetadataEvent('{broken'));

      final refreshed = await NostrProfileMetadataRepository(
        client: client,
        cache: cache,
      ).refresh(profileId);

      expect(refreshed?.displayName, 'Current Nora');
    },
  );
}
