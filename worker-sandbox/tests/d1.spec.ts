import { describe, test, expect } from "vitest";
import { mf } from "./mf";

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
});
