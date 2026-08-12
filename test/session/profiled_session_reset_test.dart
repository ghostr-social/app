import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/session/data/profiled_session_repository.dart';

import '../support/fake_profile_metadata_repository.dart';
import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('stored-session reset is forwarded to the inner repository', () async {
    final inner = FakeSessionRepository(storedSession: sampleSession());
    final repository = ProfiledSessionRepository(
      inner,
      FakeProfileMetadataRepository(),
    );

    await repository.resetStoredSession();

    expect(inner.storedSession, isNull);
  });
}
