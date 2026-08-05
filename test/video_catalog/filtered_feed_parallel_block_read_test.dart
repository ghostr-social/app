import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/social/domain/social_graph_repository.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/filtered_video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/domain/video_post_reader.dart';

import '../support/sample_data.dart';

void main() {
  test('main feed overlaps mute lookup and still excludes blocks', () async {
    final reader = _DeferredReader();
    final social = _DeferredSocial();
    final repository = FilteredVideoFeedRepository(reader, social);

    final loading = repository.loadFeed(FeedKind.forYou);
    var completed = false;
    loading.whenComplete(() => completed = true);
    await Future<void>.delayed(Duration.zero);

    expect(reader.started, isTrue);
    expect(social.started, isTrue);

    final post = samplePost();
    reader.result.complete([post]);
    await Future<void>.delayed(Duration.zero);
    expect(completed, isFalse);

    social.result.complete({post.creator.id});
    expect(await loading, isEmpty);
  });
}

final class _DeferredReader implements VideoPostReader {
  final result = Completer<List<VideoPost>>();
  bool started = false;

  @override
  Future<List<VideoPost>> load({
    Set<ProfileId>? creatorIds,
    String? searchQuery,
    Set<String>? hashtags,
  }) {
    started = true;
    return result.future;
  }

  @override
  Future<List<VideoPost>> loadOlder({
    required DateTime olderThan,
    Set<ProfileId>? creatorIds,
  }) async => const <VideoPost>[];
}

final class _DeferredSocial implements SocialGraphRepository {
  final result = Completer<Set<ProfileId>>();
  bool started = false;

  @override
  Future<Set<ProfileId>> loadBlockedProfiles() {
    started = true;
    return result.future;
  }

  @override
  Future<Set<ProfileId>> loadFollowedProfiles() async => const <ProfileId>{};

  @override
  Future<bool> toggleBlock(ProfileId profileId) async => false;

  @override
  Future<bool> toggleFollow(ProfileId profileId) async => false;
}
