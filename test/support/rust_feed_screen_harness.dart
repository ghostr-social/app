import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:ghostr/features/comments/presentation/comments_cubit.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_remote_source.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_feed_updates.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_hunt.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_screen.dart';

import 'discovery_search_fakes.dart';
import 'fake_media_ports.dart';
import 'following_feed_scope_fixture.dart';
import 'hybrid_repository_harness.dart';
import 'live_rust_feed_port.dart';
import 'fake_video_sharing.dart';
import 'rust_feed_fixtures.dart';

final class RustFeedScreenHarness {
  const RustFeedScreenHarness._({
    required this.port,
    required this.source,
    required this.repositories,
  });

  final LiveRustFeedPort port;
  final RustFeedRemoteSource source;
  final HybridRepositoryHarness repositories;

  VideoFeedRepository get feed => repositories.feed;

  static Future<RustFeedScreenHarness> empty() async {
    final port = LiveRustFeedPort(firstPage: [rustFeedUpdate(revision: 1)])
      ..moreAvailable = false;
    final source = RustFeedRemoteSource(port: port);
    return RustFeedScreenHarness._(
      port: port,
      source: source,
      repositories: await buildHybridRepositoryHarness(source),
    );
  }

  Widget app() {
    return MaterialApp(
      home: BlocProvider(
        create: (_) => _cubit()..load(),
        child: Scaffold(
          body: FeedScreen(
            bindings: FeedScreenBindings(
              onOpenProfile: (_) {},
              onOpenHashtag: (_) {},
              playbackPort: FakeVideoPlaybackPort(),
              shareWorkflow: FakeVideoShareWorkflow(),
              createComments: (post) =>
                  CommentsCubit(repositories.comments, post),
              isActive: true,
            ),
          ),
        ),
      ),
    );
  }

  FeedCubit _cubit() {
    return FeedCubit(
      FeedDependencies(
        feed: feed,
        engagement: repositories.engagement,
        optional: FeedOptionalDependencies(
          delivery: FeedDeliveryDependencies(
            updates: RemoteVideoFeedUpdates(
              remote: source,
              followingScopes: testFollowingFeedScopes(FakeSocialGraph()),
            ),
          ),
        ),
      ),
      hunt: FeedHunt(
        base: const Duration(minutes: 1),
        cap: const Duration(minutes: 1),
      ),
    );
  }
}
