import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/profile/data/nostr_profile_metadata_mapper.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

import '../support/nostr_test_values.dart';
import '../support/profile_metadata_event.dart';

void main() {
  test('missing relay names fall back to a bounded public identity', () {
    final profileId = ProfileId.parse(testViewerNpub);

    final summary = const NostrProfileMetadataMapper().summaryFromEvent(
      profileMetadataEvent('{}'),
      profileId,
    );

    expect(summary.displayName, '${testViewerNpub.substring(0, 12)}…');
    expect(summary.handle, '@$testViewerNpub');
  });
}
