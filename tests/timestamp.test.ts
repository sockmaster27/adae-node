import { Timestamp } from "../index";

test("min()", () => {
    const timestamp1 = Timestamp.fromBeatUnits(42);
    const timestamp2 = Timestamp.fromBeatUnits(43);
    expect(Timestamp.min(timestamp1, timestamp2).getBeatUnits()).toStrictEqual(
        42,
    );
});
test("max()", () => {
    const timestamp1 = Timestamp.fromBeatUnits(42);
    const timestamp2 = Timestamp.fromBeatUnits(43);
    expect(Timestamp.max(timestamp1, timestamp2).getBeatUnits()).toStrictEqual(
        43,
    );
});
test("eq()", () => {
    const timestamp1 = Timestamp.fromBeatUnits(42);
    const timestamp2 = Timestamp.fromBeatUnits(42);
    expect(Timestamp.eq(timestamp1, timestamp2)).toStrictEqual(true);
});
test("eq() not equals", () => {
    const timestamp1 = Timestamp.fromBeatUnits(42);
    const timestamp2 = Timestamp.fromBeatUnits(43);
    expect(Timestamp.eq(timestamp1, timestamp2)).toStrictEqual(false);
});

test("add()", () => {
    const timestamp1 = Timestamp.fromBeatUnits(42);
    const timestamp2 = Timestamp.fromBeatUnits(43);
    expect(Timestamp.add(timestamp1, timestamp2).getBeatUnits()).toStrictEqual(
        85,
    );
});
test("sub()", () => {
    const timestamp1 = Timestamp.fromBeatUnits(43);
    const timestamp2 = Timestamp.fromBeatUnits(42);
    expect(Timestamp.sub(timestamp1, timestamp2).getBeatUnits()).toStrictEqual(
        1,
    );
    expect(() => Timestamp.sub(timestamp2, timestamp1)).toThrow();
});
test("mul()", () => {
    const timestamp = Timestamp.fromBeatUnits(42);
    expect(Timestamp.mul(timestamp, 2).getBeatUnits()).toStrictEqual(84);
    expect(Timestamp.mul(timestamp, 2.8).getBeatUnits()).toStrictEqual(84);
    expect(() => Timestamp.mul(timestamp, -2)).toThrow();
});

test("zero() is zero", () => {
    expect(Timestamp.zero().getBeatUnits()).toStrictEqual(0);
});
test("infinity() exists", () => {
    expect(Timestamp.infinity()).toBeDefined();
});

test("Beat units -> Beat units", () => {
    const original = 42;
    const timestamp = Timestamp.fromBeatUnits(original);
    expect(timestamp.getBeatUnits()).toStrictEqual(original);
});
test("Beats -> Beats", () => {
    const original = 42;
    const timestamp = Timestamp.fromBeats(original);
    expect(timestamp.getBeats()).toStrictEqual(original);
});
test("Samples -> Samples", () => {
    // The number 375 fits nicely into the roundings of the conversion
    const original = 375;
    const timestamp = Timestamp.fromSamples(original, 48_000, 120);
    expect(timestamp.getSamples(48_000, 120)).toStrictEqual(original);
});
test("Beats -> Beat units", () => {
    const original = 42;
    const timestamp = Timestamp.fromBeats(original);
    expect(timestamp.getBeatUnits()).toStrictEqual(original * 1024);
});
test("Samples -> Beat units", () => {
    const original = 420;
    const timestamp = Timestamp.fromSamples(original, 48_000, 120);
    expect(timestamp.getBeatUnits()).toStrictEqual(17);
});
