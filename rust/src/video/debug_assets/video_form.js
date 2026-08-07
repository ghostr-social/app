function optionalNumber(id, multiplier = 1) {
  const raw = byId(id).value.trim();
  return raw ? Math.round(Number(raw) * multiplier) : null;
}

function registration() {
  return {
    url: byId("video-url").value.trim(),
    size_bytes: optionalNumber("video-size"),
    duration_ms: optionalNumber("video-duration", 1000),
  };
}

async function addVideo(event) {
  event.preventDefault();
  const button = event.currentTarget.querySelector("button");
  const status = byId("video-form-status");
  button.disabled = true;
  status.textContent = "Registering video…";
  try {
    const response = await fetch("/debug/api/videos", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify(registration()),
    });
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    const added = await response.json();
    status.textContent = `Added ${added.id}. The Rust downloader is now planning it.`;
    byId("video-url").value = "";
    await refresh();
    byId("video-modal").close();
  } catch (error) {
    status.textContent = `Could not add video: ${error.message}`;
  } finally {
    button.disabled = false;
  }
}

const videoModal = byId("video-modal");
byId("add-video-button").addEventListener("click", () => videoModal.showModal());
byId("video-form").addEventListener("submit", addVideo);
videoModal.querySelector("[data-close-video]").addEventListener("click", () => videoModal.close());
videoModal.addEventListener("click", (event) => {
  if (event.target === videoModal) videoModal.close();
});
