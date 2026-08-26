import 'dart:async';

import 'package:flutter/material.dart';
import 'package:ghostr/shared/widgets/async_state_panel.dart';
import 'package:ghostr/shared/widgets/loading_panel.dart';

import 'warp_lab_app.dart';
import 'warp_lab_destination.dart';
import 'warp_lab_menu.dart';
import 'warp_lab_session.dart';
import 'warp_lab_unknown_route.dart';

class WarpLabBootstrap extends StatefulWidget {
  const WarpLabBootstrap({
    required this.initialRoute,
    required this.loadSession,
    super.key,
  });

  final String initialRoute;
  final WarpLabSessionLoader loadSession;

  @override
  State<WarpLabBootstrap> createState() => _WarpLabBootstrapState();
}

class _WarpLabBootstrapState extends State<WarpLabBootstrap> {
  WarpLabDestination? _destination;
  Future<WarpLabSession>? _pending;
  var _startRequested = false;

  @override
  void initState() {
    super.initState();
    _destination = WarpLabDestination.fromPath(widget.initialRoute);
    if (_destination case final destination?
        when destination != WarpLabDestination.menu) {
      _pending = _start(destination);
    }
  }

  @override
  Widget build(BuildContext context) {
    final destination = _destination;
    if (destination == null) return _unknownRoute();
    if (destination == WarpLabDestination.menu) return _menu();
    return FutureBuilder<WarpLabSession>(
      future: _pending,
      builder: (context, snapshot) => _sessionContent(destination, snapshot),
    );
  }

  Widget _sessionContent(
    WarpLabDestination destination,
    AsyncSnapshot<WarpLabSession> snapshot,
  ) {
    if (snapshot.data case final session?) {
      return WarpLabApp(home: session.screen(destination));
    }
    if (snapshot.hasError) return _failure();
    return _loading(destination);
  }

  Widget _loading(WarpLabDestination destination) {
    return WarpLabApp(
      home: Scaffold(
        body: LoadingPanel(label: 'Starting ${destination.title}'),
      ),
    );
  }

  Widget _failure() {
    return WarpLabApp(
      home: const Scaffold(
        body: AsyncStatePanel(
          icon: Icons.warning_amber_rounded,
          title: 'WARP Lab could not start',
          message: 'Stop the lab process and relaunch this test route.',
        ),
      ),
    );
  }

  Widget _menu() => WarpLabMenuApp(onOpen: _open);

  Widget _unknownRoute() {
    return WarpLabUnknownRouteApp(
      route: widget.initialRoute,
      onOpenLab: _showMenu,
    );
  }

  void _open(WarpLabDestination destination) {
    if (_startRequested) return;
    final pending = _start(destination);
    setState(() {
      _destination = destination;
      _pending = pending;
    });
  }

  Future<WarpLabSession> _start(WarpLabDestination destination) {
    _startRequested = true;
    return Future.sync(() => widget.loadSession(destination));
  }

  void _showMenu() {
    setState(() => _destination = WarpLabDestination.menu);
  }

  @override
  void dispose() {
    final pending = _pending;
    if (pending != null) unawaited(_closeWhenReady(pending));
    super.dispose();
  }

  Future<void> _closeWhenReady(Future<WarpLabSession> pending) async {
    try {
      final session = await pending;
      await session.close();
    } on Object {
      // The lab is already gone, so startup and cleanup errors have no UI owner.
    }
  }
}
