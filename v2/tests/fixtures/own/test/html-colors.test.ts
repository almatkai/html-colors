import assert from "node:assert/strict";
import test, { describe, it } from "node:test";
import { getMatchScore, searchColors } from "../src/utils/search-utils";
import { sortSectionsByRelevance } from "../src/utils/section-utils";
import { groupColorsByShade, generateColorPreviewSvg, getCategoryIcons } from "../src/utils/color-utils";
import { basicColors, extendedColors } from "../src/constants";
import type { ColorResult } from "../src/utils/search-utils";

describe("getMatchScore", () => {
  it("returns 0 for exact match", () => {
    assert.equal(getMatchScore("Red", "Red"), 0);
    assert.equal(getMatchScore("red", "RED"), 0);
  });
  it("returns 0.2 for word boundary", () => {
    assert.equal(getMatchScore("Light Blue", "Light"), 0.2);
  });
  it("returns 0.4 for start match", () => {
    assert.equal(getMatchScore("RedOrange", "Red"), 0.4);
  });
  it("returns 0.6-0.8 for contains", () => {
    const s = getMatchScore("DarkRed", "Red");
    assert.ok(s >= 0.6 && s <= 0.8);
  });
  it("returns 1 for no match", () => {
    assert.equal(getMatchScore("Blue", "Green"), 1);
  });
});

describe("searchColors", () => {
  it("returns all when empty search", () => {
    const res = searchColors(basicColors, "");
    assert.equal(res.length, basicColors.length);
  });
  it("finds by name fuzzy", () => {
    const res = searchColors([...basicColors, ...extendedColors], "Red");
    assert.ok(res.length > 0);
    assert.ok(res.some((c) => c.name.toLowerCase().includes("red")));
  });
  it("finds by hex", () => {
    const res = searchColors(basicColors, "#FF0000");
    assert.ok(res.some((c) => c.hex === "#FF0000"));
  });
  it("deduplicates by name (basic + extended)", () => {
    const combined = [...basicColors, ...extendedColors];
    const res = searchColors(combined, "Red");
    const names = res.map((c) => c.name);
    assert.equal(names.length, new Set(names).size);
    // Red exists in both basic and extended, should have categories [basic,extended]
    const red = res.find((c) => c.name === "Red");
    if (red) {
      assert.ok(red.categories.includes("basic"));
      // extended also contains Red
      assert.ok(red.categories.length >= 1);
    }
  });
  it("filters by threshold", () => {
    const res = searchColors(basicColors, "zzzzzz");
    assert.equal(res.length, 0);
  });
});

describe("groupColorsByShade", () => {
  it("groups by shade category", () => {
    const sample: ColorResult[] = [
      { name: "Red", hex: "#FF0000", rgb: "rgb(255,0,0)", id: "red", format: "name", shade: "red", categories: ["basic"] },
      { name: "Blue", hex: "#0000FF", rgb: "rgb(0,0,255)", id: "blue", format: "name", shade: "blue", categories: ["basic"] },
    ];
    const grouped = groupColorsByShade(sample);
    assert.equal(grouped["red"].length, 1);
    assert.equal(grouped["blue"].length, 1);
    assert.equal(grouped["pink"].length, 0);
  });
  it("handles empty", () => {
    const grouped = groupColorsByShade([]);
    for (const k of Object.keys(grouped)) {
      assert.equal((grouped as Record<string, ColorResult[]>)[k].length, 0);
    }
  });
});

describe("sortSectionsByRelevance", () => {
  it("sorts by static order when no search", () => {
    const grouped = groupColorsByShade(
      searchColors(basicColors, "")
    );
    const sections = Object.entries(grouped);
    const sorted = sortSectionsByRelevance(sections as never, "");
    // first non-empty should be red? Actually basicColors has no pink, so first present shade in SHADE_ORDER with colors
    assert.ok(sorted.length > 0);
    // Empty sections filtered out
    for (const [, colors] of sorted) assert.ok(colors.length > 0);
  });
  it("sorts by relevance when search", () => {
    const colors = [...basicColors, ...extendedColors];
    const filtered = searchColors(colors, "Blue");
    const grouped = groupColorsByShade(filtered);
    const sections = Object.entries(grouped);
    const sorted = sortSectionsByRelevance(sections as never, "Blue");
    assert.ok(sorted.length > 0);
    // Best match section containing Blue should be first
    assert.equal(sorted[0][0], "blue");
  });
});

describe("generateColorPreviewSvg", () => {
  it("generates base64 SVG with hex and name", () => {
    const svg = generateColorPreviewSvg("#FF0000", "Red");
    assert.match(svg, /!\[Red Color Preview\]/);
    assert.match(svg, /data:image\/svg\+xml;base64,/);
    const b64 = svg.match(/base64,(.*)\)/)![1];
    const decoded = Buffer.from(b64, "base64").toString("utf-8");
    assert.match(decoded, /#FF0000/);
    assert.match(decoded, /rect/);
  });
});

describe("getCategoryIcons", () => {
  it("returns Circle for basic, CircleEllipsis for extended", () => {
    const basic = getCategoryIcons(["basic"]);
    assert.equal(basic.length, 1);
    assert.ok(basic[0].tooltip.includes("Basic"));
    const extended = getCategoryIcons(["extended"]);
    assert.ok(extended[0].tooltip.includes("Extended"));
    const both = getCategoryIcons(["extended", "basic"]);
    // sorted basic first
    assert.ok(both[0].tooltip.includes("Basic"));
    assert.ok(both[1].tooltip.includes("Extended"));
  });
});

test("package.json declares Windows+macOS and Tezbar metadata", async () => {
  const pkg = await import("../package.json", { with: { type: "json" } });
  const data = (pkg as unknown as { default: Record<string, unknown> }).default ?? (pkg as unknown as Record<string, unknown>);
  assert.deepEqual((data as { platforms: string[] }).platforms, ["macOS", "Windows"]);
  const tezbar = (data as { tezbar: { platforms: string[]; repository: string; upstream: { commit: string; path: string } } }).tezbar;
  assert.deepEqual(tezbar.platforms, ["macos", "windows"]);
  assert.equal(tezbar.repository, "https://github.com/almatkai/html-colors");
  assert.equal(tezbar.upstream.path, "extensions/html-colors");
  assert.equal(tezbar.upstream.commit, "6734d85992290066fd31a7f5bee3cca3d3545a1a");
  const commands = (data as { commands: { platforms: string[] }[] }).commands;
  assert.equal(commands.length, 1);
  for (const cmd of commands) assert.deepEqual(cmd.platforms, ["macOS", "Windows"]);
});
