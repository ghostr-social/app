export async function establishOrderedFocus(input) {
  const baseline = await input.read();
  const startedAt = (input.now ?? Date.now)();
  input.record?.({id: input.ids[0], baseline, startedAt});
  await input.select(input.ids[0]);
  const warm = input.warm?.({baseline, startedAt});
  return {startedAt, warm};
}
