import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/social/presentation/blocked_accounts_cubit.dart';
import 'package:ghostr/features/social/presentation/blocked_accounts_screen.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

import '../support/fake_social_graph_repository.dart';
import '../support/map_profile_metadata_repository.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('renders blocked accounts with names and unblock actions',
      (tester) async {
    final named = ProfileId.parse('npub1named');
    final anonymous = ProfileId.parse('npub1anonymousandverylongvalue');
    final social = FakeSocialGraphRepository(blocked: {named, anonymous});
    final metadata = MapProfileMetadataRepository({
      named: sampleCreator(id: 'npub1named', displayName: 'Alice Relay'),
    });

    await tester.pumpWidget(
      MaterialApp(
        home: BlocProvider(
          create: (_) => BlockedAccountsCubit(social, metadata)..load(),
          // Deliberately non-const so the constructor executes at runtime.
          child: BlockedAccountsScreen(),
        ),
      ),
    );
    expect(
      find.bySemanticsLabel('Loading blocked accounts'),
      findsOneWidget,
    );
    await tester.pumpAndSettle();

    expect(find.text('Alice Relay'), findsOneWidget);
    expect(find.text('npub1named'), findsOneWidget);
    expect(find.text('npub1anonym…alue'), findsOneWidget);
    expect(find.widgetWithText(TextButton, 'Unblock'), findsNWidgets(2));
  });
}
