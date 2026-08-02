import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

void main() {
  test('uses the declared Rust plugin identity on Apple platforms', () {
    const pluginName = 'rust_lib_ghostr';
    final plugin = File('rust_builder/pubspec.yaml').readAsStringSync();
    final ios = File('rust_builder/ios/$pluginName.podspec');
    final macos = File('rust_builder/macos/$pluginName.podspec');

    expect(plugin, contains('name: $pluginName'));
    expect(ios.existsSync(), isTrue);
    expect(macos.existsSync(), isTrue);
    expect(
        ios.readAsStringSync(), contains("s.name             = '$pluginName'"));
    expect(macos.readAsStringSync(),
        contains("s.name             = '$pluginName'"));
  });
}
