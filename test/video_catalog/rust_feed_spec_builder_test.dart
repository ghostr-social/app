import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_spec_builder.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

import '../support/nostr_test_values.dart';

void main() {
  // Precedence mirrors remoteVideoRetrievalContext
  // (scheduled_remote_video_source.dart): search, tag, profile, feed.
  test('a search term opens a search feed', () {
    final spec = buildRustFeedSpec(
      searchQuery: 'ghost tapes',
      hashtags: {'ghostr'},
    );

    expect(spec?.kind, 'search');
    expect(spec?.value, 'ghost tapes');
  });

  test('a hashtag opens a hashtag feed', () {
    final spec = buildRustFeedSpec(hashtags: {'ghostr'});

    expect(spec?.kind, 'hashtag');
    expect(spec?.value, 'ghostr');
  });

  test('an empty hashtag set falls through to the main feed', () {
    final spec = buildRustFeedSpec(
      hashtags: const <String>{},
      viewerPubkeyHex: testViewerPublicKey,
    );

    expect(spec?.kind, 'main');
  });

  test('a creator id opens a profile feed keyed by the decoded hex', () {
    final spec = buildRustFeedSpec(
      creatorIds: {ProfileId.parse(testViewerNpub)},
    );

    expect(spec?.kind, 'profile');
    expect(spec?.value, testViewerPublicKey);
  });

  test('no filters open the viewer main feed', () {
    final spec = buildRustFeedSpec(viewerPubkeyHex: testViewerPublicKey);

    expect(spec?.kind, 'main');
    expect(spec?.viewerPubkey, testViewerPublicKey);
    expect(spec?.value, isNull);
  });
}
