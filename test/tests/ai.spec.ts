import { describe, expect, test } from "vitest";
import { mf, mfUrl } from "./mf";

async function runTest() {
  let normal_response = await mf.dispatchFetch(`${mfUrl}ai`);
  expect(normal_response.status).toBe(200);

  let streaming_response = await mf.dispatchFetch(`${mfUrl}ai/streaming`);
  expect(streaming_response.status).toBe(200);
}
describe("ai", runTest);
