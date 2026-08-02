import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/session/presentation/session_cubit.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('a pending sign out leaves no signed-in reentry window', () async {
    final repository = _PendingSignOutRepository();
    final cubit = SessionCubit(repository);
    await cubit.restore();

    final first = cubit.signOut();
    final second = cubit.signOut();
    final pendingState = cubit.state;
    final callCount = repository.signOutCount;
    repository.pending.complete();
    await Future.wait([first, second]);

    expect(pendingState, isA<SessionSigningOut>());
    expect(callCount, 1);
    expect(cubit.state, isA<SessionSignedOut>());
    await cubit.close();
  });
}

class _PendingSignOutRepository extends FakeSessionRepository {
  _PendingSignOutRepository() : super(storedSession: sampleSession());

  final pending = Completer<void>();
  int signOutCount = 0;

  @override
  Future<void> signOut() {
    signOutCount += 1;
    return pending.future;
  }
}
