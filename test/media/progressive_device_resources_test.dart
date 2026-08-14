import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/progressive_device_resources.dart';

void main() {
  test(
    'closing device journey resources deletes its cache directory',
    () async {
      final resources = await ProgressiveDeviceResources.start();
      final cache = Directory(resources.cachePath);
      expect(cache.existsSync(), isTrue);

      await resources.close();

      expect(cache.existsSync(), isFalse);
    },
  );
}
