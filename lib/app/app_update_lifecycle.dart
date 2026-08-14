import 'dart:async';

import 'package:flutter/widgets.dart';

class AppUpdateLifecycle extends StatefulWidget {
  const AppUpdateLifecycle({
    required this.onResumed,
    required this.onCheckDue,
    required this.child,
    this.checkInterval = const Duration(minutes: 1),
    super.key,
  });

  final Future<void> Function() onResumed;
  final Future<void> Function() onCheckDue;
  final Duration checkInterval;
  final Widget child;

  @override
  State<AppUpdateLifecycle> createState() => _AppUpdateLifecycleState();
}

class _AppUpdateLifecycleState extends State<AppUpdateLifecycle>
    with WidgetsBindingObserver {
  Timer? _periodicCheck;
  bool _isActive = true;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
    _isActive = _initiallyActive();
    if (_isActive) _startPeriodicChecks();
  }

  @override
  void didUpdateWidget(AppUpdateLifecycle oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (_isActive && oldWidget.checkInterval != widget.checkInterval) {
      _startPeriodicChecks();
    }
  }

  @override
  void dispose() {
    _periodicCheck?.cancel();
    WidgetsBinding.instance.removeObserver(this);
    super.dispose();
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state == AppLifecycleState.resumed) {
      _isActive = true;
      _startPeriodicChecks();
      unawaited(widget.onResumed());
    } else {
      _isActive = false;
      _periodicCheck?.cancel();
    }
  }

  bool _initiallyActive() {
    final state = WidgetsBinding.instance.lifecycleState;
    return state == null || state == AppLifecycleState.resumed;
  }

  void _startPeriodicChecks() {
    _periodicCheck?.cancel();
    _periodicCheck = Timer.periodic(widget.checkInterval, (_) {
      unawaited(widget.onCheckDue());
    });
  }

  @override
  Widget build(BuildContext context) => widget.child;
}
