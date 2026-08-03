abstract interface class MediaUrlPolicy {
  Future<void> validate(Uri source);
}
