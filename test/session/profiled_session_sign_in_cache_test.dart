import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/session/data/profiled_session_repository.dart';
import 'package:ghostr/features/session/domain/auth_secret.dart';

import '../support/fake_profile_metadata_repository.dart';
import '../support/fakes.dart';
import '../support/nostr_test_values.dart';
import '../support/sample_data.dart';

void main() {
  test('signed-in session uses its cached public profile metadata', () async {
    final cached = sampleCreator(
      id: testViewerNpub,
      displayName: 'Cached Nora',
    );
    final repository = ProfiledSessionRepository(
      FakeSessionRepository(),
      FakeProfileMetadataRepository()..cached = cached,
    );

    final session = await repository.signIn(AuthSecret.parse(testNsec));

    expect(session.profile, same(cached));
  });
}
