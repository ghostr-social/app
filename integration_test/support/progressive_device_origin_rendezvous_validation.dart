part of 'progressive_device_origin.dart';

Set<String> _validate(Set<String> paths) {
  if (paths.length != 2 || paths.any((path) => !path.startsWith('/'))) {
    throw ArgumentError.value(paths, 'paths', 'requires two absolute paths');
  }
  return paths;
}

Duration _validateTimeout(Duration timeout) {
  if (timeout <= Duration.zero) throw ArgumentError.value(timeout);
  return timeout;
}
