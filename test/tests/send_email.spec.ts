import { describe, test, expect } from "vitest";
import { mf, mfUrl } from "./mf";

type SendResult = {
  ok: boolean;
  messageId: string | null;
  error: string | null;
};

async function runScenario(name: string): Promise<SendResult> {
  const resp = await mf.dispatchFetch(`${mfUrl}send-email?scenario=${name}`);
  expect(resp.status).toBe(200);
  return (await resp.json()) as SendResult;
}

// Miniflare's send_email binding resolves to `undefined` on success rather
// than `{ messageId }` like real workerd, so `messageId` is expected to be an
// empty string here. We still assert the type/presence.
function expectSuccess(result: SendResult) {
  expect(result.error).toBeNull();
  expect(result.ok).toBe(true);
  expect(typeof result.messageId).toBe("string");
}

describe("send email (raw MIME)", () => {
  test("sends a valid email through the binding", async () => {
    expectSuccess(await runScenario("mime-ok"));
  });

  test.each([
    ["mime-missing-message-id", /message-id/i],
    ["mime-disallowed-sender", /email from .* not allowed/i],
    ["mime-disallowed-recipient", /email to .* not allowed/i],
    ["mime-from-mismatch", /from.*does not match/i],
  ])("rejects scenario %s", async (scenario, errorPattern) => {
    const result = await runScenario(scenario);
    expect(result.ok).toBe(false);
    expect(result.error).toMatch(errorPattern);
  });
});

describe("send email (structured builder)", () => {
  test("sends a plain-text email", async () => {
    expectSuccess(await runScenario("structured-ok"));
  });

  test("sends an HTML email with a display-name sender", async () => {
    expectSuccess(await runScenario("structured-with-name"));
  });

  test.each([
    ["structured-disallowed-sender", /email from .* not allowed/i],
    ["structured-disallowed-recipient", /email to .* not allowed/i],
  ])("rejects scenario %s", async (scenario, errorPattern) => {
    const result = await runScenario(scenario);
    expect(result.ok).toBe(false);
    expect(result.error).toMatch(errorPattern);
  });
});
