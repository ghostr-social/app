import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fakes.dart';
import '../support/follow_profile_workflow.dart';
import '../support/sample_data.dart';

void main() {
  test(
    'loads followed creators with the feed before offering follows',
    () async {
      final followed = sampleCreator(id: 'followed');
      final available = sampleCreator(id: 'available');
      final repository = _PendingMembershipRepository(
        forYouFeed: [
          samplePost(id: 'one', creator: followed),
          samplePost(id: 'two', creator: available),
        ],
      );
      final cubit = FeedCubit(
        FeedDependencies(
          viewerId: sampleSession().profile.id,
          feed: repository,
          engagement: repository,
          followProfile: testFollowProfileWorkflow(repository),
          optional: FeedOptionalDependencies(social: repository),
        ),
      );
      addTearDown(cubit.close);

      final load = cubit.load();
      await Future<void>.delayed(Duration.zero);

      var loaded = cubit.state as FeedLoaded;
      expect(loaded.canFollow(followed.id), isFalse);
      expect(loaded.canFollow(available.id), isFalse);

      repository.pending.complete({followed.id});
      await load;
      loaded = cubit.state as FeedLoaded;
      expect(loaded.canFollow(followed.id), isFalse);
      expect(loaded.canFollow(available.id), isTrue);
    },
  );
}

final class _PendingMembershipRepository extends FakeVideoCatalogRepository {
  _PendingMembershipRepository({required super.forYouFeed});

  final pending = Completer<Set<ProfileId>>();

  @override
  Future<Set<ProfileId>> loadFollowedProfiles() => pending.future;
}
