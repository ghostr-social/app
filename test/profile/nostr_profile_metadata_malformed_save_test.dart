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
  test('save repairs malformed prior metadata with a fresh object', () async {
    SharedPreferences.setMockInitialValues({});
    final client = ScriptedNostrEventClient((_) {
      return [profileMetadataEvent('{broken')];
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

    expect(jsonDecode(client.events.single.content), {
      'display_name': 'Nora',
      'name': 'nora',
    });
  });
}
