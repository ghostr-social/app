import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/profile/data/local_profile_metadata_cache.dart';
import 'package:ghostr/features/profile/data/nostr_profile_metadata_repository.dart';
import 'package:ghostr/features/profile/domain/profile_metadata.dart';
import 'package:ghostr/features/session/domain/nostr_identity.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/fake_nostr_event_client.dart';
import '../support/nostr_test_values.dart';

void main() {
  test(
    'save rejects an identity that is not the active Nostr account',
    () async {
      SharedPreferences.setMockInitialValues({});
      final cache = LocalProfileMetadataCache(
        await SharedPreferences.getInstance(),
      );
      final client = FakeNostrEventClient(publicKeyHex: testViewerPublicKey);
      final repository = NostrProfileMetadataRepository(
        client: client,
        cache: cache,
      );
      final otherIdentity = NostrIdentity.parse(
        publicKeyHex: testCreatorPublicKey,
        npub: testCreatorNpub,
      );

      await expectLater(
        repository.save(
          otherIdentity,
          ProfileMetadata.parse(displayName: 'Eli', handle: 'eli'),
        ),
        throwsA(
          isA<AppFailure>().having(
            (failure) => failure.message,
            'message',
            'The active account changed. Try again.',
          ),
        ),
      );

      expect(client.queries, isEmpty);
      expect(client.publishedAuthors, isEmpty);
      expect(await cache.read(ProfileId.parse(testCreatorNpub)), isNull);
    },
  );
}
