import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/social/presentation/blocked_accounts_cubit.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

import '../support/fake_social_graph_repository.dart';
import '../support/map_profile_metadata_repository.dart';

void main() {
  test('unblocking removes the account and keeps the remaining ones',
      () async {
    final reverted = ProfileId.parse('npub1reverted');
    final kept = ProfileId.parse('npub1kept');
    final social = FakeSocialGraphRepository(blocked: {reverted, kept});
    final cubit = BlockedAccountsCubit(social, MapProfileMetadataRepository());
    await cubit.load();

    await cubit.unblock(reverted);

    expect(social.toggledBlocks, [reverted]);
    expect(social.blocked, {kept});
    final state = cubit.state;
    expect(state, isA<BlockedAccountsLoaded>());
    expect(
      (state as BlockedAccountsLoaded).accounts.map((account) => account.id),
      [kept],
    );
  });
}
