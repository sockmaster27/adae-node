const { Engine, inverseMeterScale, meterScale } = require("../../index.node"); // Relative to the destination



describe("Engine", () => {
    let engine: any;
    beforeEach(() => {
        engine = new Engine();
    });
    afterEach(() => {
        engine.close();
    });

    describe("Mixer", () => {
        test("Get master", () => {
            expect(engine.getMaster()).toBeDefined();
        });

        describe("Audio track addition and deletion", () => {
            function tracksEqual(track1: any, track2: any): [boolean, string] {
                if (track1.key() !== track2.key())
                    return [false, `Keys mismatched: ${track1.key()} != ${track2.key()}`];

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

            function containsEqualAudioTrack(list: any[], track: any): boolean {
                for (const listAudioTrack of list) {
                    if (tracksEqual(listAudioTrack, track))
                        return true;
                }
                return false;
            }

            test("Has tracks", () => {
                expect(engine.getAudioTracks()).toBeDefined();
            });

            test("Get track from key", () => {
                const track = engine.addAudioTrack();
                expect(tracksEqual(track, track)).toStrictEqual([true, null]);
            });

            test("Get track from key fails when track is deleted", () => {
                const track = engine.addAudioTrack();
                engine.deleteAudioTrack(track);
                expect(() => engine.getAudioTrack(track.key())).toThrowError();
            });

            test("Add single track", () => {
                const before = engine.getAudioTracks().length;
                const newAudioTrack = engine.addAudioTrack();
                expect(engine.getAudioTracks().length).toBe(before + 1);
                expect(
                    containsEqualAudioTrack(engine.getAudioTracks(), newAudioTrack)
                ).toBe(true);
            });

            test("Add number of tracks", () => {
                const before = engine.getAudioTracks().length;
                const newAudioTracks = engine.addAudioTracks(5);
                expect(engine.getAudioTracks().length).toBe(before + 5);
                for (const track of newAudioTracks)
                    expect(
                        containsEqualAudioTrack(engine.getAudioTracks(), track)
                    ).toBe(true);

            });

            test("Delete track", () => {
                const before = engine.getAudioTracks();
                const newAudioTrack = engine.addAudioTrack();
                engine.deleteAudioTrack(newAudioTrack);

                expect(engine.getAudioTracks().length).toBe(before.length);

                for (const track of engine.getAudioTracks())
                    expect(
                        containsEqualAudioTrack(before, track)
                    ).toBe(true);
            });

            test("Delete multiple tracks", () => {
                const before = engine.getAudioTracks();
                const newAudioTracks = engine.addAudioTracks(34);
                engine.deleteAudioTracks(newAudioTracks);

                expect(engine.getAudioTracks().length).toBe(before.length);

                for (const track of engine.getAudioTracks())
                    expect(
                        containsEqualAudioTrack(before, track)
                    ).toBe(true);
            });

            test("Reconstruct single track", () => {
                const newAudioTrack = engine.addAudioTrack();
                const before = engine.getAudioTracks();
                const state = engine.deleteAudioTrack(newAudioTrack);
                engine.reconstructAudioTrack(state);

                expect(engine.getAudioTracks().length).toBe(before.length);

                for (const track of engine.getAudioTracks())
                    expect(
                        containsEqualAudioTrack(before, track)
                    ).toBe(true);
            });

            test("Reconstruct multiple tracks", () => {
                const newAudioTracks = engine.addAudioTracks(24);
                const before = engine.getAudioTracks();
                const states = engine.deleteAudioTracks(newAudioTracks);
                engine.reconstructAudioTracks(states);

                expect(engine.getAudioTracks().length).toBe(before.length);

                for (const track of engine.getAudioTracks())
                    expect(
                        containsEqualAudioTrack(before, track)
                    ).toBe(true);
            });

            test("All methods throw when engine is closed", () => {
                engine.close();
                const methods = [
                    "getMaster",
                    "getAudioTracks",
                    "addAudioTrack",
                    "addAudioTracks",
                    "deleteAudioTrack",
                    "deleteAudioTracks",
                    "importAudioClip",
                    "close",
                ];

                for (const method of methods)
                    expect(engine[method]).toThrowError();

                // So cleanup can run
                engine = new Engine();
            });
        });


        describe("Individual track", () => {
            let track: any;

            describe("Master track", () => {
                beforeEach(() => track = engine.getMaster());

                testTrackCommon();
            });

            describe("Audio track", () => {
                beforeEach(() => track = engine.addAudioTrack());

                testTrackCommon();

                test("Has key", () => {
                    expect(typeof track.key()).toBe("number");
                });

                test("delete() deletes track", () => {
                    track.delete();
                    expect(() => engine.getAudioTrack(track.key())).toThrowError();
                });

                test("All methods throw when track is deleted", () => {
                    track.delete();
                    const methods = [
                        "getPanning",
                        "setPanning",
                        "getVolume",
                        "setVolume",
                        "readMeter",
                        "snapMeter",

                        "key",
                        "addClip",
                        "delete",
                    ];

                    for (const method of methods)
                        expect(track[method]).toThrowError();
                });
            });

            function testTrackCommon() {
                test("getPanning() returns what's passed to setPanning()", () => {
                    track.setPanning(0.5);
                    expect(track.getPanning()).toBe(0.5);
                });

                test("getVolume() returns what's passed to setVolume()", () => {
                    track.setVolume(0.5);
                    expect(track.getVolume()).toBe(0.5);
                });

                test("readMeter() returns right type", () => {
                    const result = track.readMeter();

                    expect(typeof result).toBe("object");

                    expect(
                        Object.getOwnPropertyNames(result)
                    ).toStrictEqual(["peak", "longPeak", "rms"]);

                    for (const stat of Object.values(result))
                        expect((stat as any[]).length).toBe(2);

                    for (const number of Object.values(result).flat())
                        expect(typeof number).toBe("number");
                });

                test("snapMeter() exists", () => {
                    expect(typeof track.snapMeter).toBe("function");
                });
            }
        });
    });
});

describe("Timestamp", () => {
    test("zero() is zero", () => {
        expect(Timestamp.zero().getBeatUnits()).toBe(0);
    });

    test("fromBeatUnits() -> getBeatUnits()", () => {
        const original = 42;
        const timestamp = Timestamp.fromBeatUnits(original);
        expect(timestamp.getBeatUnits()).toBe(original);
    });

    test("equals", () => {
        const timestamp1 = Timestamp.fromBeatUnits(42);
        const timestamp2 = Timestamp.fromBeatUnits(42);
        expect(timestamp1.equals(timestamp2)).toBe(true);
    });
    test("not equals", () => {
        const timestamp1 = Timestamp.fromBeatUnits(42);
        const timestamp2 = Timestamp.fromBeatUnits(43);
        expect(timestamp1.equals(timestamp2)).toBe(false);
    });
});

test("inverseMeterScale() is inverse of meterScale()", () => {
    const result = inverseMeterScale(meterScale(0.6));
    expect(Math.abs(result - 0.6)).toBeLessThan(0.00001);
});
