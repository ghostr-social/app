const previousVideo = byId("previous-video");
const nextVideo = byId("next-video");

function navigationState() {
  const videos = latestState ? debugVideos(latestState) : [];
  return {
    videos,
    index: videos.findIndex((video) => video.id === currentId),
  };
}

function playAdjacent(offset) {
  const { videos, index } = navigationState();
  const target = videos[index + offset];
  if (target) play(target);
}

function updateNavigationControls() {
  const { videos, index } = navigationState();
  previousVideo.disabled = index <= 0;
  nextVideo.disabled = videos.length === 0 || index >= videos.length - 1;
}

previousVideo.addEventListener("click", () => playAdjacent(-1));
nextVideo.addEventListener("click", () => playAdjacent(1));
new MutationObserver(updateNavigationControls).observe(byId("video-queue"), {
  childList: true,
});
updateNavigationControls();
