const path = require("path");

const {
    Engine,
    Timestamp,
    inverseMeterScale,
    meterScale,
    listenForCrash,
    stopListeningForCrash,
} = require("../../index.node"); // Relative to the destination

describe("Engine", () => {
    describe("Constructors", () => {
        test("Default constructor", () => {
            // We do not actually call this constructor,
            // because it might fail if run on a machine without a sound card
            expect(typeof Engine).toBe("function");
        });

        test("Dummy constructor", () => {
            expect(Engine.dummy()).toBeDefined();
        });
    });

    let engine: any;
    beforeEach(() => {
        engine = Engine.dummy();
        listenForCrash().catch((e: Error) => {
            throw e;
        });
    });
    afterEach(() => {
        engine.close();
        stopListeningForCrash();
    });

    function importTestClip() {
        return engine.importAudioClip(
            path.join(
                __dirname,
                "..",
                "..",
                "test_files",
                "48000 32-float.wav",
            ),
        );
    }

    describe("Timeline", () => {
        test("play()", () => {
            expect(engine.play()).toBeUndefined();
        });

        test("pause()", () => {
            expect(engine.pause()).toBeUndefined();
        });

        test("jumpTo()", () => {
            expect(engine.jumpTo(Timestamp.zero())).toBeUndefined();
        });

        test("getPlayheadPosition()", () => {
            expect(engine.getPlayheadPosition()).toBeDefined();
        });
    });

    describe("Mixer", () => {
        test("Get master", () => {
            expect(engine.getMaster()).toBeDefined();
        });

        describe("Audio track addition and deletion", () => {
            function tracksEqual(track1: any, track2: any): [boolean, string] {
                if (track1.key() !== track2.key())
                    return [
                        false,
                        `Keys mismatched: ${track1.key()} != ${track2.key()}`,
                    ];

                let equal = true;
                let reasons = [];
                const relevantMethods = ["getVolume", "getPanning"];
                for (const method of relevantMethods) {
                    const result1 = track1[method]();
                    const result2 = track2[method]();
                    if (result1 !== result2) {
                        equal = false;
                        reasons.push(
                            `Result of calling ${method}() mismatched: ${result1} != ${result2}`,
                        );
                    }
                }

                return [
                    equal,
                    reasons.length === 0 ? null : reasons.join("\n"),
                ];
            }

            function containsEqualAudioTrack(list: any[], track: any): boolean {
                for (const listAudioTrack of list) {
                    if (tracksEqual(listAudioTrack, track)) return true;
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
                    containsEqualAudioTrack(
                        engine.getAudioTracks(),
                        newAudioTrack,
                    ),
                ).toBe(true);
            });

            test("Add number of tracks", () => {
                const before = engine.getAudioTracks().length;
                const newAudioTracks = engine.addAudioTracks(5);
                expect(engine.getAudioTracks().length).toBe(before + 5);
                for (const track of newAudioTracks)
                    expect(
                        containsEqualAudioTrack(engine.getAudioTracks(), track),
                    ).toBe(true);
            });

            test("Delete track", () => {
                const before = engine.getAudioTracks();
                const newAudioTrack = engine.addAudioTrack();
                engine.deleteAudioTrack(newAudioTrack);

                expect(engine.getAudioTracks().length).toBe(before.length);

                for (const track of engine.getAudioTracks())
                    expect(containsEqualAudioTrack(before, track)).toBe(true);
            });

            test("Delete multiple tracks", () => {
                const before = engine.getAudioTracks();
                const newAudioTracks = engine.addAudioTracks(34);
                engine.deleteAudioTracks(newAudioTracks);

                expect(engine.getAudioTracks().length).toBe(before.length);

                for (const track of engine.getAudioTracks())
                    expect(containsEqualAudioTrack(before, track)).toBe(true);
            });

            test("Reconstruct single track", () => {
                const newAudioTrack = engine.addAudioTrack();
                const before = engine.getAudioTracks();
                const state = engine.deleteAudioTrack(newAudioTrack);
                engine.reconstructAudioTrack(state);

                expect(engine.getAudioTracks().length).toBe(before.length);

                for (const track of engine.getAudioTracks())
                    expect(containsEqualAudioTrack(before, track)).toBe(true);
            });

            test("Reconstruct multiple tracks", () => {
                const newAudioTracks = engine.addAudioTracks(24);
                const before = engine.getAudioTracks();
                const states = engine.deleteAudioTracks(newAudioTracks);
                engine.reconstructAudioTracks(states);

                expect(engine.getAudioTracks().length).toBe(before.length);

                for (const track of engine.getAudioTracks())
                    expect(containsEqualAudioTrack(before, track)).toBe(true);
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
                engine = Engine.dummy();
            });
        });

        describe("Individual track", () => {
            let track: any;

            describe("Master track", () => {
                beforeEach(() => (track = engine.getMaster()));

                testTrackCommon();
            });

            describe("Audio track", () => {
                beforeEach(() => (track = engine.addAudioTrack()));

                testTrackCommon();

                test("Has key", () => {
                    expect(typeof track.key()).toBe("number");
                });

                test("addClip()", () => {
                    const clip = importTestClip();
                    track.addClip(clip, Timestamp.zero());
                });

                test("delete() deletes track", () => {
                    track.delete();
                    expect(() =>
                        engine.getAudioTrack(track.key()),
                    ).toThrowError();
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

                    expect(Object.getOwnPropertyNames(result)).toStrictEqual([
                        "peak",
                        "longPeak",
                        "rms",
                    ]);

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

    describe("Stored audio clip", () => {
        test("importAudioClip()", () => {
            expect(importTestClip()).toBeDefined();
        });
        test("importAudioClip() throws when file doesn't exist", () => {
            expect(() => engine.importAudioClip("nonexistent")).toThrowError();
        });

        test("key()", () => {
            const clip = importTestClip();
            expect(clip.key()).toBeDefined();
        });

        test("sampleRate()", () => {
            const clip = importTestClip();
            expect(clip.sampleRate()).toBe(48_000);
        });

        test("length()", () => {
            const clip = importTestClip();
            expect(clip.length()).toBe(1_322_978);
        });
    });

    describe("Timeline audio clip", () => {
        let clip: any;

        beforeEach(() => {
            const track = engine.addAudioTrack();
            clip = track.addClip(
                importTestClip(),
                Timestamp.fromBeats(1),
                Timestamp.fromBeats(2),
            );
        });

        test("key()", () => {
            expect(typeof clip.key()).toBe("number");
        });

        test("start()", () => {
            expect(clip.start().getBeats()).toBe(1);
        });

        test("length()", () => {
            expect(clip.length().getBeats()).toBe(2);
        });

        test("length() null", () => {
            const track = engine.addAudioTrack();
            clip = track.addClip(importTestClip(), Timestamp.fromBeats(1));
            expect(clip.length()).toBeNull();
        });

        test("storedClip()", () => {
            expect(clip.storedClip()).toBeDefined();
        });
    });
});

describe("Timestamp", () => {
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
        expect(() => Timestamp.sub(timestamp2, timestamp1)).toThrowError();
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
});

test("inverseMeterScale() is inverse of meterScale()", () => {
    const result = inverseMeterScale(meterScale(0.6));
    expect(Math.abs(result - 0.6)).toBeLessThan(0.00001);
});
