import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/social/presentation/blocked_accounts_cubit.dart';
import 'package:ghostr/features/social/presentation/blocked_accounts_screen.dart';

import '../support/fake_social_graph_repository.dart';
import '../support/map_profile_metadata_repository.dart';

void main() {
  testWidgets('explains the empty state when nothing is blocked',
      (tester) async {
    await tester.pumpWidget(
      MaterialApp(
        home: BlocProvider(
          create: (_) => BlockedAccountsCubit(
            FakeSocialGraphRepository(),
            MapProfileMetadataRepository(),
          )..load(),
          child: const BlockedAccountsScreen(),
        ),
      ),
    );
    await tester.pumpAndSettle();

    expect(find.text('No blocked accounts'), findsOneWidget);
  });
}
