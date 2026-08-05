/// A failed feed operation, retained without deciding how a UI should
/// describe it.
final class FeedOperationFailure {
  const FeedOperationFailure(this.cause, this.stackTrace);

  final Object cause;
  final StackTrace stackTrace;
}
