import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/profile/data/nostr_profile_metadata_mapper.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

import '../support/nostr_test_values.dart';
import '../support/profile_metadata_event.dart';

void main() {
  test('parses Nostr kind-0 display_name, name, and picture fields', () {
    final summary = const NostrProfileMetadataMapper().summaryFromEvent(
      profileMetadataEvent(
        '{"display_name":" Nora Relay ","name":"NORA_RELAY",'
        '"picture":"https://cdn.example/nora.png"}',
      ),
      ProfileId.parse(testViewerNpub),
    );

    expect(summary.displayName, 'Nora Relay');
    expect(summary.handle, '@NORA_RELAY');
    expect(summary.avatarUrl, 'https://cdn.example/nora.png');
  });
}
