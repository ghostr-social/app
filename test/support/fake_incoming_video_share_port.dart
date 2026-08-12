import 'dart:async';

import 'package:ghostr/core/media/incoming_video_share.dart';
import 'package:ghostr/core/media/selected_media.dart';

final class FakeIncomingVideoSharePort implements IncomingVideoSharePort {
  FakeIncomingVideoSharePort({IncomingVideoShareEvent? initialEvent})
    : _initialEvent = initialEvent {
    _events = StreamController<IncomingVideoShareEvent>.broadcast(
      onListen: _emitInitial,
    );
  }

  final IncomingVideoShareEvent? _initialEvent;
  late final StreamController<IncomingVideoShareEvent> _events;
  bool _didEmitInitial = false;
  final acknowledgedMedia = <SelectedMedia>[];
  final releasedMedia = <SelectedMedia>[];
  Object? acknowledgeFailure;
  Object? releaseFailure;
  Future<void>? acknowledgeFuture;
  int closeCalls = 0;

  @override
  Stream<IncomingVideoShareEvent> get events => _events.stream;

  @override
  Future<void> acknowledge(SelectedMedia media) async {
    await acknowledgeFuture;
    if (acknowledgeFailure case final Object failure) throw failure;
    acknowledgedMedia.add(media);
  }

  @override
  Future<void> release(SelectedMedia media) async {
    if (releaseFailure case final Object failure) throw failure;
    releasedMedia.add(media);
  }

  void emit(IncomingVideoShareEvent event) => _events.add(event);

  void _emitInitial() {
    final initialEvent = _initialEvent;
    if (_didEmitInitial || initialEvent == null) return;
    _didEmitInitial = true;
    scheduleMicrotask(() => emit(initialEvent));
  }

  @override
  Future<void> close() async {
    closeCalls += 1;
    if (!_events.isClosed) await _events.close();
  }
}
