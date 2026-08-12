export async function establishInitialFocus(input) {
  if (!Array.isArray(input.ids) || input.ids.length === 0) {
    throw new Error("initial focus requires an admitted video");
  }
  const startedAt = (input.now ?? Date.now)();
  await input.select(input.ids[0]);
  return {startedAt};
}
