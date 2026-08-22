enum DeliveryNetworkClass { unavailable, wifi, cellular, wired, constrained }

final class DeliveryNetworkStatus {
  const DeliveryNetworkStatus(this.networkClass, {required this.generation})
    : assert(generation >= 0);

  static const unavailable = DeliveryNetworkStatus(
    DeliveryNetworkClass.unavailable,
    generation: 0,
  );

  final DeliveryNetworkClass networkClass;
  final int generation;

  bool isFresherThan(DeliveryNetworkStatus other) {
    return generation > other.generation;
  }

  @override
  bool operator ==(Object other) {
    return other is DeliveryNetworkStatus &&
        other.networkClass == networkClass &&
        other.generation == generation;
  }

  @override
  int get hashCode => Object.hash(networkClass, generation);
}
