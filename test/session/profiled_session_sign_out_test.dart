import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/session/data/profiled_session_repository.dart';

import '../support/fake_profile_metadata_repository.dart';
import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('sign-out is forwarded to the stored session repository', () async {
    final inner = FakeSessionRepository(storedSession: sampleSession());
    final repository = ProfiledSessionRepository(
      inner,
      FakeProfileMetadataRepository(),
    );

    await repository.signOut();

    expect(inner.storedSession, isNull);
  });
}
