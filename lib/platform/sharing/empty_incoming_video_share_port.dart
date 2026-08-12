import 'package:ghostr/core/media/incoming_video_share.dart';
import 'package:ghostr/core/media/selected_media.dart';

final class EmptyIncomingVideoSharePort implements IncomingVideoSharePort {
  const EmptyIncomingVideoSharePort();

  @override
  Stream<IncomingVideoShareEvent> get events => const Stream.empty();

  @override
  Future<void> acknowledge(SelectedMedia media) async {}

  @override
  Future<void> release(SelectedMedia media) async {}

  @override
  Future<void> close() async {}
}
