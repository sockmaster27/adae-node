import path from "path";

import {
    Timestamp,
    Engine,
    meterScale,
    inverseMeterScale,
    listenForCrash,
    stopListeningForCrash,
    AudioTrack,
    Track,
    AudioClip,
} from "../index";

describe("Engine", () => {
    describe("Constructors", () => {
        test("Default constructor", () => {
            // We do not actually call this constructor,
            // because it might fail if run on a machine without a sound card
            expect(typeof Engine).toBe("function");
        });

        test("Dummy constructor", () => {
            expect(Engine.getDummy()).toBeDefined();
        });
    });

    beforeAll(() => {
        listenForCrash().catch((e: Error) => {
            throw e;
        });
    });
    afterAll(async () => {
        stopListeningForCrash();
    });

    let engine: Engine;
    beforeEach(() => {
        engine = Engine.getDummy();
    });
    afterEach(() => {
        engine.close();
    });

    function importTestClip() {
        return engine.importAudioClip(
            path.join(__dirname, "..", "test_files", "48000 32-float.wav"),
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
            function tracksEqual(
                track1: AudioTrack,
                track2: AudioTrack,
            ): [boolean, string] {
                if (track1.getKey() !== track2.getKey())
                    return [
                        false,
                        `Keys mismatched: ${track1.getKey()} != ${track2.getKey()}`,
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

                if (track1.getClips().length !== track2.getClips().length) {
                    equal = false;
                    reasons.push("Number of clips mismatched");
                }

                return [
                    equal,
                    reasons.length === 0 ? null : reasons.join("\n"),
                ];
            }

            function containsEqualAudioTrack(
                list: AudioTrack[],
                track: AudioTrack,
            ): boolean {
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

            test("Reconstruct track with clip", () => {
                const storedClip = importTestClip();
                const newAudioTrack = engine.addAudioTrack();
                newAudioTrack.addClip(storedClip, Timestamp.zero());
                const before = engine.getAudioTracks();
                const state = engine.deleteAudioTrack(newAudioTrack);
                engine.reconstructAudioTrack(state);

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

                for (const method of methods) expect(engine[method]).toThrow();

                // So cleanup can run
                engine = Engine.getDummy();
            });
        });

        describe("Individual track", () => {
            describe("Master track", () => {
                testTrackCommon(() => engine.getMaster());
            });

            describe("Audio track", () => {
                let track: AudioTrack;

                beforeEach(() => (track = engine.addAudioTrack()));

                testTrackCommon(() => engine.addAudioTrack());

                test("Has key", () => {
                    expect(typeof track.getKey()).toBe("number");
                });

                test("getClips()", () => {
                    expect(track.getClips()).toBeDefined();
                });

                test("addClip()", () => {
                    const storedClip = importTestClip();

                    expect(track.getClips().length).toBe(0);

                    track.addClip(storedClip, Timestamp.zero());

                    expect(track.getClips().length).toBe(1);
                });

                test("deleteClip()", () => {
                    const storedClip = importTestClip();
                    const timelineClip = track.addClip(
                        storedClip,
                        Timestamp.zero(),
                    );

                    expect(track.getClips().length).toBe(1);

                    expect(track.deleteClip(timelineClip)).toBeDefined();

                    expect(track.getClips().length).toBe(0);
                });

                test("deleteClips()", () => {
                    const storedClip = importTestClip();

                    const timelineClips = [];
                    for (let i = 0; i < 43; i++) {
                        timelineClips.push(
                            track.addClip(
                                storedClip,
                                Timestamp.fromBeats(i),
                                Timestamp.fromBeats(1),
                            ),
                        );
                    }

                    expect(track.deleteClips(timelineClips)).toBeDefined();
                });

                test("reconstructClip()", () => {
                    const storedClip = importTestClip();
                    const clip1 = track.addClip(storedClip, Timestamp.zero());
                    const state = track.deleteClip(clip1);
                    const clip2 = track.reconstructClip(state);

                    expect(clip1.getKey()).toBe(clip2.getKey());
                });

                test("reconstructClips()", () => {
                    const storedClip = importTestClip();
                    const timelineClips1 = [];
                    for (let i = 0; i < 43; i++) {
                        timelineClips1.push(
                            track.addClip(
                                storedClip,
                                Timestamp.fromBeats(i),
                                Timestamp.fromBeats(1),
                            ),
                        );
                    }
                    const states = track.deleteClips(timelineClips1);
                    const timelineClips2 = track.reconstructClips(states);

                    expect(timelineClips1.length).toBe(timelineClips2.length);
                    for (let i = 0; i < timelineClips1.length; i++) {
                        expect(timelineClips1[i].getKey()).toBe(
                            timelineClips2[i].getKey(),
                        );
                    }
                });

                test("delete() deletes track", () => {
                    track.delete();
                    expect(
                        engine
                            .getAudioTracks()
                            .some(t => t.getKey() === track.getKey()),
                    ).toBe(false);
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

                        "getKey",
                        "getClips",
                        "addClip",
                        "deleteClip",
                        "deleteClips",
                        "reconstructClip",
                        "reconstructClips",
                        "delete",
                    ];

                    for (const method of methods)
                        expect(track[method]).toThrow();
                });
            });

            function testTrackCommon(initTrack: () => Track) {
                let track: Track;

                beforeEach(() => (track = initTrack()));

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
                        expect(stat.length).toBe(2);

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
            expect(() => engine.importAudioClip("nonexistent")).toThrow();
        });

        test("getKey()", () => {
            const clip = importTestClip();
            expect(clip.getKey()).toBeDefined();
        });

        test("getSampleRate()", () => {
            const clip = importTestClip();
            expect(clip.getSampleRate()).toBe(48_000);
        });

        test("getLength()", () => {
            const clip = importTestClip();
            expect(clip.getLength()).toBe(1_322_978);
        });
    });

    describe("Timeline audio clip", () => {
        let track: AudioTrack;
        let clip: AudioClip;

        beforeEach(() => {
            track = engine.addAudioTrack();
            clip = track.addClip(
                importTestClip(),
                Timestamp.fromBeats(1),
                Timestamp.fromBeats(2),
            );
        });

        test("getKey()", () => {
            expect(typeof clip.getKey()).toBe("number");
        });

        test("getStart()", () => {
            expect(clip.getStart().getBeats()).toBe(1);
        });

        test("getLength()", () => {
            expect(clip.getLength().getBeats()).toBe(2);
        });

        test("getLength() null", () => {
            const track = engine.addAudioTrack();
            clip = track.addClip(importTestClip(), Timestamp.fromBeats(1));
            expect(clip.getLength().getBeats()).toBe(55);
        });

        test("move()", () => {
            clip.move(Timestamp.fromBeats(2));
            expect(clip.getStart().getBeats()).toBe(2);
        });

        test("moveToTrack()", () => {
            const track2 = engine.addAudioTrack();

            clip.moveToTrack(Timestamp.fromBeats(2), track2);

            expect(track.getClips().length).toBe(0);
            expect(track2.getClips().length).toBe(1);
            expect(clip.getStart().getBeats()).toBe(2);
        });

        test("moveToTrack() overlap", () => {
            const track2 = engine.addAudioTrack();
            track2.addClip(
                importTestClip(),
                Timestamp.fromBeats(3),
                Timestamp.fromBeats(1),
            );

            expect(() =>
                clip.moveToTrack(Timestamp.fromBeats(2), track2),
            ).toThrow();
        });

        test("cropStart()", () => {
            clip.cropStart(Timestamp.fromBeats(1));
            expect(clip.getStart().getBeats()).toBe(2);
            expect(clip.getLength().getBeats()).toBe(1);
        });

        test("cropEnd()", () => {
            clip.cropEnd(Timestamp.fromBeats(1));
            expect(clip.getStart().getBeats()).toBe(1);
            expect(clip.getLength().getBeats()).toBe(1);
        });

        test("getStoredClip()", () => {
            expect(clip.getStoredClip()).toBeDefined();
        });

        test("delete()", () => {
            expect(clip.delete()).toBeDefined();
            const methods = [
                "getKey",
                "getStart",
                "getLength",
                "getStoredClip",
                "delete",
            ];

            for (const method of methods) expect(clip[method]).toThrow();
        });
    });
});

test("inverseMeterScale() is inverse of meterScale()", () => {
    const result = inverseMeterScale(meterScale(0.6));
    expect(Math.abs(result - 0.6)).toBeLessThan(0.00001);
});
