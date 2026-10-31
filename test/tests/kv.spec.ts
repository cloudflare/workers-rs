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

    // Maps are serialized as JS `Map`s by default, which `JSON.stringify` encodes
    // as `{}`; `KvStore::put` must store plain objects instead.
    test("put-struct-map", async () => {
        const body = {
            id: "put_struct_map",
            map: { k1: "v1", k2: "v2" },
            value: { a: 1, b: { c: "d" }, list: [1, "two", null] },
        };
        let response = await mf.dispatchFetch("http://fake.host/kv/put-struct-map", {
            method: "POST",
            body: JSON.stringify(body),
        });
        expect(response.status).toBe(200);

        const raw = await store.get("put_struct_map");
        expect(JSON.parse(raw!)).toEqual(body);
    });
}
describe("kv", runTest);