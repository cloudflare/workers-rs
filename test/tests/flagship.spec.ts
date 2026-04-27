import { describe, test, expect } from "vitest";
import { mf, mfUrl } from "./mf";

type Details<T> = {
  flagKey: string;
  value: T;
  variant: string | null;
  reason: string | null;
  errorCode: string | null;
  errorMessage: string | null;
};

async function json<T>(path: string): Promise<T> {
  const resp = await mf.dispatchFetch(`${mfUrl}${path}`);
  expect(resp.status).toBe(200);
  return (await resp.json()) as T;
}

describe("flagship", () => {
  describe("value methods", () => {
    test("get_boolean_value resolves a known flag", async () => {
      const data = await json<{ flag: string; value: boolean }>("flagship/bool/dark-mode");
      expect(data.value).toBe(true);
    });

    test("get_boolean_value returns the default for unknown flags", async () => {
      const data = await json<{ flag: string; value: boolean }>("flagship/bool/missing");
      expect(data.value).toBe(false);
    });

    test("get_string_value resolves a known flag", async () => {
      const data = await json<{ flag: string; value: string }>("flagship/string/checkout-flow");
      expect(data.value).toBe("v2");
    });

    test("get_string_value returns the default for unknown flags", async () => {
      const data = await json<{ flag: string; value: string }>("flagship/string/missing");
      expect(data.value).toBe("fallback");
    });

    test("get_number_value resolves a known flag", async () => {
      const data = await json<{ flag: string; value: number }>("flagship/number/max-retries");
      expect(data.value).toBe(5);
    });

    test("get_number_value returns the default for unknown flags", async () => {
      const data = await json<{ flag: string; value: number }>("flagship/number/missing");
      expect(data.value).toBe(0);
    });

    test("get_object_value deserializes into a typed struct", async () => {
      const data = await json<{ flag: string; value: { primary: string; secondary: string } }>(
        "flagship/object/theme-colors",
      );
      expect(data.value).toEqual({ primary: "#ff0000", secondary: "#00ff00" });
    });

    test("get_object_value returns the default for unknown flags", async () => {
      const data = await json<{ flag: string; value: { primary: string; secondary: string } }>(
        "flagship/object/missing",
      );
      expect(data.value).toEqual({ primary: "#000000", secondary: "#ffffff" });
    });

    test("get (untyped) round-trips arbitrary JSON", async () => {
      const data = await json<{ flag: string; value: unknown }>("flagship/get/theme-colors");
      expect(data.value).toEqual({ primary: "#ff0000", secondary: "#00ff00" });
    });
  });

  describe("evaluation context", () => {
    test("context routing picks the targeted branch", async () => {
      const data = await json<{ userId: string; value: string }>("flagship/context/alice");
      expect(data.value).toBe("alice-branch");
    });

    test("context routing falls back when targeting misses", async () => {
      const data = await json<{ userId: string; value: string }>("flagship/context/bob");
      expect(data.value).toBe("default");
    });
  });

  describe("details methods", () => {
    test("boolean details include variant + reason on a match", async () => {
      const details = await json<Details<boolean>>("flagship/details/bool/dark-mode");
      expect(details.flagKey).toBe("dark-mode");
      expect(details.value).toBe(true);
      expect(details.variant).toBe("on");
      expect(details.reason).toBe("TARGETING_MATCH");
      expect(details.errorCode).toBeNull();
    });

    test("boolean details surface errorCode on miss", async () => {
      const details = await json<Details<boolean>>("flagship/details/bool/missing");
      expect(details.value).toBe(false);
      expect(details.errorCode).toBe("GENERAL");
      expect(details.errorMessage).toBe("flag not found");
      expect(details.reason).toBe("DEFAULT");
    });

    test("string details include variant metadata", async () => {
      const details = await json<Details<string>>("flagship/details/string/checkout-flow");
      expect(details.value).toBe("v2");
      expect(details.variant).toBe("rollout-25");
    });

    test("number details round-trip numeric payloads", async () => {
      const details = await json<Details<number>>("flagship/details/number/max-retries");
      expect(details.value).toBe(5);
      expect(details.variant).toBe("bumped");
    });

    test("object details deserialize into a typed struct", async () => {
      const details = await json<Details<{ primary: string; secondary: string }>>(
        "flagship/details/object/theme-colors",
      );
      expect(details.value).toEqual({ primary: "#ff0000", secondary: "#00ff00" });
      expect(details.variant).toBe("brand-v2");
    });
  });
});
