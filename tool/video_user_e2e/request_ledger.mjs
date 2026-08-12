export class RequestLedger {
  #limit;
  #entries = [];
  #byId = new Map();

  constructor({limit = 4_096} = {}) {
    this.#limit = limit;
  }

  get entries() {
    return this.#entries.map((entry) => ({...entry}));
  }

  request(params) {
    const entry = {
      request_id: params.requestId,
      url: params.request.url,
      method: params.request.method,
      range: header(params.request.headers, "range"),
      started_at: params.timestamp,
    };
    dropUndefined(entry);
    this.#entries.push(entry);
    this.#byId.set(params.requestId, entry);
    this.#trim();
  }

  response(params) {
    const entry = this.#byId.get(params.requestId);
    if (!entry) return;
    Object.assign(entry, {
      status: params.response.status,
      content_range: header(params.response.headers, "content-range"),
      mime_type: params.response.mimeType,
      response_at: params.timestamp,
    });
    dropUndefined(entry);
  }

  finished(params) {
    const entry = this.#byId.get(params.requestId);
    if (!entry) return;
    entry.encoded_bytes = params.encodedDataLength;
    entry.finished = true;
  }

  failed(params) {
    const entry = this.#byId.get(params.requestId);
    if (!entry) return;
    entry.failure = params.errorText;
    entry.canceled = params.canceled === true;
  }

  #trim() {
    while (this.#entries.length > this.#limit) {
      const removed = this.#entries.shift();
      if (this.#byId.get(removed.request_id) === removed) {
        this.#byId.delete(removed.request_id);
      }
    }
  }
}

export function attachRequestLedger(cdp, ledger) {
  cdp.on("Network.requestWillBeSent", (event) => ledger.request(event.params));
  cdp.on("Network.responseReceived", (event) => ledger.response(event.params));
  cdp.on("Network.loadingFinished", (event) => ledger.finished(event.params));
  cdp.on("Network.loadingFailed", (event) => ledger.failed(event.params));
}

function header(headers = {}, name) {
  const key = Object.keys(headers).find((candidate) => candidate.toLowerCase() === name);
  return key ? headers[key] : undefined;
}

function dropUndefined(value) {
  for (const key of Object.keys(value)) {
    if (value[key] === undefined) delete value[key];
  }
}
