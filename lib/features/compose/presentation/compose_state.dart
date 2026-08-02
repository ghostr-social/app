import 'package:ghostr/core/media/selected_media.dart';

enum ComposeStatus { idle, selecting, publishing }

class ComposeState {
  const ComposeState._({
    this.media,
    this.status = ComposeStatus.idle,
    this.errorMessage,
    this.notice,
  });

  const ComposeState.idle() : this._();

  final SelectedMedia? media;
  final ComposeStatus status;
  final String? errorMessage;
  final String? notice;

  bool get isPublishing => status == ComposeStatus.publishing;
  bool get isSelecting => status == ComposeStatus.selecting;
  bool get isBusy => status != ComposeStatus.idle;

  ComposeState selecting() {
    return ComposeState._(media: media, status: ComposeStatus.selecting);
  }

  ComposeState selectionFinished() {
    return ComposeState._(media: media);
  }

  ComposeState selected(SelectedMedia selectedMedia) {
    return ComposeState._(media: selectedMedia);
  }

  ComposeState publishing() {
    if (media == null) throw StateError('Cannot publish without media.');
    return ComposeState._(media: media, status: ComposeStatus.publishing);
  }

  ComposeState failed(String message) {
    return ComposeState._(media: media, errorMessage: message);
  }

  ComposeState published(String message) {
    return ComposeState._(notice: message);
  }

  ComposeState withoutNotice() => const ComposeState.idle();
}
