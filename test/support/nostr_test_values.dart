const testEventId =
    'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';
const secondTestEventId =
    'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb';
const publishedTestEventId =
    'cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc';
const testViewerPublicKey =
    '7e7e9c42a91bfef19fa929e5fda1b72e0ebc1a4c1141673e2794234d86addf4e';
const testViewerNpub =
    'npub10elfcs4fr0l0r8af98jlmgdh9c8tcxjvz9qkw038js35mp4dma8qzvjptg';
const testCreatorPublicKey =
    '2222222222222222222222222222222222222222222222222222222222222222';
const testAuthorPublicKey =
    '3333333333333333333333333333333333333333333333333333333333333333';
const testFanPublicKey =
    '4444444444444444444444444444444444444444444444444444444444444444';
const testNsec =
    'nsec1vl029mgpspedva04g90vltkh6fvh240zqtv9k0t9af8935ke9laqsnlfe5';

String publishedEventId(int sequence) {
  return sequence.toRadixString(16).padLeft(64, '0');
}
