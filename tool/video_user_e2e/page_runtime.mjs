export async function evaluate(page, expression) {
  const reply = await page.cdp.send("Runtime.evaluate", {
    expression,
    awaitPromise: true,
    returnByValue: true,
  }, page.sessionId);
  if (reply.exceptionDetails) throw new Error(exceptionText(reply.exceptionDetails));
  return reply.result?.value;
}

export function debugSnapshot(page) {
  return evaluate(page, "typeof latestState === 'undefined' ? null : latestState");
}

export function refreshDebugSnapshot(page) {
  return evaluate(page, "refresh().then(() => latestState)");
}

export function playerSnapshot(page) {
  return evaluate(page, `(() => {
    const player = document.getElementById("player");
    return {id: typeof currentId === "undefined" ? null : currentId,
      phase: (typeof playbackPhase === "undefined" ? "unavailable" : playbackPhase).toLowerCase(),
      current_time: Number(player.currentTime), paused: player.paused, ended: player.ended,
      ready_state: player.readyState,
      error: player.error ? {code: player.error.code, message: player.error.message} : null};
  })()`);
}

export function controlPoint(page, id) {
  return evaluate(page, controlExpression(id));
}

export async function dispatchTrustedClick(page, point) {
  await page.cdp.send("Input.dispatchMouseEvent", {
    type: "mouseMoved", x: point.x, y: point.y,
  }, page.sessionId);
  await page.cdp.send("Input.dispatchMouseEvent", button("mousePressed", point), page.sessionId);
  await page.cdp.send("Input.dispatchMouseEvent", button("mouseReleased", point), page.sessionId);
}

export async function captureScreenshot(page) {
  const result = await page.cdp.send(
    "Page.captureScreenshot", {format: "png"}, page.sessionId,
  );
  return result.data;
}

function controlExpression(id) {
  const encoded = JSON.stringify(id);
  return `(() => {
    const id = ${encoded};
    const row = [...document.querySelectorAll(".video-row")]
      .find(item => item.dataset.videoId === id);
    const element = row?.querySelector(".row-play");
    if (!element) return {ready:false, reason:"missing"};
    const rect = element.getBoundingClientRect(); const style = getComputedStyle(element);
    const x = rect.left + rect.width / 2; const y = rect.top + rect.height / 2;
    const top = document.elementFromPoint(x, y); const visible = rect.width > 0 && rect.height > 0
      && style.visibility !== "hidden" && style.display !== "none"
      && (top === element || element.contains(top));
    return {ready:visible && !element.disabled, x, y,
      label:element.getAttribute("aria-label") || element.textContent.trim()};
  })()`;
}

function button(type, point) {
  return {type, x: point.x, y: point.y, button: "left", clickCount: 1};
}

function exceptionText(details) {
  return details.exception?.description || details.text || "browser evaluation failed";
}
