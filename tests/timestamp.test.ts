import { Timestamp } from "../index";

test("min()", () => {
    const timestamp1 = Timestamp.fromBeatUnits(42);
    const timestamp2 = Timestamp.fromBeatUnits(43);
    expect(Timestamp.min(timestamp1, timestamp2).getBeatUnits()).toBe(42);
});
test("max()", () => {
    const timestamp1 = Timestamp.fromBeatUnits(42);
    const timestamp2 = Timestamp.fromBeatUnits(43);
    expect(Timestamp.max(timestamp1, timestamp2).getBeatUnits()).toBe(43);
});
test("eq()", () => {
    const timestamp1 = Timestamp.fromBeatUnits(42);
    const timestamp2 = Timestamp.fromBeatUnits(42);
    expect(Timestamp.eq(timestamp1, timestamp2)).toBe(true);
});
test("eq() not equals", () => {
    const timestamp1 = Timestamp.fromBeatUnits(42);
    const timestamp2 = Timestamp.fromBeatUnits(43);
    expect(Timestamp.eq(timestamp1, timestamp2)).toBe(false);
});

test("add()", () => {
    const timestamp1 = Timestamp.fromBeatUnits(42);
    const timestamp2 = Timestamp.fromBeatUnits(43);
    expect(Timestamp.add(timestamp1, timestamp2).getBeatUnits()).toBe(85);
});
test("sub()", () => {
    const timestamp1 = Timestamp.fromBeatUnits(43);
    const timestamp2 = Timestamp.fromBeatUnits(42);
    expect(Timestamp.sub(timestamp1, timestamp2).getBeatUnits()).toBe(1);
    expect(() => Timestamp.sub(timestamp2, timestamp1)).toThrow();
});
test("mul()", () => {
    const timestamp = Timestamp.fromBeatUnits(42);
    expect(Timestamp.mul(timestamp, 2).getBeatUnits()).toBe(84);
    expect(Timestamp.mul(timestamp, 2.8).getBeatUnits()).toBe(84);
    expect(() => Timestamp.mul(timestamp, -2)).toThrow();
});

test("zero() is zero", () => {
    expect(Timestamp.zero().getBeatUnits()).toBe(0);
});
test("infinity() exists", () => {
    expect(Timestamp.infinity()).toBeDefined();
});

test("Beat units -> Beat units", () => {
    const original = 42;
    const timestamp = Timestamp.fromBeatUnits(original);
    expect(timestamp.getBeatUnits()).toBe(original);
});
test("Beats -> Beats", () => {
    const original = 42;
    const timestamp = Timestamp.fromBeats(original);
    expect(timestamp.getBeats()).toBe(original);
});
test("Samples -> Samples", () => {
    // The number 375 fits nicely into the roundings of the conversion
    const original = 375;
    const timestamp = Timestamp.fromSamples(original, 48_000, 120);
    expect(timestamp.getSamples(48_000, 120)).toBe(original);
});
test("Beats -> Beat units", () => {
    const original = 42;
    const timestamp = Timestamp.fromBeats(original);
    expect(timestamp.getBeatUnits()).toBe(original * 1024);
});
test("Samples -> Beat units", () => {
    const original = 420;
    const timestamp = Timestamp.fromSamples(original, 48_000, 120);
    expect(timestamp.getBeatUnits()).toBe(17);
});
