import 'package:flutter_test/flutter_test.dart';

import '../../../integration_test/support/device_resource_ownership.dart';

void main() {
  test('successful build transfers ownership without releasing', () async {
    var releaseCount = 0;
    final result = await transferDeviceResourceOwnership(
      acquire: () async => Object(),
      build: (_) => 'ready',
      release: (_) async => releaseCount += 1,
    );

    expect(result, 'ready');
    expect(releaseCount, 0);
  });

  test('synchronous build failure releases exactly once', () async {
    var releaseCount = 0;
    final result = transferDeviceResourceOwnership<Object, Object>(
      acquire: () async => Object(),
      build: (_) => throw StateError('sync build failure'),
      release: (_) async => releaseCount += 1,
    );

    await expectLater(result, throwsStateError);
    expect(releaseCount, 1);
  });

  test('asynchronous build failure releases exactly once', () async {
    var releaseCount = 0;
    final result = transferDeviceResourceOwnership<Object, Object>(
      acquire: () async => Object(),
      build: (_) async => throw StateError('async build failure'),
      release: (_) async => releaseCount += 1,
    );

    await expectLater(result, throwsStateError);
    expect(releaseCount, 1);
  });

  test('acquisition failure does not release an absent resource', () async {
    var releaseCount = 0;
    final result = transferDeviceResourceOwnership<Object, Object>(
      acquire: () => throw StateError('acquisition failure'),
      build: (_) => Object(),
      release: (_) async => releaseCount += 1,
    );

    await expectLater(result, throwsStateError);
    expect(releaseCount, 0);
  });
}
