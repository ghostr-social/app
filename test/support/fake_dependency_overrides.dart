import 'package:ghostr/app/production_app_update.dart';
import 'package:ghostr/features/profile/domain/profile_metadata_repository.dart';
import 'package:ghostr/features/publish/domain/video_publishing_repository.dart';
import 'package:ghostr/features/session/domain/account_provisioning_repository.dart';
import 'package:ghostr/features/session/domain/nostr_account_generator.dart';
import 'package:ghostr/features/session/domain/session_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_updates.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_repository.dart';
import 'package:ghostr/features/video_inventory/domain/playback_preparation_updates.dart';

final class FakeDependencyOverrides {
  const FakeDependencyOverrides({
    this.sessionRepository,
    this.feed,
    this.feedUpdates,
    this.watchHistory,
    this.publishing,
    this.appUpdateRuntime,
    this.accountGenerator,
    this.accountProvisioningRepository,
    this.profileMetadataRepository,
    this.preparationUpdates,
  });

  final SessionRepository? sessionRepository;
  final VideoFeedRepository? feed;
  final VideoFeedUpdates? feedUpdates;
  final WatchHistoryRepository? watchHistory;
  final VideoPublishingRepository? publishing;
  final AppUpdateRuntime? appUpdateRuntime;
  final NostrAccountGenerator? accountGenerator;
  final AccountProvisioningRepository? accountProvisioningRepository;
  final ProfileMetadataRepository? profileMetadataRepository;
  final PlaybackPreparationUpdates? preparationUpdates;
}
