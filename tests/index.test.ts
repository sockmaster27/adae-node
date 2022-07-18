const { Engine } = require("../index.node");

let engine;
beforeEach(() => {
    engine = new Engine();
});

test("Has tracks", () => {
    expect(engine.tracks).toBeDefined();
});

test("Add single track", () => {
    const before = engine.tracks.length;
    const track = engine.addTrack();
    expect(engine.tracks.length).toStrictEqual(before + 1);
    expect(engine.tracks).toContain(track);
});

test("Add number of tracks", () => {
    const before = engine.tracks.length;
    const tracks = engine.addTracks(5);
    expect(engine.tracks.length).toStrictEqual(before + 5);
    for (const track of tracks)
        expect(engine.tracks).toContain(track);
});

// Test currently fails, because deleted tracks aren't removed from engine.tracks
test("Delete track", () => {
    const before = [...engine.tracks];
    const newTrack = engine.addTrack();
    engine.deleteTrack(newTrack);
    expect(engine.tracks).toEqual(before);
});
