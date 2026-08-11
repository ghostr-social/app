import {mkdir, writeFile} from "node:fs/promises";
import {basename, join} from "node:path";

export class ArtifactStore {
  constructor({directory, jsonLimit = 1_048_576}) {
    this.directory = directory;
    this.jsonLimit = jsonLimit;
  }

  async writeJson(name, value) {
    requireLeafName(name);
    await mkdir(this.directory, {recursive: true});
    await writeFile(join(this.directory, name), boundedJson(value, this.jsonLimit));
  }

  async writeText(name, value, limit = 524_288) {
    requireLeafName(name);
    await mkdir(this.directory, {recursive: true});
    const bytes = Buffer.from(String(value));
    await writeFile(join(this.directory, name), bytes.subarray(Math.max(0, bytes.length - limit)));
  }

  async writeBase64(name, value, limit = 2_097_152) {
    requireLeafName(name);
    const bytes = Buffer.from(value, "base64");
    if (bytes.length > limit) throw new Error(`artifact exceeds ${limit} byte budget`);
    await mkdir(this.directory, {recursive: true});
    await writeFile(join(this.directory, name), bytes);
  }
}

export class TextRing {
  constructor(limit = 524_288) {
    this.limit = limit;
    this.bytes = Buffer.alloc(0);
  }

  append(value) {
    this.bytes = Buffer.concat([this.bytes, Buffer.from(value)]);
    if (this.bytes.length > this.limit) {
      this.bytes = this.bytes.subarray(this.bytes.length - this.limit);
    }
  }

  toString() {
    return this.bytes.toString("utf8");
  }
}

function boundedJson(value, limit) {
  const body = encode(value);
  if (Buffer.byteLength(body) <= limit) return body;
  if (Array.isArray(value)) return boundedArray(value, limit);
  return encode({truncated: true, original_bytes: Buffer.byteLength(body)});
}

function boundedArray(values, limit) {
  let low = 0;
  let high = values.length;
  while (low < high) {
    const middle = Math.ceil((low + high) / 2);
    if (fits(arrayEnvelope(values, middle), limit)) low = middle;
    else high = middle - 1;
  }
  const body = encode(arrayEnvelope(values, low));
  if (Buffer.byteLength(body) > limit) throw new Error("artifact byte budget is too small");
  return body;
}

function arrayEnvelope(values, count) {
  return {truncated: true, total: values.length, entries: values.slice(0, count)};
}

function fits(value, limit) {
  return Buffer.byteLength(encode(value)) <= limit;
}

function encode(value) {
  return `${JSON.stringify(value, null, 2)}\n`;
}

function requireLeafName(name) {
  if (!name || basename(name) !== name) throw new Error("artifact name must be a file name");
}
