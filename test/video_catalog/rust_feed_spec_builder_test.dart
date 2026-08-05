import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_spec_builder.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/src/rust/api/feed_types.dart';

import '../support/nostr_test_values.dart';

void main() {
  // Feed precedence is search, tag, profile, then main.
  test('a search term opens a search feed', () {
    final spec = buildRustFeedSpec(
      searchQuery: 'ghost tapes',
      hashtags: {'ghostr'},
    );

    expect(spec?.kind, FfiFeedKind.search);
    expect(spec?.value, 'ghost tapes');
  });

  test('a hashtag opens a hashtag feed', () {
    final spec = buildRustFeedSpec(hashtags: {'ghostr'});

    expect(spec?.kind, FfiFeedKind.hashtag);
    expect(spec?.value, 'ghostr');
  });

  test('an empty hashtag set falls through to the main feed', () {
    final spec = buildRustFeedSpec(
      hashtags: const <String>{},
      viewerPubkeyHex: testViewerPublicKey,
    );

    expect(spec?.kind, FfiFeedKind.main);
  });

  test('a creator id opens a profile feed keyed by the decoded hex', () {
    final spec = buildRustFeedSpec(
      creatorIds: {ProfileId.parse(testViewerNpub)},
    );

    expect(spec?.kind, FfiFeedKind.profile);
    expect(spec?.creators, [testViewerPublicKey]);
  });

  test('no filters open the viewer main feed', () {
    final spec = buildRustFeedSpec(viewerPubkeyHex: testViewerPublicKey);

    expect(spec?.kind, FfiFeedKind.main);
    expect(spec?.viewerPubkey, testViewerPublicKey);
    expect(spec?.value, isNull);
  });

  test('no viewer opens the signed-out main feed', () {
    final spec = buildRustFeedSpec();

    expect(spec?.kind, FfiFeedKind.main);
    expect(spec?.viewerPubkey, isNull);
  });
}
