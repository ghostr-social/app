import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/profile/data/local_profile_metadata_cache.dart';
import 'package:ghostr/features/profile/data/nostr_profile_metadata_repository.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_test_values.dart';
import '../support/profile_metadata_event.dart';

void main() {
  test('refresh queries author kind zero and caches parsed metadata', () async {
    SharedPreferences.setMockInitialValues({});
    final cache = LocalProfileMetadataCache(
      await SharedPreferences.getInstance(),
    );
    final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey)
      ..events.add(
        profileMetadataEvent(
          '{"display_name":"Nora Relay","name":"nora",'
          '"picture":"https://cdn.example/nora.png"}',
        ),
      );
    final repository = NostrProfileMetadataRepository(
      client: client,
      cache: cache,
    );
    final profileId = ProfileId.parse(testViewerNpub);

    final refreshed = await repository.refresh(profileId);

    expect(refreshed?.displayName, 'Nora Relay');
    expect(refreshed?.handle, '@nora');
    expect(client.queries, hasLength(1));
    expect(client.queries.single.kinds.map((kind) => kind.value), [0]);
    expect(client.queries.single.authors.single.value, testViewerPublicKey);
    expect(client.queries.single.limit, 20);
    expect(
      (await cache.read(profileId))?.avatarUrl,
      'https://cdn.example/nora.png',
    );
  });
}
