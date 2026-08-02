import 'package:flutter/material.dart';
import 'package:ghostr/app/build_production_dependencies.dart';
import 'package:ghostr/app/startup_gate.dart';

typedef GhostrRunner = void Function(Widget root);

Future<void> main() => launchGhostr(runApp); // coverage:ignore-line

Future<void> launchGhostr(GhostrRunner runner) async {
  WidgetsFlutterBinding.ensureInitialized();
  runner(const StartupGate(
    loadDependencies: buildProductionDependencies,
  ));
}
