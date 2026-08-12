/// Holds the device screen awake on behalf of media playback.
///
/// Implementations must never throw into playback, synchronously or through
/// the returned future; platform failures are absorbed and logged.
abstract interface class ScreenAwakePort {
  Future<void> enable();

  Future<void> disable();
}
