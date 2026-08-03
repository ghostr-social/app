import 'dart:io';

import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/video_inventory/domain/media_url_policy.dart';

typedef InternetAddressLookup = Future<List<InternetAddress>> Function(
  String host,
);

class PublicMediaAddressResolver implements MediaUrlPolicy {
  PublicMediaAddressResolver({InternetAddressLookup? lookup})
      : _lookup = lookup ?? InternetAddress.lookup;

  final InternetAddressLookup _lookup;

  @override
  Future<void> validate(Uri source) async {
    await resolve(source);
  }

  Future<InternetAddress> resolve(Uri source) async {
    return (await resolveAll(source)).first;
  }

  Future<List<InternetAddress>> resolveAll(Uri source) async {
    if (!_isHttpUrl(source)) throw const AppFailure(_message);
    final addresses = await _lookup(source.host);
    if (addresses.isEmpty || addresses.any(_isNonPublic)) {
      throw const AppFailure(_message);
    }
    return _unique(addresses);
  }

  List<InternetAddress> _unique(List<InternetAddress> addresses) {
    final seen = <String>{};
    return addresses.where((address) => seen.add(_key(address))).toList();
  }

  String _key(InternetAddress address) => address.rawAddress.join(':');

  bool _isHttpUrl(Uri source) {
    return source.host.isNotEmpty &&
        source.userInfo.isEmpty &&
        (source.scheme == 'http' || source.scheme == 'https');
  }

  bool _isNonPublic(InternetAddress address) {
    if (_hasNonPublicProperty(address)) return true;
    if (address.type == InternetAddressType.unix) return true;
    final embeddedIpv4 = _embeddedIpv4(address.rawAddress);
    if (embeddedIpv4 != null) return _isBlocked(_blockedIpv4, embeddedIpv4);
    if (address.type == InternetAddressType.IPv6 &&
        !_globalIpv6.contains(address.rawAddress)) {
      return true;
    }
    return _isBlocked(_blockedNetworks(address), address.rawAddress);
  }

  bool _hasNonPublicProperty(InternetAddress address) =>
      address.isLoopback || address.isLinkLocal || address.isMulticast;

  List<_AddressNetwork> _blockedNetworks(InternetAddress address) =>
      address.type == InternetAddressType.IPv4 ? _blockedIpv4 : _blockedIpv6;

  List<int>? _embeddedIpv4(List<int> address) {
    if (address.length != 16) return null;
    for (final prefix in _embeddedIpv4Prefixes) {
      if (_startsWith(address, prefix)) return address.sublist(12);
    }
    return null;
  }

  bool _startsWith(List<int> address, List<int> prefix) {
    for (var index = 0; index < prefix.length; index += 1) {
      if (address[index] != prefix[index]) return false;
    }
    return true;
  }

  bool _isBlocked(List<_AddressNetwork> networks, List<int> address) =>
      networks.any((network) => network.contains(address));

  static const _message =
      'Media URLs must resolve to a public internet address.';

  static final _blockedIpv4 = [
    _AddressNetwork('0.0.0.0', 8),
    _AddressNetwork('10.0.0.0', 8),
    _AddressNetwork('100.64.0.0', 10),
    _AddressNetwork('127.0.0.0', 8),
    _AddressNetwork('169.254.0.0', 16),
    _AddressNetwork('172.16.0.0', 12),
    _AddressNetwork('192.0.0.0', 24),
    _AddressNetwork('192.0.2.0', 24),
    _AddressNetwork('192.88.99.0', 24),
    _AddressNetwork('192.168.0.0', 16),
    _AddressNetwork('198.18.0.0', 15),
    _AddressNetwork('198.51.100.0', 24),
    _AddressNetwork('203.0.113.0', 24),
    _AddressNetwork('224.0.0.0', 4),
    _AddressNetwork('240.0.0.0', 4),
  ];

  static final _blockedIpv6 = [
    _AddressNetwork('64:ff9b:1::', 48),
    _AddressNetwork('100::', 64),
    _AddressNetwork('2001::', 23),
    _AddressNetwork('2001:db8::', 32),
    _AddressNetwork('2002::', 16),
    _AddressNetwork('3ffe::', 16),
    _AddressNetwork('3fff::', 20),
    _AddressNetwork('fc00::', 7),
    _AddressNetwork('fe80::', 10),
    _AddressNetwork('fec0::', 10),
    _AddressNetwork('ff00::', 8),
  ];

  static final _globalIpv6 = _AddressNetwork('2000::', 3);

  static const _embeddedIpv4Prefixes = <List<int>>[
    <int>[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    <int>[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 255],
    <int>[0, 100, 255, 155, 0, 0, 0, 0, 0, 0, 0, 0],
  ];
}

class _AddressNetwork {
  _AddressNetwork(String address, this.prefixBits)
      : bytes = InternetAddress(address).rawAddress;

  final List<int> bytes;
  final int prefixBits;

  bool contains(List<int> candidate) {
    if (candidate.length != bytes.length) return false;
    final wholeBytes = prefixBits ~/ 8;
    for (var index = 0; index < wholeBytes; index += 1) {
      if (candidate[index] != bytes[index]) return false;
    }
    final remainingBits = prefixBits % 8;
    if (remainingBits == 0) return true;
    final mask = 0xff << (8 - remainingBits) & 0xff;
    return candidate[wholeBytes] & mask == bytes[wholeBytes] & mask;
  }
}
