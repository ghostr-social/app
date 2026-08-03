import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/storage/account_storage_scope.dart';
import 'package:ghostr/features/video_catalog/data/video_post_storage_mapper.dart';
import 'package:ghostr/features/video_catalog/data/local_video_store.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/nostr_test_values.dart';
import '../support/sample_data.dart';

void main() {
  test('isolates published and social data when the active account changes',
      () async {
    final legacyPost = samplePost(id: 'legacy-post');
    SharedPreferences.setMockInitialValues({
      'ghostr.catalog.published': jsonEncode([
        const VideoPostStorageMapper().toMap(legacyPost),
      ]),
      'ghostr.catalog.followed': ['legacy-follow'],
      'ghostr.catalog.blocked': ['legacy-block'],
    });
    var account = NostrPublicKeyHex.parse(testViewerPublicKey);
    final store = LocalVideoStore(
      await SharedPreferences.getInstance(),
      accountScope: AccountStorageScope(() => account),
    );

    expect(await store.loadPublishedPosts(), isEmpty);
    expect(await store.loadFollowedProfiles(), isEmpty);
    expect(await store.loadBlockedProfiles(), isEmpty);
    final firstPost = samplePost(id: 'first-account-post');
    final firstFollow = ProfileId.parse('first-follow');
    final firstBlock = ProfileId.parse('first-block');
    await store.savePublishedPosts([firstPost]);
    await store.saveFollowedProfiles({firstFollow});
    await store.saveBlockedProfiles({firstBlock});

    account = NostrPublicKeyHex.parse(testAuthorPublicKey);
    expect(await store.loadPublishedPosts(), isEmpty);
    expect(await store.loadFollowedProfiles(), isEmpty);
    expect(await store.loadBlockedProfiles(), isEmpty);

    account = NostrPublicKeyHex.parse(testViewerPublicKey);
    expect((await store.loadPublishedPosts()).single.id, firstPost.id);
    expect(await store.loadFollowedProfiles(), {firstFollow});
    expect(await store.loadBlockedProfiles(), {firstBlock});
  });
}
