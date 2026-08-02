import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/app_controller_factory.dart';
import 'package:ghostr/app/session_gate.dart';
import 'package:ghostr/features/session/presentation/session_cubit.dart';

import '../support/fakes.dart';

void main() {
  testWidgets('shows the session loading contract before restoration',
      (tester) async {
    final dependencies = buildFakeDependencies(
      catalogRepository: FakeVideoCatalogRepository(forYouFeed: const []),
    );

    await tester.pumpWidget(MaterialApp(
      home: BlocProvider(
        create: (_) => SessionCubit(dependencies.sessionRepository),
        child: SessionGate(
          controllers: AppControllerFactory(dependencies),
        ),
      ),
    ));

    expect(find.text('Booting Ghostr'), findsOneWidget);
  });
}
