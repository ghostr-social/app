import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/errors/boundary_failure.dart';
import 'package:ghostr/core/presentation/disposal_safe_cubit.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_sharing/domain/video_share_origin.dart';
import 'package:ghostr/features/video_sharing/domain/video_share_workflow.dart';
import 'package:ghostr/features/video_sharing/presentation/video_share_state.dart';

export 'video_share_state.dart';

final class VideoShareCubit extends DisposalSafeCubit<VideoShareState> {
  VideoShareCubit(this._workflow) : super(const VideoShareIdle());

  final VideoShareWorkflow _workflow;

  bool supports(VideoPost post) => _workflow.supports(post.media);

  Future<void> share(VideoPost post, {required VideoShareOrigin origin}) async {
    if (state is VideoShareInProgress) return;
    emit(VideoShareInProgress(post.id));
    try {
      await _workflow.share(post.media, origin: origin);
      emit(const VideoShareIdle());
    } on AppFailure catch (failure) {
      emit(VideoShareFailed(post.id, failure.message));
    } on Object catch (error, stackTrace) {
      emit(VideoShareFailed(post.id, _unexpected(error, stackTrace)));
    }
  }

  void clearFailure() {
    if (state is VideoShareFailed) emit(const VideoShareIdle());
  }
}

String _unexpected(Object error, StackTrace stackTrace) {
  return translatedBoundaryFailure(
    source: 'VideoShareCubit.share',
    message: 'Could not share this video.',
    error: error,
    stackTrace: stackTrace,
  ).message;
}
