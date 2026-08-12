import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/social/presentation/blocked_accounts_cubit.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

import '../support/fake_social_graph_repository.dart';
import '../support/map_profile_metadata_repository.dart';

void main() {
  test('translates an unexpected unblock error into a safe message', () async {
    final blocked = ProfileId.parse('npub1blocked');
    final social = FakeSocialGraphRepository(blocked: {blocked})
      ..toggleFailure = StateError('boom');
    final cubit = BlockedAccountsCubit(social, MapProfileMetadataRepository());
    await cubit.load();

    await cubit.unblock(blocked);

    final state = cubit.state;
    expect(state, isA<BlockedAccountsFailure>());
    expect((state as BlockedAccountsFailure).message, isNotEmpty);
  });
}
