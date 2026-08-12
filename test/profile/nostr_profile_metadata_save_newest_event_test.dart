import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/profile/data/local_profile_metadata_cache.dart';
import 'package:ghostr/features/profile/data/nostr_profile_metadata_repository.dart';
import 'package:ghostr/features/profile/domain/profile_metadata.dart';
import 'package:ghostr/features/session/domain/nostr_identity.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/nostr_test_values.dart';
import '../support/profile_metadata_event.dart';
import '../support/scripted_nostr_event_client.dart';

void main() {
  test('save preserves fields from the newest out-of-order metadata', () async {
    SharedPreferences.setMockInitialValues({});
    final client = ScriptedNostrEventClient((_) {
      return [
        profileMetadataEvent(
          '{"about":"Stale biography"}',
          createdAt: 1700000000,
        ),
        profileMetadataEvent(
          '{"about":"Current biography"}',
          createdAt: 1700000100,
          id: secondTestEventId,
        ),
      ];
    });
    final repository = NostrProfileMetadataRepository(
      client: client,
      cache: LocalProfileMetadataCache(await SharedPreferences.getInstance()),
    );

    await repository.save(
      NostrIdentity.parse(
        publicKeyHex: testViewerPublicKey,
        npub: testViewerNpub,
      ),
      ProfileMetadata.parse(displayName: 'Nora', handle: 'nora'),
    );

    final published = jsonDecode(client.events.single.content);
    expect(published, containsPair('about', 'Current biography'));
  });
}
