import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_spec_builder.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/src/rust/api/feed_types.dart';

import '../support/nostr_test_values.dart';

void main() {
  test('Following uses its own native feed kind instead of Profile', () {
    final spec = buildRustFollowingFeedSpec({
      ProfileId.parse(testViewerNpub),
      ProfileId.parse(testCreatorNpub),
    });

    expect(spec?.kind, FfiFeedKind.following);
    expect(
      spec?.creators,
      unorderedEquals([testViewerPublicKey, testCreatorPublicKey]),
    );
  });
}
