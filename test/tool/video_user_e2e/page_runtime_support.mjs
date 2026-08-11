export function recordingPage(reply) {
  const calls = [];
  return {
    calls,
    page: {
      sessionId: "page-session",
      cdp: {
        send: async (method, params, sessionId) => {
          calls.push({method, params, sessionId});
          return typeof reply === "function" ? reply(method) : reply;
        },
      },
    },
  };
}
