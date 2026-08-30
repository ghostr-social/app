part of 'feed_state.dart';

final class FeedHlsReadiness {
  const FeedHlsReadiness._(this._authorities, this._verified);

  factory FeedHlsReadiness.empty() => const FeedHlsReadiness._({}, {});

  final Map<PlaybackDeliveryId, HlsPlaybackAuthority> _authorities;
  final Set<HlsPlaybackAuthority> _verified;

  Set<HlsPlaybackAuthority> get verified => Set.unmodifiable(_verified);

  HlsPlaybackAuthority? authorityFor(VideoMediaSource media) {
    if (media.remoteDelivery != VideoMediaDelivery.hls) return null;
    final deliveryId = media.playbackDeliveryId;
    if (deliveryId == null) return null;
    final authority = _authorities[deliveryId];
    if (authority == null) return null;
    return authority.representationId == VideoRepresentationId.forMedia(media)
        ? authority
        : null;
  }

  bool isVerified(HlsPlaybackAuthority authority) {
    return _authorities[authority.deliveryId] == authority &&
        _verified.contains(authority);
  }

  FeedHlsReadiness withAuthority(
    PlaybackDeliveryId deliveryId,
    HlsPlaybackAuthority? authority,
  ) {
    final previous = _authorities[deliveryId];
    if (previous == authority) return this;
    final authorities = Map<PlaybackDeliveryId, HlsPlaybackAuthority>.of(
      _authorities,
    );
    authority == null
        ? authorities.remove(deliveryId)
        : authorities[deliveryId] = authority;
    final verified = Set<HlsPlaybackAuthority>.of(_verified);
    if (previous != null) verified.remove(previous);
    return FeedHlsReadiness._(
      Map.unmodifiable(authorities),
      Set.unmodifiable(verified),
    );
  }

  FeedHlsReadiness verify(HlsPlaybackAuthority authority) {
    if (_authorities[authority.deliveryId] != authority ||
        _verified.contains(authority)) {
      return this;
    }
    final verified = Set<HlsPlaybackAuthority>.of(_verified)
      ..removeWhere((known) => known.deliveryId == authority.deliveryId)
      ..add(authority);
    return FeedHlsReadiness._(_authorities, Set.unmodifiable(verified));
  }

  FeedHlsReadiness release(HlsPlaybackAuthority authority) {
    if (!_verified.contains(authority)) return this;
    final verified = Set<HlsPlaybackAuthority>.of(_verified)..remove(authority);
    return FeedHlsReadiness._(_authorities, Set.unmodifiable(verified));
  }
}

extension FeedLoadedHlsReadiness on FeedLoaded {
  HlsPlaybackAuthority? hlsAuthorityFor(VideoMediaSource media) {
    return _presentation.hls.authorityFor(media);
  }

  bool isHlsPlayerVerified(HlsPlaybackAuthority authority) {
    return _presentation.hls.isVerified(authority);
  }

  Set<HlsPlaybackAuthority> get verifiedHlsAuthorities {
    return _presentation.hls.verified;
  }

  FeedLoaded withHlsAuthority(
    PlaybackDeliveryId deliveryId,
    HlsPlaybackAuthority? authority,
  ) {
    final accepted = _acceptAuthority(deliveryId, authority);
    final hls = _presentation.hls.withAuthority(deliveryId, accepted);
    return identical(hls, _presentation.hls) ? this : _withHls(hls);
  }

  FeedLoaded withVerifiedHls(HlsPlaybackAuthority authority, bool verified) {
    final hls = verified
        ? _presentation.hls.verify(authority)
        : _presentation.hls.release(authority);
    return identical(hls, _presentation.hls) ? this : _withHls(hls);
  }

  HlsPlaybackAuthority? _acceptAuthority(
    PlaybackDeliveryId deliveryId,
    HlsPlaybackAuthority? authority,
  ) {
    if (authority == null || authority.deliveryId != deliveryId) return null;
    for (final post in posts) {
      final media = post.media;
      if (media.playbackDeliveryId == deliveryId &&
          media.remoteDelivery == VideoMediaDelivery.hls &&
          VideoRepresentationId.forMedia(media) == authority.representationId) {
        return authority;
      }
    }
    return null;
  }

  FeedLoaded _withHls(FeedHlsReadiness hls) {
    return FeedLoaded._(
      kind,
      posts,
      activeIndex,
      rosterRevision,
      _presentation.withHls(hls),
    );
  }
}
