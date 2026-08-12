import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/session/data/pending_first_session_repository.dart';
import 'package:ghostr/features/session/domain/pending_account_setup.dart';

import '../support/account_creation_fakes.dart';
import '../support/fake_account_provisioning_repository.dart';
import '../support/fake_session_repository.dart';
import '../support/sample_data.dart';

void main() {
  test('reset clears pending setup before the active session', () async {
    final calls = <String>[];
    final provisioning = _RecordingProvisioning(calls)
      ..pending = PendingAccountSetup(
        account: accountCreationAccount(),
        metadata: accountCreationMetadata(),
      );
    final inner = _RecordingSessionRepository(calls)
      ..storedSession = sampleSession();
    final repository = PendingFirstSessionRepository(inner, provisioning);

    await repository.resetStoredSession();

    expect(calls, ['discard', 'reset']);
  });
}

final class _RecordingProvisioning extends FakeAccountProvisioningRepository {
  _RecordingProvisioning(this.calls);
  final List<String> calls;

  @override
  Future<void> discard() async {
    calls.add('discard');
    await super.discard();
  }
}

final class _RecordingSessionRepository extends FakeSessionRepository {
  _RecordingSessionRepository(this.calls);
  final List<String> calls;

  @override
  Future<void> resetStoredSession() async {
    calls.add('reset');
    await super.resetStoredSession();
  }
}
