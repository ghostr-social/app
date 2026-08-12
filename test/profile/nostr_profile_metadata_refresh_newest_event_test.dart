import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/profile/data/local_profile_metadata_cache.dart';
import 'package:ghostr/features/profile/data/nostr_profile_metadata_repository.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/nostr_test_values.dart';
import '../support/profile_metadata_event.dart';
import '../support/scripted_nostr_event_client.dart';

void main() {
  test(
    'refresh keeps the newest metadata when relays answer out of order',
    () async {
      SharedPreferences.setMockInitialValues({});
      final client = ScriptedNostrEventClient((_) {
        return [
          profileMetadataEvent(
            '{"display_name":"Stale Nora","name":"stale"}',
            createdAt: 1700000000,
          ),
          profileMetadataEvent(
            '{"display_name":"Current Nora","name":"current"}',
            createdAt: 1700000100,
            id: secondTestEventId,
          ),
        ];
      });
      final repository = NostrProfileMetadataRepository(
        client: client,
        cache: LocalProfileMetadataCache(await SharedPreferences.getInstance()),
      );

      final refreshed = await repository.refresh(
        ProfileId.parse(testViewerNpub),
      );

      expect(refreshed?.displayName, 'Current Nora');
      expect(refreshed?.handle, '@current');
      expect(client.queries.single.limit, greaterThan(1));
    },
  );
}
