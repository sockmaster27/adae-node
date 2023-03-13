const { Engine, inverseMeterScale, meterScale } = require("../../index.node"); // Relative to the destination



describe("Engine", () => {
    let engine: any;
    beforeEach(() => {
        engine = new Engine();
    });
    afterEach(() => {
        engine.close();
    });

    // Common for timeline and mixer
    function testTrackHandlingCommon(container: any, tracksEqual: (t1: any, t2: any) => [boolean, string]) {

        function containsEqualTrack(list: any[], track: any): boolean {
            for (const listTrack of list) {
                if (tracksEqual(listTrack, track))
                    return true;
            }
            return false;
        }

        test("Has tracks", () => {
            expect(container.getTracks()).toBeDefined();
        });

        test("Get track from key", () => {
            const track = container.addTrack();
            const gottenTrack = container.getTrack(track.key);
            expect(tracksEqual(track, gottenTrack)).toStrictEqual([true, null]);
        });

        test("Get track from key fails when track is deleted", () => {
            const track = container.addTrack();
            container.deleteTrack(track);
            expect(() => container.getTrack(track.key)).toThrowError();
        });

        test("Get track from key after reconstruction", () => {
            const track = container.addTrack();
            const data = container.deleteTrack(track);
            container.addTrack(data);
            const gottenTrack = container.getTrack(track.key);
            expect(tracksEqual(track, gottenTrack)).toStrictEqual([true, null]);
        });

        test("Add single track", () => {
            const before = container.getTracks().length;
            const newTrack = container.addTrack();
            expect(container.getTracks().length).toBe(before + 1);
            expect(
                containsEqualTrack(container.getTracks(), newTrack)
            ).toBe(true);
        });

        test("Add number of tracks", () => {
            const before = container.getTracks().length;
            const newTracks = container.addTracks(5);
            expect(container.getTracks().length).toBe(before + 5);
            for (const track of newTracks)
                expect(
                    containsEqualTrack(container.getTracks(), track)
                ).toBe(true);

        });

        test("Delete track", () => {
            const before = container.getTracks();
            const newTrack = container.addTrack();
            container.deleteTrack(newTrack);

            expect(container.getTracks().length).toBe(before.length);

            for (const track of container.getTracks())
                expect(
                    containsEqualTrack(before, track)
                ).toBe(true);
        });

        test("Delete multiple tracks", () => {
            const before = container.getTracks();
            const newTracks = container.addTracks(34);
            container.deleteTracks(newTracks);

            expect(container.getTracks().length).toBe(before.length);

            for (const track of container.getTracks())
                expect(
                    containsEqualTrack(before, track)
                ).toBe(true);
        });

        test("Reconstruct single track", () => {
            const newTrack = container.addTrack();
            const before = container.getTracks();
            const data = container.deleteTrack(newTrack);
            container.addTrack(data);

            expect(container.getTracks().length).toBe(before.length);

            for (const track of container.getTracks())
                expect(
                    containsEqualTrack(before, track)
                ).toBe(true);
        });

        test("Reconstruct multiple tracks", () => {
            const newTracks = container.addTracks(24);
            const before = container.getTracks();
            const data = container.deleteTracks(newTracks);
            container.addTracks(data);

            expect(container.getTracks().length).toBe(before.length);

            for (const track of container.getTracks())
                expect(
                    containsEqualTrack(before, track)
                ).toBe(true);
        });

        test("All methods throw when engine is closed", () => {
            engine.close();
            const methods = [
                "getTracks",
                "getTrack",
                "addTrack",
                "addTracks",
                "deleteTrack",
                "deleteTracks",
                "close",
            ];

            for (const method of methods)
                expect(container[method]).toThrowError();

            // So cleanup can run
            engine = new Engine();
        });
    }

    describe("Mixer", () => {
        test("Get master", () => {
            expect(engine.getMaster()).toBeDefined();
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

            testTrackHandlingCommon(engine, tracksEqual);
        });


        describe("Individual track", () => {
            let track: any;

            describe("Master track", () => {
                beforeEach(() => track = engine.getMaster());

                testTrackCommon();
            });

            describe("Regular track", () => {
                beforeEach(() => track = engine.addTrack());

                testTrackCommon();

                test("Has key", () => {
                    expect(typeof track.key).toBe("number");
                });

                test("delete() deletes track", () => {
                    track.delete();
                    expect(() => engine.getTrack(track.key)).toThrowError();
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

    describe("Timeline", () => {
        test("Get timeline", () => {
            expect(engine.getTimeline()).toBeDefined();
        });

        describe("Track addition and deletion", () => {
            function tracksEqual(track1: any, track2: any): [boolean, string] {
                if (track1.key !== track2.key)
                    return [false, `Keys mismatched: ${track1.key} != ${track2.key}`]
                else
                    return [true, null]
            }

            testTrackHandlingCommon(engine.timeline, tracksEqual)
        });
    });
});

test("inverseMeterScale() is inverse of meterScale()", () => {
    const result = inverseMeterScale(meterScale(0.6));
    expect(Math.abs(result - 0.6)).toBeLessThan(0.00001);
});
