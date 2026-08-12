export function writeResponseChunk(response, bytes, onAccepted = () => {}) {
  if (response.destroyed) return false;
  const writable = response.write(bytes);
  onAccepted();
  if (writable) return true;
  return new Promise((resolve) => {
    const done = (writable) => {
      response.off("drain", drained);
      response.off("close", closed);
      resolve(writable);
    };
    const drained = () => done(true);
    const closed = () => done(false);
    response.once("drain", drained);
    response.once("close", closed);
  });
}
