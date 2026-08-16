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
import 'package:ghostr/features/video_catalog/domain/video_interaction_target.dart';
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
part 'feed_cubit_navigation.dart';
part 'feed_cubit_update_loading.dart';
part 'feed_update_state.dart';
part 'feed_cubit_updates.dart';
part 'feed_cubit_delivery.dart';

typedef _PendingTransportRescue = ({
  int fromIndex,
  int intendedIndex,
  VideoInteractionTarget intendedTarget,
  bool graceExpired,
});

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
  int _pageTransition = 0;
  var _isSurfaceVisible = true;
  var _reloadWhenSurfaceVisible = false;
  var _refreshWhenSurfaceVisible = false;
  var _isPreparingLoad = false;
  StreamSubscription<VideoDeliverySnapshot>? _deliverySubscription;
  _PendingTransportRescue? _awaitingTransportRescue;
  Timer? _rescueTimer;
  int? _pendingTransportJump;
  late final _fetch = FeedFetcher(
    _dependencies.feed,
    loadBlockedProfiles: _dependencies.social?.loadBlockedProfiles,
  );
  late final _backfill = FeedBackfill(_fetch, _loads);

  void surfaceVisibilityChanged(bool isVisible) =>
      _surfaceVisibilityChanged(isVisible);
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
    onWatchFailure: _watchPersistenceFailed,
  );
  late FeedFollowState _follows = FeedFollowState.unavailable(
    viewerId: _dependencies.viewerId,
  );
  int _followLoadRequest = 0;
  final _followRequests = <ProfileId, int>{};
  var _isClosing = false;

  Future<void> load([FeedKind? selectedKind]) async {
    _reloadWhenSurfaceVisible = false;
    _refreshWhenSurfaceVisible = false;
    _loads.take();
    _clearPendingRescue();
    _pendingTransportJump = null;
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
    _refreshWhenSurfaceVisible = false;
    final previous = state;
    if (previous is! FeedLoaded) return load();
    _clearPendingRescue();
    _pendingTransportJump = null;
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

  void _showNotice(String message) {
    final current = state;
    if (current is FeedLoaded) emit(current.withNotice(message));
  }

  void _emitState(FeedState next) => emit(next);

  void _reportUpdateError(Object error, StackTrace stackTrace) {
    addError(error, stackTrace);
  }

  void _watchPersistenceFailed(Object error, StackTrace stackTrace) {
    if (_isClosing || isClosed) return;
    _loads.take();
    _pageTransition += 1;
    _emitState(
      FeedFailure(
        state.kind,
        'Watch history is unavailable. Clear it in Settings to continue.',
      ),
    );
  }

  @override
  Future<void> close() async {
    if (_isClosing || isClosed) return;
    _isClosing = true;
    _loads.take();
    _hunt.dispose();
    _backfillRetry.cancel();
    await _viewer.dispose();
    await _stopDeliveryUpdates();
    await _stopFeedUpdates();
    await super.close();
  }
}
