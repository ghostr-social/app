import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/social/presentation/blocked_accounts_cubit.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';

import '../support/fake_social_graph_repository.dart';
import '../support/map_profile_metadata_repository.dart';
import '../support/sample_data.dart';

void main() {
  test('lists blocked accounts with cached names, sorted by label', () async {
    final named = ProfileId.parse('npub1named');
    final anonymous = ProfileId.parse('npub1anonymousandverylongvalue');
    final social = FakeSocialGraphRepository(blocked: {anonymous, named});
    final metadata = MapProfileMetadataRepository({
      named: sampleCreator(id: 'npub1named', displayName: 'Alice Relay'),
    });
    final cubit = BlockedAccountsCubit(social, metadata);

    await cubit.load();

    final state = cubit.state;
    expect(state, isA<BlockedAccountsLoaded>());
    final accounts = (state as BlockedAccountsLoaded).accounts;
    expect(accounts.map((account) => account.id), [named, anonymous]);
    expect(accounts.first.label, 'Alice Relay');
    expect(accounts.last.label, 'npub1anonym…alue');
  });
}
