import { describe, test, expect } from "vitest";
import { mf, mfUrl } from "./mf";

async function exec(query: string): Promise<number> {
  const resp = await mf.dispatchFetch("http://fake.host/d1/exec", {
    method: "POST",
    body: query.split("\n").join(""),
  });

  const body = await resp.text();
  expect(resp.status).toBe(200);
  return Number(body);
}

describe("d1", () => {
  test("create table", async () => {
    const query = `CREATE TABLE IF NOT EXISTS uniqueTable (
      id INTEGER PRIMARY KEY,
      name TEXT NOT NULL,
      age INTEGER NOT NULL
    );`;

    expect(await exec(query)).toBe(1);
  });

  test("insert data", async () => {
    let query = `CREATE TABLE IF NOT EXISTS people (
      id INTEGER PRIMARY KEY,
      name TEXT NOT NULL,
      age INTEGER NOT NULL
    );`;

    expect(await exec(query)).toBe(1);

    query = `INSERT OR IGNORE INTO people
    (id, name, age)
    VALUES
    (1, 'Freddie Pearce', 26),
    (2, 'Wynne Ogley', 67),
    (3, 'Dorian Fischer', 19),
    (4, 'John Smith', 92),
    (5, 'Magaret Willamson', 54),
    (6, 'Ryan Upton', 21);`;

    expect(await exec(query)).toBe(1);
  });

  test("prepared statement", async () => {
    const resp = await mf.dispatchFetch("http://fake.host/d1/prepared");
    expect(resp.status).toBe(200);
  });

  test("batch", async () => {
    const resp = await mf.dispatchFetch("http://fake.host/d1/batch");
    expect(resp.status).toBe(200);
  });

  test("error", async () => {
    const resp = await mf.dispatchFetch("http://fake.host/d1/error");
    expect(resp.status).toBe(200);
  });

  test("create table nullable", async () => {
    let query = `CREATE TABLE IF NOT EXISTS nullable_people (
      id INTEGER PRIMARY KEY,
      name TEXT,
      age INTEGER
    );`;

    expect(await exec(query)).toBe(1);

    query = `INSERT OR IGNORE INTO nullable_people
    (id, name, age)
    VALUES
    (1, NULL, NULL),
    (2, 'Wynne Ogley', 67)`;

    expect(await exec(query)).toBe(1);
  });

  test("jsvalue_null_is_null", async () => {
    const resp = await mf.dispatchFetch("http://fake.host/d1/jsvalue_null_is_null");
    expect(await resp.text()).toBe("ok");
    expect(resp.status).toBe(200);
  });

  test("serialize_optional_none", async () => {
    const resp = await mf.dispatchFetch("http://fake.host/d1/serialize_optional_none");
    expect(await resp.text()).toBe("ok");
    expect(resp.status).toBe(200);
  });

  test("serialize_optional_some", async () => {
    const resp = await mf.dispatchFetch("http://fake.host/d1/serialize_optional_some");
    expect(await resp.text()).toBe("ok");
    expect(resp.status).toBe(200);
  });

  test("deserialize_optional_none", async () => {
    const resp = await mf.dispatchFetch("http://fake.host/d1/deserialize_optional_none");
    expect(await resp.text()).toBe("ok");
    expect(resp.status).toBe(200);
  });

  test("insert_and_retrieve_optional_none", async () => {
    const resp = await mf.dispatchFetch("http://fake.host/d1/insert_and_retrieve_optional_none");
    expect(await resp.text()).toBe("ok");
    expect(resp.status).toBe(200);
  });

  test("insert_and_retrieve_optional_some", async () => {
    const resp = await mf.dispatchFetch("http://fake.host/d1/insert_and_retrieve_optional_some");
    expect(await resp.text()).toBe("ok");
    expect(resp.status).toBe(200);
  });

  test("retrieve_optional_none", async () => {
    const resp = await mf.dispatchFetch("http://fake.host/d1/retrieve_optional_none");
    expect(await resp.text()).toBe("ok");
    expect(resp.status).toBe(200);
    });
  
  test("retrieve_optional_some", async () => {
    const resp = await mf.dispatchFetch("http://fake.host/d1/retrieve_optional_some");
    expect(await resp.text()).toBe("ok");
    expect(resp.status).toBe(200);
    });
  
  test("retrive_first_none", async () => {
    const resp = await mf.dispatchFetch("http://fake.host/d1/retrive_first_none");
    expect(await resp.text()).toBe("ok");
    expect(resp.status).toBe(200);
  });
});
