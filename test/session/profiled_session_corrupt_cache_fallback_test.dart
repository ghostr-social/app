import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/session/data/profiled_session_repository.dart';

import '../support/fake_profile_metadata_repository.dart';
import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('a broken public profile cache cannot block session restore', () async {
    final stored = sampleSession();
    final profiles = FakeProfileMetadataRepository()
      ..loadFailure = const AppFailure('Corrupt public profile cache.');
    final repository = ProfiledSessionRepository(
      FakeSessionRepository(storedSession: stored),
      profiles,
    );

    final restored = await repository.restore();

    expect(restored, same(stored));
  });
}
