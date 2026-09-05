import 'dart:async';

import 'package:flutter/widgets.dart';

final class WarpAndroidLifecycleProbe with WidgetsBindingObserver {
  WarpAndroidLifecycleProbe._(this._binding) {
    final initial = _binding.lifecycleState;
    if (initial != null) states.add(initial);
    _binding.addObserver(this);
  }

  factory WarpAndroidLifecycleProbe.attach() {
    return WarpAndroidLifecycleProbe._(WidgetsBinding.instance);
  }

  final WidgetsBinding _binding;
  final states = <AppLifecycleState>[];
  final _backgrounded = Completer<void>();
  final _resumed = Completer<void>();
  var _sawBackground = false;

  Future<void> get backgrounded => _backgrounded.future;
  bool get hasResumedAfterBackground => _resumed.isCompleted;

  String get evidence => states.map((state) => state.name).join('|');

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    states.add(state);
    if (state == AppLifecycleState.paused) {
      _sawBackground = true;
      if (!_backgrounded.isCompleted) _backgrounded.complete();
      return;
    }
    if (state != AppLifecycleState.resumed || !_sawBackground) return;
    if (!_resumed.isCompleted) _resumed.complete();
  }

  void close() => _binding.removeObserver(this);
}
