import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/profile/data/local_profile_metadata_cache.dart';
import 'package:ghostr/features/profile/data/nostr_profile_metadata_repository.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/nostr_test_values.dart';
import '../support/profile_metadata_event.dart';
import '../support/scripted_nostr_event_client.dart';

void main() {
  test('equal-time metadata uses the lexically lowest event ID', () async {
    SharedPreferences.setMockInitialValues({});
    final client = ScriptedNostrEventClient((_) {
      return [
        profileMetadataEvent(
          '{"display_name":"Higher ID"}',
          id: secondTestEventId,
        ),
        profileMetadataEvent('{"display_name":"Lower ID"}'),
      ];
    });
    final repository = NostrProfileMetadataRepository(
      client: client,
      cache: LocalProfileMetadataCache(await SharedPreferences.getInstance()),
    );

    final refreshed = await repository.refresh(ProfileId.parse(testViewerNpub));

    expect(refreshed?.displayName, 'Lower ID');
  });
}
