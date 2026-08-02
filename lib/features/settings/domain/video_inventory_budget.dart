enum VideoInventoryBudget {
  twoHundredFiftySixMegabytes(256, '256 MB'),
  oneGigabyte(1024, '1 GB'),
  twoGigabytes(2048, '2 GB'),
  fourGigabytes(4096, '4 GB');

  const VideoInventoryBudget(this.megabytes, this.label);

  final int megabytes;
  final String label;

  int get bytes => megabytes * 1024 * 1024;
}
