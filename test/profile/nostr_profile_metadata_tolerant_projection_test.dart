import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/profile/data/nostr_profile_metadata_mapper.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

import '../support/nostr_test_values.dart';
import '../support/profile_metadata_event.dart';

void main() {
  test(
    'projects unconventional relay metadata into a bounded safe summary',
    () {
      final longName = List<String>.filled(60, 'N').join();
      final summary = const NostrProfileMetadataMapper().summaryFromEvent(
        profileMetadataEvent(
          '{"display_name":"  $longName  ","name":"Alice.Dev",'
          '"picture":"javascript:alert(1)"}',
        ),
        ProfileId.parse(testViewerNpub),
      );

      expect(summary.displayName, List<String>.filled(50, 'N').join());
      expect(summary.handle, '@Alice.Dev');
      expect(summary.avatarUrl, isNull);
    },
  );
}
