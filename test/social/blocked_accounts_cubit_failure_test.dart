import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/social/presentation/blocked_accounts_cubit.dart';

import '../support/fake_social_graph_repository.dart';
import '../support/map_profile_metadata_repository.dart';

void main() {
  test('surfaces a retryable message when the block list read fails',
      () async {
    final social = FakeSocialGraphRepository()
      ..loadFailure = const AppFailure('Relays unavailable.');
    final cubit = BlockedAccountsCubit(social, MapProfileMetadataRepository());

    await cubit.load();

    final state = cubit.state;
    expect(state, isA<BlockedAccountsFailure>());
    expect((state as BlockedAccountsFailure).message, 'Relays unavailable.');

    social.loadFailure = null;
    await cubit.load();
    expect(cubit.state, isA<BlockedAccountsEmpty>());
  });
}
