import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/app_controller_factory.dart';
import 'package:ghostr/app/session_gate.dart';
import 'package:ghostr/features/session/presentation/session_cubit.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('identifies a pending sign-out operation', (tester) async {
    final repository = _PendingSignOutRepository();
    final dependencies = buildFakeDependencies(
      catalogRepository: FakeVideoCatalogRepository(forYouFeed: []),
      overrides: FakeDependencyOverrides(sessionRepository: repository),
    );
    final cubit = SessionCubit(repository);
    await cubit.restore();
    final signOut = cubit.signOut();

    await tester.pumpWidget(
      MaterialApp(
        home: BlocProvider.value(
          value: cubit,
          child: SessionGate(controllers: AppControllerFactory(dependencies)),
        ),
      ),
    );

    expect(find.text('Signing out'), findsOneWidget);
    repository.pending.complete();
    await signOut;
    await tester.pumpWidget(const SizedBox());
    await cubit.close();
  });
}

class _PendingSignOutRepository extends FakeSessionRepository {
  _PendingSignOutRepository() : super(storedSession: sampleSession());

  final pending = Completer<void>();

  @override
  Future<void> signOut() => pending.future;
}
