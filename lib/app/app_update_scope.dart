import 'dart:async';

import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:ghostr/app/app_update_lifecycle.dart';
import 'package:ghostr/features/app_update/presentation/app_update_cubit.dart';

typedef AppUpdateCubitFactory = AppUpdateCubit Function();
typedef AppUpdateRuntimeDisposer = Future<void> Function();

class AppUpdateScope extends StatefulWidget {
  const AppUpdateScope({
    required this.create,
    required this.child,
    this.disposeRuntime,
    super.key,
  });

  final AppUpdateCubitFactory create;
  final AppUpdateRuntimeDisposer? disposeRuntime;
  final Widget child;

  static AppUpdateCubit? maybeOf(BuildContext context) {
    return context
        .dependOnInheritedWidgetOfExactType<_AppUpdateAccess>()
        ?.cubit;
  }

  @override
  State<AppUpdateScope> createState() => _AppUpdateScopeState();
}

class _AppUpdateScopeState extends State<AppUpdateScope> {
  late final AppUpdateCubit _cubit;

  @override
  void initState() {
    super.initState();
    _cubit = widget.create();
    unawaited(_cubit.start());
  }

  @override
  void dispose() {
    unawaited(_dispose());
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return _AppUpdateAccess(
      cubit: _cubit,
      child: BlocProvider.value(
        value: _cubit,
        child: AppUpdateLifecycle(
          onResumed: _cubit.onAppResumed,
          child: widget.child,
        ),
      ),
    );
  }

  Future<void> _dispose() async {
    await _cubit.close();
    await widget.disposeRuntime?.call();
  }
}

class _AppUpdateAccess extends InheritedWidget {
  const _AppUpdateAccess({required this.cubit, required super.child});

  final AppUpdateCubit cubit;

  @override
  bool updateShouldNotify(_AppUpdateAccess oldWidget) {
    return cubit != oldWidget.cubit;
  }
}
