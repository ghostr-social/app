import 'dart:async';
import 'dart:developer';

import 'package:flutter/material.dart';
import 'package:ghostr/app/app_dependencies.dart';
import 'package:ghostr/app/ghostr_app.dart';
import 'package:ghostr/shared/theme/app_theme.dart';
import 'package:ghostr/shared/widgets/async_state_panel.dart';
import 'package:ghostr/shared/widgets/loading_panel.dart';

typedef AppDependenciesLoader = Future<AppDependencies> Function();

class StartupGate extends StatefulWidget {
  const StartupGate({required this.loadDependencies, super.key});

  final AppDependenciesLoader loadDependencies;

  @override
  State<StartupGate> createState() => _StartupGateState();
}

class _StartupGateState extends State<StartupGate> {
  late Future<AppDependencies> _dependencies = _load();
  AppDependencies? _ownedDependencies;
  bool _isDisposed = false;

  @override
  Widget build(BuildContext context) {
    return FutureBuilder<AppDependencies>(
      future: _dependencies,
      builder: (context, snapshot) => _content(snapshot),
    );
  }

  Widget _content(AsyncSnapshot<AppDependencies> snapshot) {
    if (snapshot.data case final dependencies?) {
      return GhostrApp(dependencies: dependencies);
    }
    return MaterialApp(
      debugShowCheckedModeBanner: false,
      theme: buildAppTheme(),
      home: Scaffold(body: snapshot.hasError ? _error() : _loading()),
    );
  }

  Widget _loading() {
    return const LoadingPanel(label: 'Starting Ghostr');
  }

  Widget _error() {
    return AsyncStatePanel(
      icon: Icons.cloud_off,
      title: 'Ghostr could not start',
      message: 'Local settings or secure services could not be initialized.',
      actionLabel: 'Retry',
      onAction: _retry,
    );
  }

  Future<AppDependencies> _load() async {
    try {
      final dependencies = await widget.loadDependencies();
      if (_isDisposed) {
        await dependencies.close();
      } else {
        _ownedDependencies = dependencies;
      }
      return dependencies;
    } catch (error, stackTrace) {
      log(
        'Application startup failed.',
        name: 'ghostr.startup',
        error: error,
        stackTrace: stackTrace,
      );
      rethrow;
    }
  }

  void _retry() {
    setState(() {
      _dependencies = _load();
    });
  }

  @override
  void dispose() {
    _isDisposed = true;
    final dependencies = _ownedDependencies;
    _ownedDependencies = null;
    if (dependencies != null) unawaited(dependencies.close());
    super.dispose();
  }
}
