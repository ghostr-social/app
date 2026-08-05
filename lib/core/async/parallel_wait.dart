import 'dart:async';

Future<(A, B)> waitForBoth<A, B>(Future<A> first, Future<B> second) async {
  late A firstValue;
  late B secondValue;
  await Future.wait<void>([
    first.then((value) => firstValue = value),
    second.then((value) => secondValue = value),
  ], eagerError: true);
  return (firstValue, secondValue);
}
