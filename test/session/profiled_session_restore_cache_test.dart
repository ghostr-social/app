import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/session/data/profiled_session_repository.dart';

import '../support/fake_profile_metadata_repository.dart';
import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test(
    'restored session uses cached public profile metadata immediately',
    () async {
      final stored = sampleSession();
      final cached = sampleCreator(
        id: stored.profile.id,
        displayName: 'Cached Nora',
      );
      final profiles = FakeProfileMetadataRepository()..cached = cached;
      final repository = ProfiledSessionRepository(
        FakeSessionRepository(storedSession: stored),
        profiles,
      );

      final restored = await repository.restore();

      expect(restored?.profile, same(cached));
    },
  );
}
