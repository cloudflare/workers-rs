import { describe, test, expect } from "vitest";
import { mf, mfUrl } from "./mf";

const CASES = [
    "get",
    "get-not-found",
    "list-keys",
    "put-simple",
    "put-metadata",
    "put-metadata-struct",
    "put-expiration"
]

async function runTest() {
    let store = await mf.getKVNamespace("TEST");
    await store.put("simple", "passed");

    for (let testCase of CASES) {
        test(testCase, async () => {
            let response = await  mf.dispatchFetch(`http://fake.host/kv/${testCase}`);
            expect(response.status).toBe(200);
        });
    }
}
describe("kv", runTest);