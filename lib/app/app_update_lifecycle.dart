import 'dart:async';

import 'package:flutter/widgets.dart';

class AppUpdateLifecycle extends StatefulWidget {
  const AppUpdateLifecycle({
    required this.onResumed,
    required this.child,
    super.key,
  });

  final Future<void> Function() onResumed;
  final Widget child;

  @override
  State<AppUpdateLifecycle> createState() => _AppUpdateLifecycleState();
}

class _AppUpdateLifecycleState extends State<AppUpdateLifecycle>
    with WidgetsBindingObserver {
  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    super.dispose();
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state == AppLifecycleState.resumed) {
      unawaited(widget.onResumed());
    }
  }

  @override
  Widget build(BuildContext context) => widget.child;
}
