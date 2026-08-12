import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/profile/data/nostr_profile_metadata_mapper.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

import '../support/nostr_test_values.dart';
import '../support/profile_metadata_event.dart';

void main() {
  test('uses the Nostr name when display_name is absent', () {
    final summary = const NostrProfileMetadataMapper().summaryFromEvent(
      profileMetadataEvent('{"name":"nora_relay"}'),
      ProfileId.parse(testViewerNpub),
    );

    expect(summary.displayName, 'nora_relay');
    expect(summary.handle, '@nora_relay');
  });
}
