import { describe, test, expect } from "vitest";
import { mf, mfUrl } from "./mf";

type SendResult = { ok: boolean; error: string | null };

async function runScenario(name: string): Promise<SendResult> {
  const resp = await mf.dispatchFetch(`${mfUrl}send-email?scenario=${name}`);
  expect(resp.status).toBe(200);
  return (await resp.json()) as SendResult;
}

describe("send email", () => {
  test("sends a valid email through the binding", async () => {
    const result = await runScenario("ok");
    expect(result).toEqual({ ok: true, error: null });
  });

  test.each([
    ["missing-message-id", /message-id/i],
    ["disallowed-sender", /email from .* not allowed/i],
    ["disallowed-recipient", /email to .* not allowed/i],
    ["from-mismatch", /from.*does not match/i],
  ])("rejects scenario %s", async (scenario, errorPattern) => {
    const result = await runScenario(scenario);
    expect(result.ok).toBe(false);
    expect(result.error).toMatch(errorPattern);
  });
});
