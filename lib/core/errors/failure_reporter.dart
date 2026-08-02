abstract interface class FailureReporter {
  void report({
    required String source,
    required Object error,
    required StackTrace stackTrace,
  });
}
