import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/nostr_video_snapshot.dart';
import 'package:ghostr/features/video_catalog/data/remembering_remote_video_source.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_source.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

import '../support/fakes.dart';
import '../support/nostr_test_values.dart';
import '../support/nostr_video_post_fixture.dart';

void main() {
  test('retains the last non-empty canonical Nostr feed snapshot', () async {
    final post = nostrVideoPost(const NostrVideoPostFixture(
      eventId: testEventId,
      mediaId: 'remembered',
    ));
    final remote = FakeRemoteVideoSource([post]);
    final snapshot = NostrVideoSnapshot();
    final source = RememberingRemoteVideoSource(remote, snapshot);

    await source.loadRemoteFeed();
    remote.posts.clear();
    await source.loadRemoteFeed();

    expect(snapshot.read(), [post]);
  });

  test('a scoped result cannot replace the canonical full snapshot', () async {
    final full = nostrVideoPost(const NostrVideoPostFixture(
      eventId: testEventId,
      mediaId: 'full',
    ));
    final narrow = nostrVideoPost(const NostrVideoPostFixture(
      eventId: secondTestEventId,
      mediaId: 'narrow',
    ));
    final remote = FakeRemoteVideoSource([full]);
    final snapshot = NostrVideoSnapshot();
    final source = RememberingRemoteVideoSource(remote, snapshot);

    await source.loadRemoteFeed();
    remote.posts
      ..clear()
      ..add(narrow);
    await source.loadRemoteFeed(creatorIds: <ProfileId>{narrow.creator.id});

    expect(snapshot.read(), [full]);
  });

  test('an older overlapping full load cannot replace the newer result',
      () async {
    final older = nostrVideoPost(const NostrVideoPostFixture(
      eventId: testEventId,
      mediaId: 'older',
    ));
    final newer = nostrVideoPost(const NostrVideoPostFixture(
      eventId: secondTestEventId,
      mediaId: 'newer',
    ));
    final remote = _OverlappingRemote();
    final snapshot = NostrVideoSnapshot();
    final source = RememberingRemoteVideoSource(remote, snapshot);

    final first = source.loadRemoteFeed();
    final second = source.loadRemoteFeed();
    remote.second.complete(<VideoPost>[newer]);
    await second;
    remote.first.complete(<VideoPost>[older]);
    await first;

    expect(snapshot.read(), [newer]);
  });
}

class _OverlappingRemote implements RemoteVideoSource {
  final first = Completer<List<VideoPost>>();
  final second = Completer<List<VideoPost>>();
  var calls = 0;

  @override
  Future<List<VideoPost>> loadRemoteFeed({
    Set<ProfileId>? creatorIds,
    String? searchQuery,
  }) {
    return calls++ == 0 ? first.future : second.future;
  }
}
