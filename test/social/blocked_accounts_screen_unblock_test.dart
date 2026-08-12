import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/social/presentation/blocked_accounts_cubit.dart';
import 'package:ghostr/features/social/presentation/blocked_accounts_screen.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

import '../support/fake_social_graph_repository.dart';
import '../support/map_profile_metadata_repository.dart';

void main() {
  testWidgets('tapping unblock reverts the block and updates the list',
      (tester) async {
    final blocked = ProfileId.parse('npub1blockedcreatorvalue');
    final social = FakeSocialGraphRepository(blocked: {blocked});

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

    await tester.tap(find.widgetWithText(TextButton, 'Unblock'));
    await tester.pumpAndSettle();

    expect(social.toggledBlocks, [blocked]);
    expect(social.blocked, isEmpty);
    expect(find.text('No blocked accounts'), findsOneWidget);
  });
}
