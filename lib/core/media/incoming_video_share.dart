import 'package:ghostr/core/media/selected_media.dart';

sealed class IncomingVideoShareEvent {
  const IncomingVideoShareEvent();
}

final class IncomingVideoShareReady extends IncomingVideoShareEvent {
  const IncomingVideoShareReady(this.media);

  final SelectedMedia media;
}

final class IncomingVideoShareFailure extends IncomingVideoShareEvent {
  const IncomingVideoShareFailure(this.message);

  final String message;
}

abstract interface class IncomingVideoSharePort {
  Stream<IncomingVideoShareEvent> get events;

  Future<void> acknowledge(SelectedMedia media);

  Future<void> release(SelectedMedia media);

  Future<void> close();
}
