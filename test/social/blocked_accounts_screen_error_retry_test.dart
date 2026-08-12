import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/social/presentation/blocked_accounts_cubit.dart';
import 'package:ghostr/features/social/presentation/blocked_accounts_screen.dart';

import '../support/fake_social_graph_repository.dart';
import '../support/map_profile_metadata_repository.dart';

void main() {
  testWidgets('retries a failed block list load successfully',
      (tester) async {
    final social = FakeSocialGraphRepository()
      ..loadFailure = const AppFailure('Relays unavailable.');

    await tester.pumpWidget(
      MaterialApp(
        home: BlocProvider(
          create: (_) => BlockedAccountsCubit(
            social,
            MapProfileMetadataRepository(),
          )..load(),
          child: const BlockedAccountsScreen(),
        ),
      ),
    );
    await tester.pumpAndSettle();
    expect(find.text('Relays unavailable.'), findsOneWidget);

    social.loadFailure = null;
    await tester.tap(find.text('Retry'));
    await tester.pumpAndSettle();

    expect(find.text('No blocked accounts'), findsOneWidget);
  });
}
