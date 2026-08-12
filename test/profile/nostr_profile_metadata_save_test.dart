import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/profile/data/local_profile_metadata_cache.dart';
import 'package:ghostr/features/profile/data/nostr_profile_metadata_repository.dart';
import 'package:ghostr/features/profile/domain/profile_metadata.dart';
import 'package:ghostr/features/session/domain/nostr_identity.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_test_values.dart';
import '../support/profile_metadata_event.dart';

void main() {
  test(
    'save preserves unrelated fields, publishes, then caches profile',
    () async {
      SharedPreferences.setMockInitialValues({});
      final cache = LocalProfileMetadataCache(
        await SharedPreferences.getInstance(),
      );
      final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey)
        ..events.add(
          profileMetadataEvent(
            '{"display_name":"Old","name":"old",'
            '"about":"Still here","nip05":"nora@example.com"}',
          ),
        );
      final repository = NostrProfileMetadataRepository(
        client: client,
        cache: cache,
      );
      final identity = NostrIdentity.parse(
        publicKeyHex: testViewerPublicKey,
        npub: testViewerNpub,
      );
      final metadata = ProfileMetadata.parse(
        displayName: 'Nora Relay',
        handle: 'nora',
        pictureUrl: 'https://cdn.example/nora.png',
      );

      await repository.save(identity, metadata);

      final payload = jsonDecode(client.events.last.content);
      expect(payload, containsPair('about', 'Still here'));
      expect(payload, containsPair('nip05', 'nora@example.com'));
      expect(client.events.last.kind.value, 0);
      expect(client.publishedAuthors.single.value, testViewerPublicKey);
      final cached = await cache.read(ProfileId.parse(testViewerNpub));
      expect(cached?.displayName, 'Nora Relay');
      expect(cached?.handle, '@nora');
    },
  );
}
