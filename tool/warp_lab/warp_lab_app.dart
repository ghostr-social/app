import 'package:flutter/material.dart';
import 'package:ghostr/shared/theme/app_theme.dart';

class WarpLabApp extends StatefulWidget {
  const WarpLabApp({required this.home, super.key});

  final Widget home;

  @override
  State<WarpLabApp> createState() => _WarpLabAppState();
}

class _WarpLabAppState extends State<WarpLabApp> {
  late final _home = ValueNotifier<Widget>(widget.home);

  @override
  void didUpdateWidget(covariant WarpLabApp oldWidget) {
    super.didUpdateWidget(oldWidget);
    _home.value = widget.home;
  }

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      debugShowCheckedModeBanner: false,
      theme: buildAppTheme(),
      onGenerateRoute: _route,
      onGenerateInitialRoutes: _initialRoutes,
    );
  }

  List<Route<void>> _initialRoutes(String _) {
    return [_route(const RouteSettings(name: '/'))];
  }

  Route<void> _route(RouteSettings settings) {
    return MaterialPageRoute<void>(
      settings: settings,
      builder: (_) => ValueListenableBuilder<Widget>(
        valueListenable: _home,
        builder: (_, home, _) => home,
      ),
    );
  }

  @override
  void dispose() {
    _home.dispose();
    super.dispose();
  }
}
