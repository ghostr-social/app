import 'dart:async';

import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/presentation/disposal_safe_cubit.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/feed_focus_port.dart';
import 'package:ghostr/features/video_catalog/domain/use_cases/feed_backfill.dart';
import 'package:ghostr/features/video_catalog/domain/use_cases/feed_engagement.dart';
import 'package:ghostr/features/video_catalog/domain/use_cases/feed_fetcher.dart';
import 'package:ghostr/features/video_catalog/domain/use_cases/feed_loads.dart';
import 'package:ghostr/features/video_catalog/domain/use_cases/feed_operation_failure.dart';
import 'package:ghostr/features/video_catalog/domain/use_cases/feed_reposts.dart';
import 'package:ghostr/features/video_catalog/domain/use_cases/feed_session.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_updates.dart';
import 'package:ghostr/features/video_catalog/domain/video_delivery_updates.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/domain/video_post_id.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_dependencies.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_backfill_retry.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_failure_messages.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_follow_state.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_hunt.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_ready_selector.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_state.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_update_retry.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_viewer.dart';

export 'feed_dependencies.dart';
export 'feed_state.dart';

part 'feed_cubit_engagement.dart';
part 'feed_cubit_backfill.dart';
part 'feed_cubit_follow.dart';
part 'feed_cubit_loading.dart';
part 'feed_cubit_update_loading.dart';
part 'feed_update_state.dart';
part 'feed_cubit_updates.dart';
part 'feed_cubit_delivery.dart';

/// Turns feed intents into feed states. The rules behind a transition — what
/// survives a refresh, when to dig into the past — live in collaborators.
class FeedCubit extends DisposalSafeCubit<FeedState> {
  FeedCubit(
    this._dependencies, {
    VideoPostId? openAt,
    FeedHunt? hunt,
    FeedUpdateRetry? updateRetry,
    FeedBackfillRetry? backfillRetry,
  }) : _openAt = openAt,
       _backfillRetry = backfillRetry ?? FeedBackfillRetry(),
       _updates = _FeedUpdateState(updateRetry ?? FeedUpdateRetry()),
       _hunt = hunt ?? FeedHunt(),
       super(const FeedLoading(FeedKind.forYou)) {
    _startDeliveryUpdates();
  }

  final FeedDependencies _dependencies;

  /// The video every full load opens on, when the feed still carries it.
  final VideoPostId? _openAt;
  final FeedHunt _hunt;
  final FeedBackfillRetry _backfillRetry;
  final _loads = FeedLoads();
  final _session = FeedSession();
  final _FeedUpdateState _updates;
  final _readySelector = const FeedReadySelector();
  final _delivery = <PlaybackDeliveryId, VideoDeliverySnapshot>{};
  StreamSubscription<VideoDeliverySnapshot>? _deliverySubscription;
  ({int fromIndex, int intendedIndex, bool graceExpired})?
  _awaitingTransportRescue;
  Timer? _rescueTimer;
  int? _pendingTransportJump;
  late final _fetch = FeedFetcher(_dependencies.feed);
  late final _backfill = FeedBackfill(_fetch, _loads);
  late final _engagement = FeedEngagement(
    _dependencies.engagement,
    _dependencies.social,
  );
  late final _reposts = _dependencies.optional.reposts == null
      ? null
      : FeedReposts(_dependencies.optional.reposts!);
  late final _viewer = FeedViewer(
    focus: _dependencies.focus,
    watchTracker: _dependencies.watchTracker,
  );
  late FeedFollowState _follows = FeedFollowState.unavailable(
    viewerId: _dependencies.viewerId,
  );
  int _followLoadRequest = 0;
  final _followRequests = <ProfileId, int>{};
  var _isClosing = false;

  Future<void> load([FeedKind? selectedKind]) async {
    _reposts?.forget();
    final follows = _reloadFollows();
    await _runFeedPull(() async {
      final kind = selectedKind ?? state.kind;
      emit(FeedLoading(kind));
      await _ensureFeedUpdates(kind);
      if (isClosed || state.kind != kind) return;
      await _accepting(kind, (reason) => FeedFailure(kind, reason));
    });
    await follows;
  }

  Future<void> retry() => load();

  Future<void> reload() async {
    final previous = state;
    if (previous is! FeedLoaded) return load();
    _reposts?.forget();
    final follows = _reloadFollows();
    emit(FeedLoading(previous.kind));
    await _runFeedPull(
      () => _accepting(
        previous.kind,
        (reason) => previous.withFollows(_follows).withNotice(reason),
      ),
    );
    await follows;
  }

  Future<void> refresh() async {
    final previous = state;
    if (previous is! FeedLoaded) return load();
    _reposts?.forget();
    final follows = _reloadFollows();
    await _runFeedPull(() async {
      await _refreshFeedUpdates(previous.kind);
      final result = await _loads.newest(() => _fetch.resync(previous.kind));
      if (isClosed || result == null) return;
      if (result case FeedUnavailable()) {
        emit(
          previous
              .withFollows(_follows)
              .withNotice(feedLoadFailureMessage(result.failure)),
        );
      } else if (result case FeedFetched(:final posts, :final eligiblePosts)) {
        _acknowledgePendingFeedUpdate();
        _acceptRefresh(previous, posts, eligiblePosts);
      }
    });
    await follows;
  }

  void pageChanged(int index) {
    final current = state;
    if (current is! FeedLoaded) return;
    if (index < 0 || index >= current.posts.length) return;
    if (_consumeTransportJump(index)) return;
    final decision = _readyDecision(current, index);
    if (decision.action == FeedReadyAction.rescue) {
      return _rescueTo(current, decision);
    }
    _rememberPendingRescue(current, decision);
    emit(current.withPage(index));
    _viewer.landedOn(current.posts, index);
    _ensureBuffered();
  }

  void surfaceVisibilityChanged(bool isVisible) {
    _viewer.visibilityChanged(isVisible);
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

  void _reportUpdateError(Object error, StackTrace stackTrace) {
    addError(error, stackTrace);
  }

  @override
  Future<void> close() async {
    if (_isClosing || isClosed) return;
    _isClosing = true;
    _loads.take();
    _hunt.dispose();
    _backfillRetry.cancel();
    _viewer.dispose();
    await _stopDeliveryUpdates();
    await _stopFeedUpdates();
    await super.close();
  }
}
