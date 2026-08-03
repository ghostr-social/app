import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/errors/boundary_failure.dart';
import 'package:ghostr/core/media/media_picker_port.dart';
import 'package:ghostr/core/media/media_picker_capabilities.dart';
import 'package:ghostr/core/media/selected_media.dart';
import 'package:ghostr/core/presentation/disposal_safe_cubit.dart';
import 'package:ghostr/features/compose/domain/publish_video_workflow.dart';
import 'package:ghostr/features/compose/presentation/compose_state.dart';
import 'package:ghostr/features/session/domain/user_session.dart';

export 'compose_state.dart';

class ComposeDependencies {
  const ComposeDependencies({
    required this.publishVideo,
    required this.mediaPicker,
  });

  final PublishVideoWorkflow publishVideo;
  final MediaPickerPort mediaPicker;
}

class ComposeCubit extends DisposalSafeCubit<ComposeState> {
  ComposeCubit(this._dependencies) : super(const ComposeState.idle());

  final ComposeDependencies _dependencies;

  MediaPickerCapabilities get pickerCapabilities =>
      _dependencies.mediaPicker.capabilities;

  Future<void> recoverLostVideo() {
    return _select(_dependencies.mediaPicker.recoverLostVideo);
  }

  Future<void> captureVideo() {
    return _select(_dependencies.mediaPicker.captureVideo);
  }

  Future<void> chooseFromGallery() {
    return _select(_dependencies.mediaPicker.pickFromGallery);
  }

  Future<bool> publish(UserSession session, String rawCaption) async {
    if (state.isBusy) return false;
    final media = state.media;
    if (media == null) return false;
    emit(state.publishing());
    try {
      final outcome = await _publish(session, media, rawCaption);
      emit(state.published(_publishNotice(outcome)));
      return true;
    } on AppFailure catch (failure) {
      return _reject(failure.message);
    } on Object catch (error, stackTrace) {
      return _reject(_unexpectedPublish(error, stackTrace));
    }
  }

  void clearNotice() {
    if (state.notice != null) emit(state.withoutNotice());
  }

  Future<void> _select(Future<SelectedMedia?> Function() operation) async {
    if (state.isBusy) return;
    emit(state.selecting());
    try {
      final media = await operation();
      emit(media == null ? state.selectionFinished() : state.selected(media));
    } on AppFailure catch (failure) {
      emit(state.failed(failure.message));
    } on Object catch (error, stackTrace) {
      emit(state.failed(_unexpectedSelection(error, stackTrace)));
    }
  }

  String _publishNotice(PublishVideoOutcome outcome) {
    final catalog = outcome.warnings.contains(
      PublishVideoWarning.localCatalogUnavailable,
    );
    final activity = outcome.warnings.contains(
      PublishVideoWarning.localActivityUnavailable,
    );
    if (catalog && activity) {
      return 'Published, but your local video list and activity history '
          'could not be updated.';
    }
    if (catalog) {
      return 'Published, but your local video list could not be updated.';
    }
    if (activity) {
      return 'Published, but local activity history could not be updated.';
    }
    return 'Published to your Ghostr profile.';
  }

  Future<PublishVideoOutcome> _publish(
    UserSession session,
    SelectedMedia media,
    String caption,
  ) {
    return _dependencies.publishVideo.publish(
      session: session,
      media: media,
      rawCaption: caption,
    );
  }

  bool _reject(String message) {
    emit(state.failed(message));
    return false;
  }

  String _unexpectedPublish(Object error, StackTrace stackTrace) {
    return translatedBoundaryFailure(
      source: 'ComposeCubit.publish',
      message: 'Could not publish this video.',
      error: error,
      stackTrace: stackTrace,
    ).message;
  }

  String _unexpectedSelection(Object error, StackTrace stackTrace) {
    return translatedBoundaryFailure(
      source: 'ComposeCubit.select',
      message: 'Could not open this video.',
      error: error,
      stackTrace: stackTrace,
    ).message;
  }
}
