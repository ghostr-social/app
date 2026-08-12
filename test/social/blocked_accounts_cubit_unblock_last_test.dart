import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/social/presentation/blocked_accounts_cubit.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

import '../support/fake_social_graph_repository.dart';
import '../support/map_profile_metadata_repository.dart';

void main() {
  test('unblocking the last account lands on the empty state', () async {
    final reverted = ProfileId.parse('npub1reverted');
    final social = FakeSocialGraphRepository(blocked: {reverted});
    final cubit = BlockedAccountsCubit(social, MapProfileMetadataRepository());
    await cubit.load();

    await cubit.unblock(reverted);

    expect(social.blocked, isEmpty);
    expect(cubit.state, isA<BlockedAccountsEmpty>());
  });
}
