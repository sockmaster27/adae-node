const { Engine } = require("../../index.node"); // Relative to the destination

let engine: any;
beforeEach(() => {
    engine = new Engine();
});
afterEach(() => {
    engine.close();
});

describe("Track addition and deletion", () => {
    function tracksEqual(track1: any, track2: any): [boolean, string] {
        if (track1.key !== track2.key)
            return [false, `Keys mismatched: ${track1.key} != ${track2.key}`];

        let equal = true;
        let reasons = [];
        const relevantMethods = [
            "getVolume",
            "getPanning",
        ];
        for (const method of relevantMethods) {
            const result1 = track1[method]();
            const result2 = track2[method]();
            if (result1 !== result2) {
                equal = false;
                reasons.push(`Result of calling ${method}() mismatched: ${result1} != ${result2}`);
            }
        }

        return [
            equal,
            reasons.length === 0 ? null : reasons.join("\n"),
        ];
    }

    function containsEqualTrack(list: any[], track: any): boolean {
        for (const listTrack of list) {
            if (tracksEqual(listTrack, track))
                return true;
        }
        return false;
    }

    test("Has tracks", () => {
        expect(engine.getTracks()).toBeDefined();
    });

    test("Get track from key", () => {
        const track = engine.addTrack();
        const gottenTrack = engine.getTrack(track.key);
        expect(tracksEqual(track, gottenTrack)).toStrictEqual([true, null]);
    });

    test("Get track from key fails when track is deleted", () => {
        const track = engine.addTrack();
        engine.deleteTrack(track);
        expect(engine.getTrack(track.key)).toThrowError();
    });

    test("Add single track", () => {
        const before = engine.getTracks().length;
        const newTrack = engine.addTrack();
        expect(engine.getTracks().length).toBe(before + 1);
        expect(
            containsEqualTrack(engine.getTracks(), newTrack)
        ).toBe(true);
    });

    test("Add number of tracks", () => {
        const before = engine.getTracks().length;
        const newTracks = engine.addTracks(5);
        expect(engine.getTracks().length).toBe(before + 5);
        for (const track of newTracks)
            expect(
                containsEqualTrack(engine.getTracks(), track)
            ).toBe(true);

    });

    test("Delete track", () => {
        const before = engine.getTracks();
        const newTrack = engine.addTrack();
        engine.deleteTrack(newTrack);

        expect(engine.getTracks().length).toBe(before.length);

        for (const track of engine.getTracks())
            expect(
                containsEqualTrack(before, track)
            ).toBe(true);
    });

    test("Delete multiple tracks", () => {
        const before = engine.getTracks();
        const newTracks = engine.addTracks(34);
        engine.deleteTracks(newTracks);

        expect(engine.getTracks().length).toBe(before.length);

        for (const track of engine.getTracks())
            expect(
                containsEqualTrack(before, track)
            ).toBe(true);
    });

    test("Reconstruct single track", () => {
        const newTrack = engine.addTrack();
        const before = engine.getTracks();
        const data = engine.deleteTrack(newTrack);
        engine.addTrack(data);

        expect(engine.getTracks().length).toBe(before.length);

        for (const track of engine.getTracks())
            expect(
                containsEqualTrack(before, track)
            ).toBe(true);
    });

    test("Reconstruct multiple tracks", () => {
        const newTracks = engine.addTracks(24);
        const before = engine.getTracks();
        const data = engine.deleteTracks(newTracks);
        engine.addTracks(data);

        expect(engine.getTracks().length).toBe(before.length);

        for (const track of engine.getTracks())
            expect(
                containsEqualTrack(before, track)
            ).toBe(true);
    });
});
