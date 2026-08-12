import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/profile/data/local_profile_metadata_cache.dart';
import 'package:ghostr/features/profile/data/nostr_profile_metadata_repository.dart';
import 'package:ghostr/features/profile/domain/profile_metadata.dart';
import 'package:ghostr/features/session/domain/nostr_identity.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/nostr_test_values.dart';
import '../support/rejecting_nostr_event_client.dart';

void main() {
  test(
    'rejected profile publish leaves the previous cache unchanged',
    () async {
      SharedPreferences.setMockInitialValues({});
      final cache = LocalProfileMetadataCache(
        await SharedPreferences.getInstance(),
      );
      final profileId = ProfileId.parse(testViewerNpub);
      await cache.write(
        ProfileSummary(
          id: profileId,
          displayName: 'Before',
          handle: '@before',
          avatarUrl: null,
        ),
      );
      final client = RejectingNostrEventClient(
        publicKeyHex: testViewerPublicKey,
        failure: const AppFailure('No Nostr relay accepted.'),
      );
      final repository = NostrProfileMetadataRepository(
        client: client,
        cache: cache,
      );
      final identity = NostrIdentity.parse(
        publicKeyHex: testViewerPublicKey,
        npub: testViewerNpub,
      );

      await expectLater(
        repository.save(
          identity,
          ProfileMetadata.parse(displayName: 'After', handle: 'after'),
        ),
        throwsA(isA<AppFailure>()),
      );

      expect((await cache.read(profileId))?.displayName, 'Before');
    },
  );
}
