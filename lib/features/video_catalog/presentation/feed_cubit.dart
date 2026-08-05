import 'dart:async';

import 'package:ghostr/core/presentation/disposal_safe_cubit.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/use_cases/feed_backfill.dart';
import 'package:ghostr/features/video_catalog/domain/use_cases/feed_engagement.dart';
import 'package:ghostr/features/video_catalog/domain/use_cases/feed_fetcher.dart';
import 'package:ghostr/features/video_catalog/domain/use_cases/feed_loads.dart';
import 'package:ghostr/features/video_catalog/domain/use_cases/feed_session.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_dependencies.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_failure_messages.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_hunt.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_state.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_viewer.dart';

export 'feed_dependencies.dart';
export 'feed_state.dart';

part 'feed_cubit_engagement.dart';

/// Turns feed intents into feed states. The rules behind a transition — what
/// survives a refresh, when to dig into the past — live in collaborators.
class FeedCubit extends DisposalSafeCubit<FeedState> {
  FeedCubit(this._dependencies, {FeedHunt? hunt})
      : _hunt = hunt ?? FeedHunt(),
        super(const FeedLoading(FeedKind.forYou));

  final FeedDependencies _dependencies;
  final FeedHunt _hunt;
  final _loads = FeedLoads();
  final _session = FeedSession();
  late final _fetch = FeedFetcher(_dependencies.feed);
  late final _backfill = FeedBackfill(_fetch, _loads);
  late final _engagement =
      FeedEngagement(_dependencies.engagement, _dependencies.social);
  late final _viewer = FeedViewer(
    focus: _dependencies.focus,
    watchTracker: _dependencies.watchTracker,
  );

  Future<void> load([FeedKind? selectedKind]) {
    final kind = selectedKind ?? state.kind;
    emit(FeedLoading(kind));
    return _accepting(kind, (reason) => FeedFailure(kind, reason));
  }

  Future<void> retry() => load();

  Future<void> reload() {
    final previous = state;
    if (previous is! FeedLoaded) return load();
    emit(FeedLoading(previous.kind));
    return _accepting(previous.kind, previous.withNotice);
  }

  Future<void> refresh() async {
    final previous = state;
    if (previous is! FeedLoaded) return load();
    final result = await _loads.newest(() => _fetch.resync(previous.kind));
    if (isClosed || result == null) return;
    switch (result) {
      case FeedUnavailable():
        emit(previous.withNotice(feedLoadFailureMessage(result.failure)));
      case FeedFetched(:final posts):
        _acceptRefresh(previous, posts);
    }
  }

  Future<void> _accepting(
    FeedKind kind,
    FeedState Function(String reason) unavailable,
  ) async {
    final result = await _loads.newest(() => _fetch.unwatched(kind));
    if (isClosed || result == null) return;
    switch (result) {
      case FeedUnavailable():
        emit(unavailable(feedLoadFailureMessage(result.failure)));
      case FeedFetched(:final posts):
        _acceptLoad(kind, posts);
    }
  }

  void _acceptLoad(FeedKind kind, List<VideoPost> fresh) {
    final roster = _session.loaded(fresh);
    _backfill.restartFrom(roster.posts);
    if (roster.isEmpty) return _emitEmpty(kind);
    emit(FeedLoaded.of(kind, roster));
    _hunt.filled();
    _viewer.landedOn(roster.posts, 0);
    _ensureBuffered();
  }

  void _acceptRefresh(FeedLoaded initial, List<VideoPost> refreshed) {
    final current = state is FeedLoaded ? state as FeedLoaded : initial;
    final roster = _session.resynced(current.roster, refreshed);
    if (roster.isEmpty) return _emitEmpty(current.kind);
    emit(FeedLoaded.of(current.kind, roster));
    _viewer.stayedOn(roster.active);
  }

  void _emitEmpty(FeedKind kind) {
    emit(FeedEmpty(kind));
    _hunt.emptied(_startHuntAttempt);
  }

  // An empty feed is never a dead end: reload quietly — no spinner, no
  // state churn — until the relays finally hand over content.
  Future<void> _huntEmptyFeed() async {
    final current = state;
    if (current is! FeedEmpty) return;
    final result = await _loads.newest(() => _fetch.unwatched(current.kind));
    if (isClosed || result == null) return;
    if (result case FeedFetched(:final posts) when posts.isNotEmpty) {
      return _acceptLoad(current.kind, posts);
    }
    _hunt.emptied(_startHuntAttempt);
  }

  void _startHuntAttempt() => unawaited(_huntEmptyFeed());

  void pageChanged(int index) {
    final current = state;
    if (current is! FeedLoaded) return;
    if (index < 0 || index >= current.posts.length) return;
    emit(current.withPage(index));
    _viewer.landedOn(current.posts, index);
    _ensureBuffered();
  }

  void _ensureBuffered() {
    final current = state;
    if (current is! FeedLoaded) return;
    if (_backfill.isStarved(current.roster)) unawaited(loadMore());
  }

  Future<void> loadMore() async {
    if (state is! FeedLoaded) return;
    final dug = await _backfill.dig(state.kind);
    if (dug case FeedDigFailed(:final failure)) {
      return _showNotice(feedLoadFailureMessage(failure.failure));
    }
    if (dug case FeedDigPage(:final posts)) _appendPage(posts);
  }

  void _appendPage(List<VideoPost> incoming) {
    final current = state;
    if (current is! FeedLoaded) return;
    final posts = _session.appended(current.roster, incoming);
    if (posts == null) return;
    emit(current.withPosts(posts));
    _ensureBuffered();
  }

  void clearNotice() {
    final current = state;
    if (current is! FeedLoaded || current.notice == null) return;
    emit(current.withoutNotice());
  }

  void _showNotice(String message) {
    final current = state;
    if (current is FeedLoaded) emit(current.withNotice(message));
  }

  void _emitState(FeedState next) => emit(next);

  @override
  Future<void> close() {
    _hunt.dispose();
    return super.close();
  }
}
