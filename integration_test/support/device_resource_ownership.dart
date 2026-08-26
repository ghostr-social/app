import 'dart:async';

Future<Result> transferDeviceResourceOwnership<Resource, Result>({
  required Future<Resource> Function() acquire,
  required FutureOr<Result> Function(Resource resource) build,
  required FutureOr<void> Function(Resource resource) release,
}) async {
  final resource = await acquire();
  try {
    return await Future<Result>.sync(() => build(resource));
  } on Object {
    await release(resource);
    rethrow;
  }
}
