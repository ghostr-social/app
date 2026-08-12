import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/session/data/pending_first_session_repository.dart';

import '../support/fake_account_provisioning_repository.dart';
import '../support/fake_session_repository.dart';
import '../support/sample_data.dart';

void main() {
  test('sign out cannot fail after the active session was cleared', () async {
    final inner = FakeSessionRepository(storedSession: sampleSession());
    final provisioning = FakeAccountProvisioningRepository()
      ..discardFailure = StateError('pending store unavailable');
    final repository = PendingFirstSessionRepository(inner, provisioning);

    await repository.signOut();

    expect(inner.storedSession, isNull);
    expect(provisioning.discardCount, 0);
  });
}
