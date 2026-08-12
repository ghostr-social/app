import 'dart:async';

import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/profile/domain/profile_metadata_repository.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/video_catalog/presentation/profile_failure_messages.dart';
import 'package:ghostr/features/video_catalog/presentation/profile_request.dart';

typedef ProfileMetadataAccepted = void Function(ProfileSummary profile);
typedef ProfileMetadataRejected = void Function(String message);

final class ProfileMetadataRefresh {
  ProfileMetadataRefresh(this._repository);

  final ProfileMetadataRepository? _repository;
  int _generation = 0;

  void start(
    ProfileRequest request, {
    required ProfileMetadataAccepted onAccepted,
    required ProfileMetadataRejected onRejected,
  }) {
    final generation = ++_generation;
    final repository = _repository;
    if (repository == null || request.viewer.id != request.profileId) return;
    final callbacks = _ProfileMetadataCallbacks(onAccepted, onRejected);
    unawaited(_complete(repository, request, generation, callbacks));
  }

  void cancel() => _generation += 1;

  Future<void> _complete(
    ProfileMetadataRepository repository,
    ProfileRequest request,
    int generation,
    _ProfileMetadataCallbacks callbacks,
  ) async {
    try {
      final refreshed = await repository.refresh(request.profileId);
      if (_isCurrent(generation, refreshed, request)) {
        callbacks.accepted(refreshed!);
      }
    } on AppFailure catch (failure) {
      if (generation == _generation) callbacks.rejected(failure.message);
    } on Object catch (error, stackTrace) {
      if (generation == _generation) {
        callbacks.rejected(unexpectedProfileMetadataFailure(error, stackTrace));
      }
    }
  }

  bool _isCurrent(
    int generation,
    ProfileSummary? refreshed,
    ProfileRequest request,
  ) {
    return generation == _generation && refreshed?.id == request.profileId;
  }
}

final class _ProfileMetadataCallbacks {
  const _ProfileMetadataCallbacks(this.accepted, this.rejected);

  final ProfileMetadataAccepted accepted;
  final ProfileMetadataRejected rejected;
}
