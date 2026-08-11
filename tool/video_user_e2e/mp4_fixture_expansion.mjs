const SCALAR_TIMING_FIELDS = [
  {type: "mvhd", offset: 24, signed: false},
  {type: "tkhd", offset: 28, signed: false},
  {type: "elst", offset: 16, signed: false},
  {type: "elst", offset: 20, signed: true},
  {type: "mdhd", offset: 24, signed: false},
];
const TIMING_TABLES = ["stts", "ctts"];

export function expandAvcSamples(input, extraBytes) {
  const layout = readLayout(input);
  const output = Buffer.alloc(input.length + layout.sampleCount * extraBytes);
  input.copy(output, 0, 0, layout.payloadStart);
  const offsets = copySamples({input, output, layout, extraBytes});
  input.copy(output, offsets.target, layout.payloadEnd);
  output.writeUInt32BE(layout.mdatSize + layout.sampleCount * extraBytes, layout.mdatStart);
  return output;
}

export function scaleAvcTiming(input, multiplier) {
  requirePositiveInteger(multiplier);
  const output = Buffer.from(input);
  for (const field of SCALAR_TIMING_FIELDS) {
    scaleScalar(output, field, multiplier);
  }
  for (const type of TIMING_TABLES) scaleTable(output, type, multiplier);
  return output;
}

function readLayout(bytes) {
  const stszType = bytes.indexOf(Buffer.from("stsz"));
  const mdatType = bytes.indexOf(Buffer.from("mdat"));
  const mdatStart = mdatType - 4;
  const mdatSize = bytes.readUInt32BE(mdatStart);
  return {
    sampleCount: bytes.readUInt32BE(stszType + 12),
    sampleEntries: stszType + 16,
    mdatStart,
    mdatSize,
    payloadStart: mdatType + 4,
    payloadEnd: mdatStart + mdatSize,
  };
}

function copySamples(input) {
  let source = input.layout.payloadStart;
  let target = input.layout.payloadStart;
  for (let index = 0; index < input.layout.sampleCount; index += 1) {
    const entry = input.layout.sampleEntries + index * 4;
    const size = input.input.readUInt32BE(entry);
    input.input.copy(input.output, target, source, source + size);
    appendFiller(input.output, target + size, input.extraBytes);
    input.output.writeUInt32BE(size + input.extraBytes, entry);
    source += size;
    target += size + input.extraBytes;
  }
  return {source, target};
}

function appendFiller(output, offset, byteLength) {
  const nalSize = byteLength - 4;
  output.writeUInt32BE(nalSize, offset);
  output[offset + 4] = 0x0c;
  output.fill(0xff, offset + 5, offset + 4 + nalSize - 1);
  output[offset + 4 + nalSize - 1] = 0x80;
}

function scaleScalar(bytes, field, multiplier) {
  const offset = boxStart(bytes, field.type) + field.offset;
  const value = field.signed ? bytes.readInt32BE(offset) : bytes.readUInt32BE(offset);
  if (field.signed) bytes.writeInt32BE(value * multiplier, offset);
  else bytes.writeUInt32BE(value * multiplier, offset);
}

function scaleTable(bytes, type, multiplier) {
  const start = boxStart(bytes, type);
  const count = bytes.readUInt32BE(start + 12);
  for (let index = 0; index < count; index += 1) {
    const offset = start + 20 + index * 8;
    bytes.writeUInt32BE(bytes.readUInt32BE(offset) * multiplier, offset);
  }
}

function boxStart(bytes, type) {
  const typeOffset = bytes.indexOf(Buffer.from(type));
  if (typeOffset < 4) throw new Error(`missing ${type} box`);
  return typeOffset - 4;
}

function requirePositiveInteger(value) {
  if (!Number.isSafeInteger(value) || value < 1) {
    throw new RangeError("timing multiplier must be a positive integer");
  }
}
