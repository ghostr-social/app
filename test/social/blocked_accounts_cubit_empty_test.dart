import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/social/presentation/blocked_accounts_cubit.dart';

import '../support/fake_social_graph_repository.dart';
import '../support/map_profile_metadata_repository.dart';

void main() {
  test('shows the empty state when nothing is blocked', () async {
    final cubit = BlockedAccountsCubit(
      FakeSocialGraphRepository(),
      MapProfileMetadataRepository(),
    );

    expect(cubit.state, isA<BlockedAccountsLoading>());
    await cubit.load();

    expect(cubit.state, isA<BlockedAccountsEmpty>());
  });
}
