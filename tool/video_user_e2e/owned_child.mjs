export async function trackSpawned(child, lifecycle, label) {
  await spawned(child, label);
  lifecycle.track({pid: child.pid, label});
}

function spawned(child, label) {
  if (child.pid) return Promise.resolve();
  return new Promise((resolve, reject) => {
    child.once("spawn", resolve);
    child.once("error", (error) => {
      reject(new Error(`cannot start ${label}: ${error.message}`));
    });
  });
}
